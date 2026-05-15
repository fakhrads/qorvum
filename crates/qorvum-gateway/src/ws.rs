//! WebSocket handler — real-time blockchain explorer stream.
//!
//! Connect: ws://host/api/v1/ws[?token=<bearer>]
//!
//! # Client → Server
//! ```json
//! { "type": "ping" }
//! { "type": "subscribe",   "topics": ["blocks", "tx", "node_status"] }
//! { "type": "unsubscribe", "topics": ["tx"] }
//! ```
//!
//! # Server → Client
//! ```json
//! { "type": "connected",    "data": { "channel": "...", "latest_block": 42, "mode": "dev" } }
//! { "type": "block",        "data": { "block_num": 43, "tx_count": 2, "timestamp": 1234567890 } }
//! { "type": "tx",           "data": { "tx_id": "...", "block_num": 43, ... } }
//! { "type": "node_status",  "data": { "status": "ok", "peer_count": 3, ... } }
//! { "type": "heartbeat" }
//! { "type": "pong" }
//! { "type": "subscribed",   "topics": ["blocks"] }
//! { "type": "unsubscribed", "topics": ["tx"] }
//! { "type": "error",        "message": "invalid message format" }
//! ```
//!
//! Topics available: `blocks`, `tx`, `node_status`, `heartbeat`
//! Default: all topics active on connect.

use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::sse::QorvumEvent;
use crate::state::AppState;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WsQuery {
    /// Bearer token — browser WebSocket API cannot set custom headers,
    /// so auth token is passed as a query param instead.
    #[allow(dead_code)]
    pub token: Option<String>,
}

// ── Inbound message schema ────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    Ping,
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
}

// ── Upgrade handler ───────────────────────────────────────────────────────────

/// GET /api/v1/ws — upgrades HTTP connection to WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.token))
}

// ── Per-connection loop ───────────────────────────────────────────────────────

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, _token: Option<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx: broadcast::Receiver<QorvumEvent> = state.broadcaster.subscribe();

    // All topics active by default; client can narrow down with subscribe/unsubscribe.
    let mut active: HashSet<&'static str> =
        ["blocks", "tx", "node_status", "heartbeat"].into_iter().collect();

    // Send welcome frame
    let latest = state.store.get_latest_block_num().unwrap_or(None);
    let mode = if state.consensus.is_some() { "consensus" } else { "dev" };

    let welcome = serde_json::to_string(&json!({
        "type": "connected",
        "data": {
            "channel":      state.channel_id,
            "latest_block": latest,
            "mode":         mode,
        }
    }))
    .unwrap_or_default();

    if sender.send(Message::Text(welcome)).await.is_err() {
        return;
    }

    // Send current peer topology snapshot immediately so the client doesn't
    // have to wait for the next topology-change event after a page reload.
    if let Some(status) = state.broadcaster.current_node_status() {
        let frame = serde_json::to_string(&json!({ "type": "node_status", "data": status }))
            .unwrap_or_default();
        if sender.send(Message::Text(frame)).await.is_err() {
            return;
        }
    }

    info!(channel = %state.channel_id, "ws: client connected");

    loop {
        tokio::select! {
            // ── Push events from broadcaster ──────────────────────────────
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        let topic: &'static str = match &event {
                            QorvumEvent::Block(_)      => "blocks",
                            QorvumEvent::Tx(_)         => "tx",
                            QorvumEvent::NodeStatus(_) => "node_status",
                            QorvumEvent::Heartbeat     => "heartbeat",
                        };

                        if !active.contains(topic) {
                            continue;
                        }

                        let frame = match &event {
                            QorvumEvent::Block(e) =>
                                json!({ "type": "block",       "data": e }),
                            QorvumEvent::Tx(e) =>
                                json!({ "type": "tx",          "data": e }),
                            QorvumEvent::NodeStatus(e) =>
                                json!({ "type": "node_status", "data": e }),
                            QorvumEvent::Heartbeat =>
                                json!({ "type": "heartbeat" }),
                        };

                        let text = serde_json::to_string(&frame).unwrap_or_default();
                        if sender.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("ws: client lagged, skipped {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }

            // ── Handle inbound client messages ────────────────────────────
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMsg>(&text) {
                            Ok(ClientMsg::Ping) => {
                                let pong = serde_json::to_string(
                                    &json!({ "type": "pong" })
                                ).unwrap_or_default();
                                if sender.send(Message::Text(pong)).await.is_err() { break; }
                            }

                            Ok(ClientMsg::Subscribe { topics }) => {
                                let added: Vec<&str> = topics.iter()
                                    .filter_map(|t| topic_str(t))
                                    .collect();
                                for t in &added { active.insert(t); }
                                let ack = serde_json::to_string(
                                    &json!({ "type": "subscribed", "topics": added })
                                ).unwrap_or_default();
                                if sender.send(Message::Text(ack)).await.is_err() { break; }
                            }

                            Ok(ClientMsg::Unsubscribe { topics }) => {
                                let removed: Vec<&str> = topics.iter()
                                    .filter_map(|t| topic_str(t))
                                    .collect();
                                for t in &removed { active.remove(t); }
                                let ack = serde_json::to_string(
                                    &json!({ "type": "unsubscribed", "topics": removed })
                                ).unwrap_or_default();
                                if sender.send(Message::Text(ack)).await.is_err() { break; }
                            }

                            Err(_) => {
                                let err = serde_json::to_string(
                                    &json!({ "type": "error", "message": "invalid message format" })
                                ).unwrap_or_default();
                                if sender.send(Message::Text(err)).await.is_err() { break; }
                            }
                        }
                    }

                    // Browser-level ping → reply with pong
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() { break; }
                    }

                    // Normal close or stream ended
                    Some(Ok(Message::Close(_))) | None => break,

                    _ => {}
                }
            }
        }
    }

    info!(channel = %state.channel_id, "ws: client disconnected");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Validate and convert a topic string to its canonical &'static str form.
fn topic_str(s: &str) -> Option<&'static str> {
    match s {
        "blocks"      => Some("blocks"),
        "tx"          => Some("tx"),
        "node_status" => Some("node_status"),
        "heartbeat"   => Some("heartbeat"),
        _             => None,
    }
}
