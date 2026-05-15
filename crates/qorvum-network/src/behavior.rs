use libp2p::{
    gossipsub, mdns, swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub struct QorvumBehavior {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}
