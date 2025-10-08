//! Inter-process communication (IPC) subsystem

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ChannelId = u64;

#[derive(Debug, Clone)]
pub struct Message {
    pub from: u64,
    pub to: u64,
    pub data: Vec<u8>,
}

lazy_static::lazy_static! {
    static ref MESSAGE_QUEUES: Arc<Mutex<HashMap<ChannelId, Vec<Message>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize IPC subsystem
pub fn init() -> Result<(), &'static str> {
    println!("[IPC] Initializing inter-process communication...");
    Ok(())
}

/// Create a new channel
pub fn create_channel(channel_id: ChannelId) -> Result<(), &'static str> {
    let mut queues = MESSAGE_QUEUES.lock().unwrap();
    queues.insert(channel_id, Vec::new());
    Ok(())
}

/// Send a message
pub fn send_message(channel_id: ChannelId, message: Message) -> Result<(), &'static str> {
    let mut queues = MESSAGE_QUEUES.lock().unwrap();
    if let Some(queue) = queues.get_mut(&channel_id) {
        queue.push(message);
        Ok(())
    } else {
        Err("Channel not found")
    }
}

/// Receive a message
pub fn receive_message(channel_id: ChannelId) -> Option<Message> {
    let mut queues = MESSAGE_QUEUES.lock().unwrap();
    if let Some(queue) = queues.get_mut(&channel_id) {
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    } else {
        None
    }
}
