//! Server-Sent Events (SSE) broadcaster.
//!
//! Semua event penting (block committed, node status, chaincode TX)
//! di-push ke semua frontend client yang sedang subscribe ke
//! GET /api/v1/events/stream — tanpa polling.
//!
//! Frontend cukup:
//!   const es = new EventSource('/api/v1/events/stream', { headers: { Authorization: ... } })
//!   es.addEventListener('block', e => console.log(JSON.parse(e.data)))

use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

// ── Event types yang dikirim ke frontend ─────────────────────────────────────

/// Block baru berhasil di-commit ke ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEvent {
    pub block_num: u64,
    pub tx_count:  usize,
    pub timestamp: u64,
}

/// Satu peer yang sedang terhubung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub addr:    String,
}

/// Status node berubah (connected peer baru, dll)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusEvent {
    pub status:       String,   // "ok" | "degraded"
    pub peer_count:   usize,
    pub latest_block: Option<u64>,
    pub mode:         String,   // "dev" | "consensus"
    pub peers:        Vec<PeerInfo>,
}

/// Contract TX selesai dieksekusi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEvent {
    pub tx_id:       String,
    pub block_num:   u64,
    pub contract_id: String,
    pub function_name: String,
    pub caller:      String,
    pub success:     bool,
}

/// Envelope semua event types
#[derive(Debug, Clone)]
pub enum QorvumEvent {
    Block(BlockEvent),
    NodeStatus(NodeStatusEvent),
    Tx(TxEvent),
    /// Heartbeat — dikirim tiap 30 detik untuk keep connection alive
    Heartbeat,
}

impl QorvumEvent {
    /// SSE event name (dipakai di `es.addEventListener('block', ...)`)
    pub fn event_name(&self) -> &'static str {
        match self {
            QorvumEvent::Block(_)      => "block",
            QorvumEvent::NodeStatus(_) => "node_status",
            QorvumEvent::Tx(_)         => "tx",
            QorvumEvent::Heartbeat     => "heartbeat",
        }
    }

    /// JSON data untuk event ini
    pub fn data(&self) -> String {
        match self {
            QorvumEvent::Block(e)      => serde_json::to_string(e).unwrap_or_default(),
            QorvumEvent::NodeStatus(e) => serde_json::to_string(e).unwrap_or_default(),
            QorvumEvent::Tx(e)         => serde_json::to_string(e).unwrap_or_default(),
            QorvumEvent::Heartbeat     => r#"{"ts":"heartbeat"}"#.to_string(),
        }
    }
}

// ── Broadcaster ───────────────────────────────────────────────────────────────

/// Shared broadcaster — clone murah, semua handler bisa pakai.
#[derive(Clone)]
pub struct EventBroadcaster {
    tx: broadcast::Sender<QorvumEvent>,
    /// Cached last node_status so new WS/SSE clients can get current state on connect.
    last_node_status: Arc<std::sync::RwLock<Option<NodeStatusEvent>>>,
}

impl EventBroadcaster {
    /// Capacity 256 — cukup untuk burst events tanpa memory waste
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx, last_node_status: Arc::new(std::sync::RwLock::new(None)) }
    }

    /// Kirim event ke semua subscriber. Silent kalau tidak ada subscriber.
    pub fn send(&self, event: QorvumEvent) {
        let _ = self.tx.send(event);
    }

    /// Subscribe ke event stream (satu per SSE connection)
    pub fn subscribe(&self) -> broadcast::Receiver<QorvumEvent> {
        self.tx.subscribe()
    }

    /// Returns the last node_status snapshot — used by new WS clients on connect.
    pub fn current_node_status(&self) -> Option<NodeStatusEvent> {
        self.last_node_status.read().unwrap().clone()
    }

    // ── Convenience senders ───────────────────────────────────────────────────

    pub fn block_committed(&self, block_num: u64, tx_count: usize) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.send(QorvumEvent::Block(BlockEvent { block_num, tx_count, timestamp: ts }));
    }

    pub fn node_status(&self, status: &str, peer_count: usize, latest_block: Option<u64>, mode: &str, peers: Vec<PeerInfo>) {
        let event = NodeStatusEvent {
            status:       status.to_string(),
            peer_count,
            latest_block,
            mode:         mode.to_string(),
            peers,
        };
        *self.last_node_status.write().unwrap() = Some(event.clone());
        self.send(QorvumEvent::NodeStatus(event));
    }

    pub fn tx_committed(&self, tx_id: String, block_num: u64, contract_id: String, function_name: String, caller: String, success: bool) {
        self.send(QorvumEvent::Tx(TxEvent {
            tx_id, block_num, contract_id, function_name, caller, success,
        }));
    }
}

// ── SSE Stream handler ────────────────────────────────────────────────────────

/// Buat SSE stream dari broadcaster untuk satu client connection.
/// Dipanggil dari handler GET /api/v1/events/stream
pub fn make_sse_stream(
    broadcaster: Arc<EventBroadcaster>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = broadcaster.subscribe();

    let stream = BroadcastStream::new(rx)
        .filter_map(|result| {
            // BroadcastStream error = lagged (buffer penuh) → skip
            result.ok()
        })
        .map(|event| {
            let sse_event = Event::default()
                .event(event.event_name())
                .data(event.data());
            Ok::<Event, Infallible>(sse_event)
        });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}

// ── Heartbeat task ────────────────────────────────────────────────────────────

/// Spawn background task yang kirim heartbeat tiap 30 detik.
/// Ini memastikan koneksi SSE tidak di-timeout oleh load balancer / proxy.
pub fn spawn_heartbeat(broadcaster: Arc<EventBroadcaster>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.tick().await; // skip first immediate tick
        loop {
            interval.tick().await;
            broadcaster.send(QorvumEvent::Heartbeat);
        }
    });
}