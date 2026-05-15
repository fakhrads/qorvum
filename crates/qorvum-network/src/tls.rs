use crate::handshake::{
    perform_client_handshake, perform_server_handshake, HandshakeError, QorvumTlsSession,
};
use qorvum_msp::{Identity, IdentityVerifier};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

pub struct QorvumTlsListener {
    listener: TcpListener,
    identity: Arc<Identity>,
    verifier: Arc<IdentityVerifier>,
}

pub struct QorvumTlsConnector {
    identity: Arc<Identity>,
    verifier: Arc<IdentityVerifier>,
}

impl QorvumTlsListener {
    pub async fn bind(
        addr: &str,
        identity: Arc<Identity>,
        verifier: Arc<IdentityVerifier>,
    ) -> Result<Self, HandshakeError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener, identity, verifier })
    }

    pub async fn accept(&self) -> Result<QorvumTlsSession, HandshakeError> {
        let (stream, _peer_addr) = self.listener.accept().await?;
        perform_server_handshake(stream, Some(&self.identity), Some(&self.verifier)).await
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }
}

impl QorvumTlsConnector {
    pub fn new(identity: Arc<Identity>, verifier: Arc<IdentityVerifier>) -> Self {
        Self { identity, verifier }
    }

    pub async fn connect(&self, addr: &str) -> Result<QorvumTlsSession, HandshakeError> {
        let stream = TcpStream::connect(addr).await?;
        perform_client_handshake(stream, Some(&self.identity), Some(&self.verifier)).await
    }
}
