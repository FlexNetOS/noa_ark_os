use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

use crate::schema::PageEnvelope;
use crate::session::SessionBridge;

#[derive(Clone)]
pub struct UiApiState {
    pages: Arc<RwLock<HashMap<String, PageEnvelope>>>,
    session: Arc<Mutex<Option<SessionBridge>>>,
}

impl UiApiState {
    pub fn new() -> Self {
        Self {
            pages: Arc::new(RwLock::new(HashMap::new())),
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn pages(&self) -> &Arc<RwLock<HashMap<String, PageEnvelope>>> {
        &self.pages
    }

    pub fn session(&self) -> &Arc<Mutex<Option<SessionBridge>>> {
        &self.session
    }
}

#[derive(Clone)]
pub struct UiApiServer {
    state: UiApiState,
}

impl UiApiServer {
    pub fn new() -> Self {
        Self {
            state: UiApiState::new(),
        }
    }

    pub fn with_session(self, bridge: SessionBridge) -> Self {
        if let Ok(mut session) = self.state.session.lock() {
            *session = Some(bridge);
        }
        self
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/ui/pages/:page_id", get(Self::get_page))
            .route("/ui/pages/:page_id/events", get(Self::stream_events))
            .with_state(self.state.clone())
    }

    async fn get_page(
        State(state): State<UiApiState>,
        Path(page_id): Path<String>,
    ) -> Json<PageEnvelope> {
        let mut pages = state.pages.write().await;
        let envelope = pages
            .entry(page_id.clone())
            .or_insert_with(|| PageEnvelope::with_sample(&page_id))
            .clone();
        Json(envelope)
    }

    async fn stream_events(
        ws: WebSocketUpgrade,
        State(state): State<UiApiState>,
        Path(_page_id): Path<String>,
    ) -> impl IntoResponse {
        let bridge = state.session.lock().ok().and_then(|guard| guard.clone());
        if let Some(bridge) = bridge {
            ws.on_upgrade(move |socket| handle_websocket(socket, bridge))
        } else {
            (StatusCode::NOT_FOUND, "workflow streaming disabled").into_response()
        }
    }
}

async fn handle_websocket(mut socket: WebSocket, bridge: SessionBridge) {
    let mut events = bridge.subscribe();
    while let Some(event) = events.next().await {
        match event {
            Ok(event) => match serde_json::to_string(&SessionBridge::map_event(event)) {
                Ok(payload) => {
                    if socket.send(Message::Text(payload)).await.is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = socket
                        .send(Message::Text(format!("{{\"error\":\"{}\"}}", error)))
                        .await;
                }
            },
            Err(_) => continue,
        }
    }
}
