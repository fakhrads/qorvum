//! PeerRole — manages all P2P libp2p networking.
//!
//! Responsibilities:
//! - Run the libp2p Swarm (gossipsub + mDNS)
//! - Receive P2POutbound from NodeBus and publish to gossipsub topics
//! - Receive gossipsub messages from peers and push as P2PInbound to NodeBus
//! - Connect to bootstrap peers on startup

use std::sync::Arc;
use tracing::{error, info, warn};

use qorvum_ledger::store::LedgerStore;
use qorvum_network::NetworkService;

use crate::bus::{NodeBus, P2PInbound};

pub struct PeerRole {
    p2p_listen:      String,
    bootstrap_peers: Vec<String>,
    store:           Arc<dyn LedgerStore>,
    bus:             NodeBus,
}

impl PeerRole {
    pub fn new(
        p2p_listen:      String,
        bootstrap_peers: Vec<String>,
        store:           Arc<dyn LedgerStore>,
        bus:             NodeBus,
    ) -> Self {
        Self { p2p_listen, bootstrap_peers, store, bus }
    }

    pub async fn run(self) {
        info!("[peer] Role started — P2P listen: {}", self.p2p_listen);

        let (network_service, net_handles) =
            match NetworkService::new(&self.p2p_listen, self.bootstrap_peers.clone(), None, None) {
                Ok(pair) => pair,
                Err(e) => {
                    error!("[peer] Failed to initialize libp2p: {}", e);
                    return;
                }
            };

        // Bridge: peer topology events → NodeBus watch channel
        let peer_status_tx = self.bus.peer_status_tx.clone();
        let mut peer_events = net_handles.peer_events;
        tokio::spawn(async move {
            while let Some(peers) = peer_events.recv().await {
                let connected: Vec<crate::bus::ConnectedPeer> = peers.into_iter()
                    .map(|p| crate::bus::ConnectedPeer { peer_id: p.peer_id, addr: p.addr })
                    .collect();
                let _ = peer_status_tx.send(connected);
            }
        });

        // Bridge: NodeBus P2POutbound → gossipsub publish
        let consensus_out = net_handles.consensus_out.clone();
        let p2p_out_rx = self.bus.p2p_out_receiver();
        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut rx = p2p_out_rx.lock().await;
                    rx.recv().await
                };
                match msg {
                    Some(m) if m.topic == "qorvum-consensus" => {
                        if let Err(e) = consensus_out.send(m.data).await {
                            warn!("[peer] consensus_out send failed: {}", e);
                        }
                    }
                    Some(m) => {
                        // tx topic or others — could add tx_broadcast here
                        let _ = m;
                    }
                    None => {
                        warn!("[peer] p2p_out channel closed");
                        break;
                    }
                }
            }
        });

        // Bridge: gossipsub consensus_in → NodeBus P2PInbound
        let p2p_in_tx = self.bus.p2p_in_tx.clone();
        let mut consensus_in = net_handles.consensus_in;
        tokio::spawn(async move {
            while let Some(raw) = consensus_in.recv().await {
                if let Err(e) = p2p_in_tx.send(P2PInbound {
                    topic: "qorvum-consensus".to_string(),
                    data: raw,
                }).await {
                    warn!("[peer] p2p_in send failed: {}", e);
                }
            }
            warn!("[peer] consensus_in channel closed");
        });

        info!("[peer] P2P network running — waiting for peers via mDNS...");
        network_service.run().await;
        warn!("[peer] Network service exited");
    }
}