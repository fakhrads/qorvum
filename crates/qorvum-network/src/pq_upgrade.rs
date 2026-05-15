//! Custom libp2p security upgrade: `/qorvum/pq-tls/1.0.0`
//!
//! Wire protocol after multistream-select negotiation:
//!
//!   1. Both sides exchange their libp2p ed25519 public key as a length-prefixed
//!      protobuf blob (same encoding as libp2p noise/tls) so `PeerId` can be
//!      derived and the swarm keeps stable peer identity.
//!   2. The full PQ handshake runs on top:
//!         ClientHello → ServerHello → ClientFinish → ServerAck
//!      (Kyber-1024 KEM + X25519 + Dilithium3 signatures, AES-256-GCM sessions)
//!
//! ## AsyncRead/AsyncWrite bridge
//!
//! libp2p passes a `Negotiated<TcpStream>` which implements `futures::AsyncRead
//! + futures::AsyncWrite`.  Our handshake layer and `QorvumTlsSession` use
//! `tokio::io::AsyncRead + AsyncWrite`.  We bridge via `tokio_util::compat`:
//!
//!   futures stream  →  `Compat<C>` (tokio) → handshake → `QorvumTlsSession<Compat<C>>`
//!   session (tokio) →  `.compat()`          → `Compat<Session>` (futures) → yamux

use std::sync::Arc;

use futures::{AsyncRead as FuturesRead, AsyncWrite as FuturesWrite};
use futures::future::BoxFuture;
use libp2p::{core::upgrade::UpgradeInfo, identity, PeerId};
use qorvum_msp::{Identity, IdentityVerifier};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::debug;

use crate::handshake::{
    perform_client_handshake, perform_server_handshake, HandshakeError, QorvumTlsSession,
};

// ── Protocol id ───────────────────────────────────────────────────────────────

pub const PQ_PROTOCOL: &str = "/qorvum/pq-tls/1.0.0";

// ── Shorthand for the output stream type returned to yamux ───────────────────

/// The type returned to yamux after a successful PQ upgrade.
/// `C` is the raw libp2p stream (`Negotiated<TcpStream>`).
pub type PqStream<C> = Compat<QorvumTlsSession<Compat<C>>>;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct QorvumPqConfig {
    pub keypair:  identity::Keypair,
    pub identity: Option<Arc<Identity>>,
    pub verifier: Option<Arc<IdentityVerifier>>,
}

impl QorvumPqConfig {
    pub fn new(
        keypair:  identity::Keypair,
        identity: Option<Arc<Identity>>,
        verifier: Option<Arc<IdentityVerifier>>,
    ) -> Self {
        Self { keypair, identity, verifier }
    }
}

// ── UpgradeInfo ───────────────────────────────────────────────────────────────

impl UpgradeInfo for QorvumPqConfig {
    type Info     = &'static str;
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        std::iter::once(PQ_PROTOCOL)
    }
}

// ── PeerId helpers ────────────────────────────────────────────────────────────

fn encode_pubkey(kp: &identity::Keypair) -> Vec<u8> {
    kp.public().encode_protobuf()
}

/// Exchange libp2p public keys over a tokio-compat stream and derive remote PeerId.
async fn exchange_and_derive_peer_id<T>(
    stream:   &mut T,
    local_pk: &[u8],
) -> Result<PeerId, HandshakeError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    // Send our pubkey (u16 length prefix)
    let len = local_pk.len() as u16;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(local_pk).await?;

    // Receive peer pubkey
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf).await?;
    let remote_len = u16::from_be_bytes(len_buf) as usize;
    if remote_len > 4096 {
        return Err(HandshakeError::PeerCertInvalid("pubkey blob too large".into()));
    }
    let mut remote_pk_bytes = vec![0u8; remote_len];
    stream.read_exact(&mut remote_pk_bytes).await?;

    let remote_pk = identity::PublicKey::try_decode_protobuf(&remote_pk_bytes)
        .map_err(|e| HandshakeError::PeerCertInvalid(e.to_string()))?;

    Ok(PeerId::from_public_key(&remote_pk))
}

/// Derive a deterministic PeerId from arbitrary bytes (fallback for anonymous peers).
#[allow(dead_code)]
pub fn peer_id_from_bytes(raw: &[u8]) -> PeerId {
    use libp2p::multihash::Multihash;
    let digest = Sha256::digest(raw);
    let mh = Multihash::<64>::wrap(0x12, &digest)
        .expect("sha2-256 multihash always valid");
    PeerId::from_multihash(mh).expect("valid multihash always yields PeerId")
}

// ── Inbound upgrade (server side) ─────────────────────────────────────────────

impl<C> libp2p::core::upgrade::InboundConnectionUpgrade<C> for QorvumPqConfig
where
    C: FuturesRead + FuturesWrite + Unpin + Send + 'static,
{
    type Output = (PeerId, PqStream<C>);
    type Error  = HandshakeError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: C, _info: Self::Info) -> Self::Future {
        Box::pin(async move {
            // Bridge futures::AsyncRead → tokio::io::AsyncRead
            let mut tokio_socket = socket.compat();

            let local_pk       = encode_pubkey(&self.keypair);
            let remote_peer_id =
                exchange_and_derive_peer_id(&mut tokio_socket, &local_pk).await?;

            debug!(peer = %remote_peer_id, "PQ-TLS server: PeerId exchange done");

            let session = perform_server_handshake(
                tokio_socket,
                self.identity.as_deref(),
                self.verifier.as_deref(),
            )
            .await?;

            // Bridge tokio::io::AsyncRead → futures::AsyncRead for yamux
            Ok((remote_peer_id, session.compat()))
        })
    }
}

// ── Outbound upgrade (client side) ────────────────────────────────────────────

impl<C> libp2p::core::upgrade::OutboundConnectionUpgrade<C> for QorvumPqConfig
where
    C: FuturesRead + FuturesWrite + Unpin + Send + 'static,
{
    type Output = (PeerId, PqStream<C>);
    type Error  = HandshakeError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, socket: C, _info: Self::Info) -> Self::Future {
        Box::pin(async move {
            let mut tokio_socket = socket.compat();

            let local_pk       = encode_pubkey(&self.keypair);
            let remote_peer_id =
                exchange_and_derive_peer_id(&mut tokio_socket, &local_pk).await?;

            debug!(peer = %remote_peer_id, "PQ-TLS client: PeerId exchange done");

            let session = perform_client_handshake(
                tokio_socket,
                self.identity.as_deref(),
                self.verifier.as_deref(),
            )
            .await?;

            Ok((remote_peer_id, session.compat()))
        })
    }
}
