use anyhow::Result;
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sled::{IVec, Tree};
use std::{path::Path, sync::Arc};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub created_ts: i64,    // unix ms
    pub not_before_ts: i64, // unix ms (for delayed processing/retry backoff)
    pub attempts: u32,
    pub payload: serde_json::Value, // IngestRequest as JSON
    pub status: String,             // "pending"|"in_progress"|"done"|"failed"
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItemBrief {
    pub id: String,
    pub status: String,
    pub attempts: u32,
    pub created_ts: i64,
    pub not_before_ts: i64,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct Queue {
    items: Tree,
    order: Tree,          // key: not_before_ts:uuid -> id (for time-ordered processing)
    lock: Arc<Mutex<()>>, // Ensure atomic pop operations
}

impl Queue {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        std::fs::create_dir_all(&path).ok();
        let db = sled::open(path)?;
        info!("Opened persistent queue database");

        Ok(Self {
            items: db.open_tree("items")?,
            order: db.open_tree("order")?,
            lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn enqueue(&self, payload: serde_json::Value, delay_ms: i64) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        let not_before = now + delay_ms.max(0);

        let item = QueueItem {
            id: id.clone(),
            created_ts: now,
            not_before_ts: not_before,
            attempts: 0,
            payload,
            status: "pending".to_string(),
            last_error: None,
        };

        let order_key = Self::make_order_key(not_before, &id);

        // Store item and its order key atomically
        self.items
            .insert(id.as_bytes(), serde_json::to_vec(&item)?)?;
        self.order.insert(order_key, id.as_bytes())?;

        debug!("Enqueued item {} for processing at {}", id, not_before);
        Ok(id)
    }

    pub fn requeue_with_delay(&self, id: &str, delay_ms: i64, error: Option<String>) -> Result<()> {
        let mut item = self
            .get(id)?
            .ok_or_else(|| anyhow::anyhow!("Item not found: {}", id))?;

        let now = Utc::now().timestamp_millis();
        item.status = "pending".to_string();
        item.not_before_ts = now + delay_ms.max(0);
        item.last_error = error;

        let order_key = Self::make_order_key(item.not_before_ts, &item.id);

        // Update item and add back to order queue
        self.items
            .insert(id.as_bytes(), serde_json::to_vec(&item)?)?;
        self.order.insert(order_key, id.as_bytes())?;

        debug!("Requeued item {} for retry at {}", id, item.not_before_ts);
        Ok(())
    }

    pub fn requeue(&self, id: &str) -> Result<()> {
        self.requeue_with_delay(id, 0, None)
    }

    pub fn mark_done(&self, id: &str) -> Result<()> {
        if let Some(mut item) = self.get(id)? {
            item.status = "done".to_string();
            self.items
                .insert(id.as_bytes(), serde_json::to_vec(&item)?)?;
            debug!("Marked item {} as done", id);
        }
        Ok(())
    }

    pub fn mark_failed(&self, id: &str, error: String) -> Result<()> {
        if let Some(mut item) = self.get(id)? {
            item.status = "failed".to_string();
            item.last_error = Some(error);
            self.items
                .insert(id.as_bytes(), serde_json::to_vec(&item)?)?;
            warn!(
                "Marked item {} as failed: {}",
                id,
                item.last_error.as_ref().unwrap_or(&"unknown".to_string())
            );
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<QueueItem>> {
        Ok(self
            .items
            .get(id.as_bytes())?
            .map(|v| serde_json::from_slice::<QueueItem>(&v))
            .transpose()?)
    }

    pub fn list(&self, status: Option<&str>, limit: usize) -> Result<Vec<QueueItemBrief>> {
        let mut items = Vec::new();

        for result in self.items.iter() {
            let (_, value_bytes) = result?;
            if let Ok(item) = serde_json::from_slice::<QueueItem>(&value_bytes) {
                // Filter by status if specified
                if status.map(|s| s == item.status).unwrap_or(true) {
                    items.push(QueueItemBrief {
                        id: item.id,
                        status: item.status,
                        attempts: item.attempts,
                        created_ts: item.created_ts,
                        not_before_ts: item.not_before_ts,
                        last_error: item.last_error,
                    });

                    if items.len() >= limit {
                        break;
                    }
                }
            }
        }

        // Sort by creation time, newest first
        items.sort_by_key(|item| std::cmp::Reverse(item.created_ts));

        debug!(
            "Listed {} queue items (status filter: {:?}, limit: {})",
            items.len(),
            status,
            limit
        );
        Ok(items)
    }

    pub fn stats(&self) -> Result<serde_json::Value> {
        let mut counts = std::collections::HashMap::new();
        counts.insert("pending".to_string(), 0i64);
        counts.insert("in_progress".to_string(), 0i64);
        counts.insert("done".to_string(), 0i64);
        counts.insert("failed".to_string(), 0i64);

        for result in self.items.iter() {
            let (_, value_bytes) = result?;
            if let Ok(item) = serde_json::from_slice::<QueueItem>(&value_bytes) {
                *counts.entry(item.status).or_insert(0) += 1;
            }
        }

        let total_items = self.items.len();
        let pending_items = self.order.len();

        Ok(serde_json::json!({
            "total_items": total_items,
            "pending_in_order": pending_items,
            "status_counts": counts,
            "timestamp": Utc::now().timestamp_millis()
        }))
    }

    pub fn requeue_failed(&self) -> Result<i64> {
        let mut count = 0i64;

        for result in self.items.iter() {
            let (key, value) = result?;
            if let Ok(mut item) = serde_json::from_slice::<QueueItem>(&value) {
                if item.status == "failed" {
                    item.status = "pending".to_string();
                    item.not_before_ts = Utc::now().timestamp_millis();
                    item.last_error = None;

                    let order_key = Self::make_order_key(item.not_before_ts, &item.id);
                    self.items.insert(key, serde_json::to_vec(&item)?)?;
                    self.order.insert(order_key, item.id.as_bytes())?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Pop the earliest ready item (not_before_ts <= now)
    /// Marks as in_progress and removes from order queue
    pub fn pop_ready(&self) -> Result<Option<QueueItem>> {
        let _guard = self.lock.lock();
        let now = Utc::now().timestamp_millis();

        // Scan order queue for the earliest ready item
        if let Some(result) = self.order.iter().next() {
            let (order_key, id_bytes) = result?;
            let (timestamp, _uuid) = Self::parse_order_key(&order_key)?;

            if timestamp > now {
                // Earliest item is not ready yet
                return Ok(None);
            }

            let id = std::str::from_utf8(&id_bytes)?.to_string();

            // Remove from order queue first
            self.order.remove(order_key)?;

            // Get and update the item
            if let Some(mut item) = self.get(&id)? {
                item.status = "in_progress".to_string();
                item.attempts += 1;
                self.items
                    .insert(id.as_bytes(), serde_json::to_vec(&item)?)?;

                debug!("Popped ready item {} (attempt {})", id, item.attempts);
                return Ok(Some(item));
            }
        }

        Ok(None)
    }

    /// Create time-ordered key: timestamp + uuid for lexical ordering
    fn make_order_key(timestamp: i64, id: &str) -> Vec<u8> {
        // Add offset to ensure positive numbers for lexical ordering
        let offset_timestamp = (timestamp as i128 + (1i128 << 60)) as u128;
        format!("{:016x}:{}", offset_timestamp, id).into_bytes()
    }

    /// Parse order key back to timestamp and uuid
    fn parse_order_key(key: &IVec) -> Result<(i64, String)> {
        let key_str = std::str::from_utf8(key)?;
        let (hex_timestamp, uuid) = key_str
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid order key format"))?;

        let offset_timestamp = u128::from_str_radix(hex_timestamp, 16)?;
        let timestamp = (offset_timestamp as i128 - (1i128 << 60)) as i64;

        Ok((timestamp, uuid.to_string()))
    }
}

pub async fn worker_loop(queue: Arc<Queue>) {
    let poll_interval = std::env::var("AGENT_QUEUE_POLL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000u64);
    let sleep_duration = Duration::from_millis(poll_interval);

    loop {
        match queue.pop_ready() {
            Ok(Some(item)) => {
                info!("Processing queued task {}", item.id);
                if let Err(e) = queue.mark_done(&item.id) {
                    warn!("Failed to mark queue item {} done: {}", item.id, e);
                }
            }
            Ok(None) => sleep(sleep_duration).await,
            Err(e) => {
                warn!("Queue worker error: {}", e);
                sleep(sleep_duration).await;
            }
        }
    }
}
