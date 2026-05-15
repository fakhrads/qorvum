pub mod behavior;
pub mod handshake;
pub mod pq_upgrade;
pub mod tls;

pub use handshake::{perform_client_handshake, perform_server_handshake, QorvumTlsSession};
pub use pq_upgrade::QorvumPqConfig;
pub use tls::{QorvumTlsConnector, QorvumTlsListener};

use anyhow::Result;
use behavior::QorvumBehavior;
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, mdns, swarm::SwarmEvent, tcp, yamux, Multiaddr, PeerId, Swarm,
    SwarmBuilder,
};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use qorvum_msp::{Identity, IdentityVerifier};

// ── Peer info emitted when topology changes ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub addr:    String,
}

// ── Public handle returned to the caller ─────────────────────────────────────

pub struct NetworkHandles {
    pub tx_broadcast:  mpsc::Sender<Vec<u8>>,
    pub consensus_out: mpsc::Sender<Vec<u8>>,
    pub consensus_in:  mpsc::Receiver<Vec<u8>>,
    pub peer_events:   mpsc::Receiver<Vec<PeerInfo>>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct NetworkService {
    swarm:             Swarm<QorvumBehavior>,
    bootstrap_peers:   Vec<Multiaddr>,
    tx_rx:             mpsc::Receiver<Vec<u8>>,
    consensus_out_rx:  mpsc::Receiver<Vec<u8>>,
    consensus_in_tx:   mpsc::Sender<Vec<u8>>,
    peer_events_tx:    mpsc::Sender<Vec<PeerInfo>>,
}

impl NetworkService {
    /// Create a new network service.
    ///
    /// * `listen_addr`     – libp2p multiaddr to listen on, e.g. `/ip4/0.0.0.0/tcp/9000`
    /// * `bootstrap_peers` – static peers to dial at startup (multiaddr strings,
    ///                        may include `/p2p/<PeerId>` suffix)
    /// * `msp_identity`    – node's MSP identity for Dilithium3 auth in PQ handshake
    /// * `msp_verifier`    – CA verifier for checking remote MSP certificates
    pub fn new(
        listen_addr:    &str,
        bootstrap_peers: Vec<String>,
        msp_identity:   Option<Arc<Identity>>,
        msp_verifier:   Option<Arc<IdentityVerifier>>,
    ) -> Result<(Self, NetworkHandles)> {
        let local_key     = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {local_peer_id}");

        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub build: {}", e))?;

        let mdns     = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;
        let behavior = QorvumBehavior { gossipsub, mdns };

        // Build the PQ security upgrade — replaces noise::Config::new
        let pq_identity = msp_identity.clone();
        let pq_verifier = msp_verifier.clone();

        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                move |keypair: &identity::Keypair| -> Result<QorvumPqConfig, std::convert::Infallible> {
                    Ok(QorvumPqConfig::new(
                        keypair.clone(),
                        pq_identity.clone(),
                        pq_verifier.clone(),
                    ))
                },
                yamux::Config::default,
            )?
            .with_behaviour(|_| behavior)?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        swarm.listen_on(listen_addr.parse()?)?;

        // Parse bootstrap multiaddrs, warn and skip invalid ones
        let parsed_bootstrap: Vec<Multiaddr> = bootstrap_peers
            .into_iter()
            .filter_map(|s| match s.parse::<Multiaddr>() {
                Ok(a)  => Some(a),
                Err(e) => { warn!("Invalid bootstrap addr '{}': {}", s, e); None }
            })
            .collect();

        let (tx_tx, tx_rx)            = mpsc::channel(200);
        let (cons_out_tx, cons_out_rx) = mpsc::channel(200);
        let (cons_in_tx, cons_in_rx)   = mpsc::channel(200);
        let (peer_ev_tx, peer_ev_rx)   = mpsc::channel(32);

        let service = Self {
            swarm,
            bootstrap_peers: parsed_bootstrap,
            tx_rx,
            consensus_out_rx: cons_out_rx,
            consensus_in_tx:  cons_in_tx,
            peer_events_tx:   peer_ev_tx,
        };
        let handles = NetworkHandles {
            tx_broadcast:  tx_tx,
            consensus_out: cons_out_tx,
            consensus_in:  cons_in_rx,
            peer_events:   peer_ev_rx,
        };

        Ok((service, handles))
    }

    pub async fn run(mut self) {
        let tx_topic   = gossipsub::IdentTopic::new("qorvum-tx");
        let cons_topic = gossipsub::IdentTopic::new("qorvum-consensus");

        self.swarm.behaviour_mut().gossipsub.subscribe(&tx_topic).unwrap();
        self.swarm.behaviour_mut().gossipsub.subscribe(&cons_topic).unwrap();

        // Dial all bootstrap peers
        for addr in &self.bootstrap_peers {
            info!("Dialing bootstrap peer: {addr}");
            if let Err(e) = self.swarm.dial(addr.clone()) {
                warn!("Bootstrap dial failed for {addr}: {e}");
            }
        }

        let mut connected_peers: HashMap<PeerId, String> = HashMap::new();

        loop {
            tokio::select! {
                Some(msg) = self.tx_rx.recv() => {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(tx_topic.clone(), msg) {
                        error!("tx publish: {e:?}");
                    }
                }

                Some(msg) = self.consensus_out_rx.recv() => {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(cons_topic.clone(), msg) {
                        match &e {
                            gossipsub::PublishError::InsufficientPeers => {}
                            _ => error!("consensus publish: {e:?}"),
                        }
                    }
                }

                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(behavior::QorvumBehaviorEvent::Mdns(
                        mdns::Event::Discovered(list)
                    )) => {
                        let mut changed = false;
                        for (peer_id, addr) in list {
                            info!("mDNS discovered: {peer_id}");
                            connected_peers.insert(peer_id, addr.to_string());
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            changed = true;
                        }
                        if changed { self.emit_peers(&connected_peers); }
                    }

                    SwarmEvent::Behaviour(behavior::QorvumBehaviorEvent::Mdns(
                        mdns::Event::Expired(list)
                    )) => {
                        for (peer_id, _) in list {
                            info!("mDNS expired: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }

                    // Bootstrap peer connected — record it
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        let addr = endpoint.get_remote_address().to_string();
                        info!("Connected to peer {peer_id} at {addr}");
                        if !connected_peers.contains_key(&peer_id) {
                            connected_peers.insert(peer_id, addr);
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            self.emit_peers(&connected_peers);
                        }
                    }

                    SwarmEvent::ConnectionClosed { peer_id, num_established, .. } => {
                        if num_established == 0 && connected_peers.remove(&peer_id).is_some() {
                            info!("Peer disconnected: {peer_id}");
                            self.emit_peers(&connected_peers);
                        }
                    }

                    SwarmEvent::Behaviour(behavior::QorvumBehaviorEvent::Gossipsub(
                        gossipsub::Event::Message {
                            propagation_source: peer_id,
                            message_id: id,
                            message,
                        }
                    )) => {
                        if message.topic == tx_topic.hash() {
                            info!(
                                "tx msg '{}' (id={id}) from {peer_id}",
                                String::from_utf8_lossy(&message.data)
                            );
                        } else if message.topic == cons_topic.hash() {
                            if let Err(e) = self.consensus_in_tx.try_send(message.data) {
                                error!("consensus_in_tx full or closed: {e}");
                            }
                        }
                    }

                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        warn!("Outgoing connection error (peer={peer_id:?}): {error}");
                    }

                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {address}");
                    }

                    _ => {}
                }
            }
        }
    }

    fn emit_peers(&self, map: &HashMap<PeerId, String>) {
        let snapshot: Vec<PeerInfo> = map
            .iter()
            .map(|(id, a)| PeerInfo { peer_id: id.to_string(), addr: a.clone() })
            .collect();
        let _ = self.peer_events_tx.try_send(snapshot);
    }
}
