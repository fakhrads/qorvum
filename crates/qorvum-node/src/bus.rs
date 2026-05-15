//! NodeBus — typed internal message bus connecting roles within one process.

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch, Mutex};
use serde::{Deserialize, Serialize};

/// Connected peer snapshot — updated whenever P2P topology changes.
#[derive(Debug, Clone)]
pub struct ConnectedPeer {
    pub peer_id: String,
    pub addr:    String,
}

/// A transaction submission from GatewayRole to ConsensusRole.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSubmission {
    pub tx_id:         [u8; 32],
    pub contract_id:   String,
    pub function_name: String,
    pub args:          serde_json::Value,
    pub caller_id:     String,
    pub caller_org:    String,
    pub caller_roles:  Vec<String>,
    pub verified:      bool,
    pub timestamp:     u64,
}

/// Notification that a block was committed to the ledger.
#[derive(Debug, Clone)]
pub struct BlockCommittedEvent {
    pub block_num: u64,
    pub tx_count:  usize,
}

/// Raw P2P message received from a peer.
#[derive(Debug, Clone)]
pub struct P2PInbound {
    pub topic: String,
    pub data:  Vec<u8>,
}

/// Raw bytes to broadcast over P2P to all peers on a topic.
#[derive(Debug, Clone)]
pub struct P2POutbound {
    pub topic: String,
    pub data:  Vec<u8>,
}

const TX_BUF:     usize = 512;
const BLOCK_BUF:  usize = 64;
const P2P_BUF:    usize = 512;

/// Cloneable handle to all inter-role channels.
#[derive(Clone)]
pub struct NodeBus {
    /// Gateway → Consensus: new transaction to execute and commit
    pub tx_tx: mpsc::Sender<TxSubmission>,
    tx_rx:     Arc<Mutex<mpsc::Receiver<TxSubmission>>>,

    /// Consensus → Gateway (broadcast): a block was committed
    pub block_committed_tx: broadcast::Sender<BlockCommittedEvent>,

    /// Peer → Consensus: raw inbound P2P message
    pub p2p_in_tx: mpsc::Sender<P2PInbound>,
    p2p_in_rx:     Arc<Mutex<mpsc::Receiver<P2PInbound>>>,

    /// Consensus → Peer: bytes to broadcast over P2P
    pub p2p_out_tx: mpsc::Sender<P2POutbound>,
    p2p_out_rx:     Arc<Mutex<mpsc::Receiver<P2POutbound>>>,

    /// PeerRole → GatewayRole: current connected peer list (watch = latest value)
    pub peer_status_tx: watch::Sender<Vec<ConnectedPeer>>,
}

impl NodeBus {
    pub fn new() -> Self {
        let (tx_tx, tx_rx)           = mpsc::channel(TX_BUF);
        let (block_tx, _)            = broadcast::channel(BLOCK_BUF);
        let (p2p_in_tx, p2p_in_rx)   = mpsc::channel(P2P_BUF);
        let (p2p_out_tx, p2p_out_rx) = mpsc::channel(P2P_BUF);
        let (peer_tx, _)             = watch::channel(Vec::new());

        Self {
            tx_tx,
            tx_rx:             Arc::new(Mutex::new(tx_rx)),
            block_committed_tx: block_tx,
            p2p_in_tx,
            p2p_in_rx:         Arc::new(Mutex::new(p2p_in_rx)),
            p2p_out_tx,
            p2p_out_rx:        Arc::new(Mutex::new(p2p_out_rx)),
            peer_status_tx:    peer_tx,
        }
    }

    /// Subscribe to peer topology updates — receives current list on each change.
    pub fn peer_status_rx(&self) -> watch::Receiver<Vec<ConnectedPeer>> {
        self.peer_status_tx.subscribe()
    }

    /// ConsensusRole uses this to receive TX submissions from GatewayRole.
    pub fn tx_receiver(&self) -> Arc<Mutex<mpsc::Receiver<TxSubmission>>> {
        self.tx_rx.clone()
    }

    /// Subscribe to block-committed broadcast events (multiple receivers allowed).
    pub fn block_committed_rx(&self) -> broadcast::Receiver<BlockCommittedEvent> {
        self.block_committed_tx.subscribe()
    }

    /// PeerRole uses this to receive bytes to publish on gossipsub.
    pub fn p2p_out_receiver(&self) -> Arc<Mutex<mpsc::Receiver<P2POutbound>>> {
        self.p2p_out_rx.clone()
    }

    /// ConsensusRole uses this to receive inbound P2P messages from PeerRole.
    pub fn p2p_in_receiver(&self) -> Arc<Mutex<mpsc::Receiver<P2PInbound>>> {
        self.p2p_in_rx.clone()
    }
}