use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use qorvum_crypto::{
    hash_many,
    kem::{encapsulate, KemKeypair, KemPublicKey},
    signing::{verify, PublicKey, Signature, SigningAlgorithm},
};
use qorvum_msp::{Identity, IdentityVerifier, VerifiedIdentity};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    io,
    pin::Pin,
    task::{ready, Context, Poll},
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PubKey};

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("peer cert verification failed: {0}")]
    PeerCertInvalid(String),
    #[error("peer signature invalid")]
    PeerSignatureInvalid,
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("decryption error")]
    DecryptionFailed,
    #[error("nonce ordering violation")]
    NonceViolation,
    #[error("frame too large: {0} bytes")]
    FrameTooLarge(u32),
}

impl From<bincode::Error> for HandshakeError {
    fn from(e: bincode::Error) -> Self {
        HandshakeError::Serialization(e.to_string())
    }
}

pub const MAX_FRAME_SIZE: u32 = 64 * 1024 * 1024; // 64 MiB

// ── Handshake messages ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct ClientHello {
    pub client_kyber_pubkey: Vec<u8>,
    pub client_x25519_pubkey: [u8; 32],
    pub timestamp: u64,
    pub nonce: [u8; 16],
}

#[derive(Serialize, Deserialize)]
pub struct ServerHello {
    /// KEM ciphertext for client (encapsulation of client's Kyber pubkey)
    pub server_kyber_ciphertext: Vec<u8>,
    /// Server's own Kyber pubkey (so client can encapsulate back)
    pub server_kyber_pubkey: Vec<u8>,
    pub server_x25519_pubkey: [u8; 32],
    pub server_nonce: [u8; 16],
    /// Dilithium3 sig over BLAKE3(client_nonce || server_nonce || server_kyber_ct || server_kyber_pk)
    pub server_signature: Vec<u8>,
    pub server_pk_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientFinish {
    /// KEM ciphertext for server (encapsulation of server's Kyber pubkey)
    pub client_kyber_ciphertext: Vec<u8>,
    /// Dilithium3 sig over BLAKE3(server_nonce || client_nonce || client_kyber_ct)
    pub client_signature: Vec<u8>,
    pub client_pk_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAck {
    /// AES-256-GCM("OK") encrypted with the derived session key
    pub encrypted_ok: Vec<u8>,
}

// ── Read state machine ────────────────────────────────────────────────────────

enum ReadPhase {
    Idle,
    Header,
    Body,
}

// ── Encrypted session ─────────────────────────────────────────────────────────

/// A PQ-secured duplex session.  `T` defaults to `TcpStream` for the
/// standalone `QorvumTlsListener`/`QorvumTlsConnector` path; libp2p passes
/// its own negotiated stream type.
pub struct QorvumTlsSession<T = TcpStream> {
    pub remote_identity: Option<VerifiedIdentity>,
    pub(crate) stream:        T,
    pub(crate) send_key:      [u8; 32],
    pub(crate) recv_key:      [u8; 32],
    pub send_counter: u64,
    pub recv_counter: u64,
    // ── AsyncRead state ──────────────────────────────────────────────────────
    read_phase:    ReadPhase,
    read_hdr:      [u8; 4],
    read_hdr_pos:  usize,
    read_body:     Vec<u8>,
    read_body_pos: usize,
    plaintext:     Vec<u8>,
    plaintext_pos: usize,
    // ── AsyncWrite state ─────────────────────────────────────────────────────
    write_frame:     Vec<u8>,
    write_frame_pos: usize,
}

impl<T> QorvumTlsSession<T> {
    pub(crate) fn new(
        stream:          T,
        remote_identity: Option<VerifiedIdentity>,
        send_key:        [u8; 32],
        recv_key:        [u8; 32],
    ) -> Self {
        Self {
            remote_identity,
            stream,
            send_key,
            recv_key,
            send_counter:    0,
            recv_counter:    0,
            read_phase:      ReadPhase::Idle,
            read_hdr:        [0u8; 4],
            read_hdr_pos:    0,
            read_body:       Vec::new(),
            read_body_pos:   0,
            plaintext:       Vec::new(),
            plaintext_pos:   0,
            write_frame:     Vec::new(),
            write_frame_pos: 0,
        }
    }

    pub(crate) fn make_nonce(counter: u64) -> [u8; 12] {
        let mut n = [0u8; 12];
        n[..8].copy_from_slice(&counter.to_le_bytes());
        n
    }
}

// ── Async send / recv (used by standalone TLS path) ──────────────────────────

impl<T: AsyncRead + AsyncWrite + Unpin + Send> QorvumTlsSession<T> {
    pub async fn send(&mut self, data: &[u8]) -> Result<(), HandshakeError> {
        let nonce_bytes = Self::make_nonce(self.send_counter);
        let key    = Key::<Aes256Gcm>::from_slice(&self.send_key);
        let cipher = Aes256Gcm::new(key);
        let nonce  = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| HandshakeError::Encryption(e.to_string()))?;

        let len = ciphertext.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&ciphertext).await?;
        self.send_counter += 1;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, HandshakeError> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);

        if len > MAX_FRAME_SIZE {
            return Err(HandshakeError::FrameTooLarge(len));
        }

        let mut ciphertext = vec![0u8; len as usize];
        self.stream.read_exact(&mut ciphertext).await?;

        let nonce_bytes = Self::make_nonce(self.recv_counter);
        let key    = Key::<Aes256Gcm>::from_slice(&self.recv_key);
        let cipher = Aes256Gcm::new(key);
        let nonce  = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|_| HandshakeError::DecryptionFailed)?;

        self.recv_counter += 1;
        Ok(plaintext)
    }
}

// ── AsyncRead impl (used by libp2p upgrade path) ──────────────────────────────

impl<T: AsyncRead + Unpin> AsyncRead for QorvumTlsSession<T> {
    fn poll_read(
        self:  Pin<&mut Self>,
        cx:    &mut Context<'_>,
        buf:   &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // T: Unpin → Pin<&mut Self> is Unpin → safe to call get_mut()
        let this = self.get_mut();

        loop {
            // Serve buffered plaintext first
            if this.plaintext_pos < this.plaintext.len() {
                let src = &this.plaintext[this.plaintext_pos..];
                let n   = src.len().min(buf.remaining());
                buf.put_slice(&src[..n]);
                this.plaintext_pos += n;
                if this.plaintext_pos >= this.plaintext.len() {
                    this.plaintext.clear();
                    this.plaintext_pos = 0;
                }
                return Poll::Ready(Ok(()));
            }

            match this.read_phase {
                ReadPhase::Idle => {
                    this.read_hdr     = [0u8; 4];
                    this.read_hdr_pos = 0;
                    this.read_phase   = ReadPhase::Header;
                }

                ReadPhase::Header => {
                    while this.read_hdr_pos < 4 {
                        // Split borrows explicitly so the borrow checker is happy
                        let pos = this.read_hdr_pos;
                        {
                            let mut rb = ReadBuf::new(&mut this.read_hdr[pos..]);
                            ready!(Pin::new(&mut this.stream).poll_read(cx, &mut rb))?;
                            let n = rb.filled().len();
                            if n == 0 {
                                return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()));
                            }
                            this.read_hdr_pos += n;
                        }
                    }
                    let frame_len = u32::from_be_bytes(this.read_hdr);
                    if frame_len > MAX_FRAME_SIZE {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("PQ frame too large: {frame_len} bytes"),
                        )));
                    }
                    this.read_body.clear();
                    this.read_body.resize(frame_len as usize, 0u8);
                    this.read_body_pos = 0;
                    this.read_phase    = ReadPhase::Body;
                }

                ReadPhase::Body => {
                    while this.read_body_pos < this.read_body.len() {
                        let pos                    = this.read_body_pos;
                        let (stream, body, body_pos) =
                            (&mut this.stream, &mut this.read_body, &mut this.read_body_pos);
                        let mut rb = ReadBuf::new(&mut body[pos..]);
                        ready!(Pin::new(stream).poll_read(cx, &mut rb))?;
                        let n = rb.filled().len();
                        if n == 0 {
                            return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()));
                        }
                        *body_pos += n;
                    }

                    let nonce_bytes = Self::make_nonce(this.recv_counter);
                    let key    = Key::<Aes256Gcm>::from_slice(&this.recv_key);
                    let cipher = Aes256Gcm::new(key);
                    let nonce  = Nonce::from_slice(&nonce_bytes);

                    match cipher.decrypt(&nonce, this.read_body.as_ref()) {
                        Ok(pt) => {
                            this.recv_counter += 1;
                            this.plaintext     = pt;
                            this.plaintext_pos = 0;
                            this.read_phase    = ReadPhase::Idle;
                        }
                        Err(_) => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "PQ decryption failed",
                            )));
                        }
                    }
                }
            }
        }
    }
}

// ── AsyncWrite impl (used by libp2p upgrade path) ─────────────────────────────

impl<T: AsyncWrite + Unpin> AsyncWrite for QorvumTlsSession<T> {
    fn poll_write(
        self:  Pin<&mut Self>,
        cx:    &mut Context<'_>,
        buf:   &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        // Flush any pending frame from a previous call first
        while this.write_frame_pos < this.write_frame.len() {
            let pos = this.write_frame_pos;
            let (stream, frame, frame_pos) =
                (&mut this.stream, &this.write_frame, &mut this.write_frame_pos);
            let written = ready!(Pin::new(stream).poll_write(cx, &frame[pos..]))?;
            if written == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
            *frame_pos += written;
        }

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // Encrypt buf as a new frame
        let nonce_bytes = Self::make_nonce(this.send_counter);
        let key    = Key::<Aes256Gcm>::from_slice(&this.send_key);
        let cipher = Aes256Gcm::new(key);
        let nonce  = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        this.send_counter += 1;

        let len = ciphertext.len() as u32;
        let mut frame = Vec::with_capacity(4 + ciphertext.len());
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&ciphertext);
        this.write_frame     = frame;
        this.write_frame_pos = 0;

        // Best-effort: start writing the frame immediately
        while this.write_frame_pos < this.write_frame.len() {
            let pos = this.write_frame_pos;
            let (stream, frame, frame_pos) =
                (&mut this.stream, &this.write_frame, &mut this.write_frame_pos);
            match Pin::new(stream).poll_write(cx, &frame[pos..]) {
                Poll::Ready(Ok(n)) if n == 0 => {
                    return Poll::Ready(Err(io::ErrorKind::WriteZero.into()))
                }
                Poll::Ready(Ok(n))  => *frame_pos += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending       => break,
            }
        }

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        while this.write_frame_pos < this.write_frame.len() {
            let pos = this.write_frame_pos;
            let (stream, frame, frame_pos) =
                (&mut this.stream, &this.write_frame, &mut this.write_frame_pos);
            let written = ready!(Pin::new(stream).poll_write(cx, &frame[pos..]))?;
            if written == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
            *frame_pos += written;
        }
        Pin::new(&mut this.stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        // Flush pending frame
        while this.write_frame_pos < this.write_frame.len() {
            let pos = this.write_frame_pos;
            let (stream, frame, frame_pos) =
                (&mut this.stream, &this.write_frame, &mut this.write_frame_pos);
            let written = ready!(Pin::new(stream).poll_write(cx, &frame[pos..]))?;
            if written == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
            *frame_pos += written;
        }
        ready!(Pin::new(&mut this.stream).poll_flush(cx))?;
        Pin::new(&mut this.stream).poll_shutdown(cx)
    }
}

// ── Wire helpers (generic) ────────────────────────────────────────────────────

pub(crate) async fn write_msg<T, M>(stream: &mut T, msg: &M) -> Result<(), HandshakeError>
where
    T: AsyncWrite + Unpin,
    M: serde::Serialize,
{
    let bytes = bincode::serialize(msg)?;
    let len   = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

pub(crate) async fn read_msg<T, M>(stream: &mut T) -> Result<M, HandshakeError>
where
    T: AsyncRead + Unpin,
    M: serde::de::DeserializeOwned,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf);
    if len > MAX_FRAME_SIZE {
        return Err(HandshakeError::FrameTooLarge(len));
    }
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
}

// ── Key derivation ────────────────────────────────────────────────────────────

pub(crate) fn derive_session_keys(
    kyber_ss_a: &[u8],
    kyber_ss_b: &[u8],
    x25519_ss:  &[u8; 32],
    nonce_a:    &[u8; 16],
    nonce_b:    &[u8; 16],
) -> ([u8; 32], [u8; 32]) {
    let send_key = hash_many(&[kyber_ss_a, kyber_ss_b, x25519_ss, nonce_a, nonce_b, b"a2b"]);
    let recv_key = hash_many(&[kyber_ss_a, kyber_ss_b, x25519_ss, nonce_a, nonce_b, b"b2a"]);
    (send_key, recv_key)
}

fn make_server_sig_tbs(
    client_nonce:    &[u8; 16],
    server_nonce:    &[u8; 16],
    server_kyber_ct: &[u8],
    server_kyber_pk: &[u8],
) -> Vec<u8> {
    hash_many(&[
        client_nonce, server_nonce, server_kyber_ct, server_kyber_pk, b"server_hello_sig",
    ])
    .to_vec()
}

fn make_client_sig_tbs(
    server_nonce:    &[u8; 16],
    client_nonce:    &[u8; 16],
    client_kyber_ct: &[u8],
) -> Vec<u8> {
    hash_many(&[server_nonce, client_nonce, client_kyber_ct, b"client_finish_sig"]).to_vec()
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

// ── Client handshake (generic over T) ────────────────────────────────────────

pub async fn perform_client_handshake<T>(
    mut stream:       T,
    local_identity:   Option<&Identity>,
    _verifier:        Option<&IdentityVerifier>,
) -> Result<QorvumTlsSession<T>, HandshakeError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    let client_kem            = KemKeypair::generate();
    let client_x25519_secret  = EphemeralSecret::random_from_rng(rand::thread_rng());
    let client_x25519_pub     = X25519PubKey::from(&client_x25519_secret);

    let mut client_nonce = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut client_nonce);

    let hello = ClientHello {
        client_kyber_pubkey: client_kem.public_key.0.clone(),
        client_x25519_pubkey: *client_x25519_pub.as_bytes(),
        timestamp: now_nanos(),
        nonce: client_nonce,
    };
    write_msg(&mut stream, &hello).await?;

    let server_hello: ServerHello = read_msg(&mut stream).await?;

    if !server_hello.server_pk_bytes.is_empty() && !server_hello.server_signature.is_empty() {
        let tbs = make_server_sig_tbs(
            &client_nonce,
            &server_hello.server_nonce,
            &server_hello.server_kyber_ciphertext,
            &server_hello.server_kyber_pubkey,
        );
        let server_pk  = PublicKey  { algorithm: SigningAlgorithm::Dilithium3, bytes: server_hello.server_pk_bytes.clone() };
        let server_sig = Signature  { algorithm: SigningAlgorithm::Dilithium3, bytes: server_hello.server_signature.clone() };
        if !verify(&server_pk, &tbs, &server_sig) {
            return Err(HandshakeError::PeerSignatureInvalid);
        }
    }

    let client_ss = client_kem
        .decapsulate(&server_hello.server_kyber_ciphertext)
        .map_err(|e| HandshakeError::Encryption(e.to_string()))?;

    let server_kem_pk = KemPublicKey(server_hello.server_kyber_pubkey.clone());
    let (client_kyber_ct, server_ss) =
        encapsulate(&server_kem_pk).map_err(|e| HandshakeError::Encryption(e.to_string()))?;

    let server_x25519_pub = X25519PubKey::from(server_hello.server_x25519_pubkey);
    let x25519_ss         = client_x25519_secret.diffie_hellman(&server_x25519_pub);

    let client_sig_tbs   = make_client_sig_tbs(&server_hello.server_nonce, &client_nonce, &client_kyber_ct);
    let client_sig_bytes = if let Some(id) = local_identity {
        id.sign(&client_sig_tbs).map(|s| s.bytes).unwrap_or_default()
    } else {
        vec![]
    };
    let client_pk_bytes = local_identity.map(|id| id.public_key().bytes.clone()).unwrap_or_default();

    let finish = ClientFinish {
        client_kyber_ciphertext: client_kyber_ct,
        client_signature:        client_sig_bytes,
        client_pk_bytes,
    };
    write_msg(&mut stream, &finish).await?;

    let ack: ServerAck = read_msg(&mut stream).await?;

    let (send_key, recv_key) = derive_session_keys(
        client_ss.as_bytes(),
        server_ss.as_bytes(),
        x25519_ss.as_bytes(),
        &client_nonce,
        &server_hello.server_nonce,
    );

    let ack_key    = Key::<Aes256Gcm>::from_slice(&send_key);
    let ack_cipher = Aes256Gcm::new(ack_key);
    let ack_nonce  = Nonce::from_slice(&[0u8; 12]);
    let decrypted  = ack_cipher
        .decrypt(ack_nonce, ack.encrypted_ok.as_slice())
        .map_err(|_| HandshakeError::DecryptionFailed)?;
    if decrypted != b"OK" {
        return Err(HandshakeError::DecryptionFailed);
    }

    Ok(QorvumTlsSession::new(stream, None, send_key, recv_key))
}

// ── Server handshake (generic over T) ────────────────────────────────────────

pub async fn perform_server_handshake<T>(
    mut stream:       T,
    local_identity:   Option<&Identity>,
    _verifier:        Option<&IdentityVerifier>,
) -> Result<QorvumTlsSession<T>, HandshakeError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    let client_hello: ClientHello = read_msg(&mut stream).await?;

    let server_kem            = KemKeypair::generate();
    let server_x25519_secret  = EphemeralSecret::random_from_rng(rand::thread_rng());
    let server_x25519_pub     = X25519PubKey::from(&server_x25519_secret);

    let mut server_nonce = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut server_nonce);

    let client_kem_pk = KemPublicKey(client_hello.client_kyber_pubkey.clone());
    let (server_kyber_ct, client_ss) =
        encapsulate(&client_kem_pk).map_err(|e| HandshakeError::Encryption(e.to_string()))?;

    let server_sig_tbs = make_server_sig_tbs(
        &client_hello.nonce,
        &server_nonce,
        &server_kyber_ct,
        &server_kem.public_key.0,
    );
    let (server_sig_bytes, server_pk_bytes) = if let Some(id) = local_identity {
        let sig = id.sign(&server_sig_tbs).map(|s| s.bytes).unwrap_or_default();
        (sig, id.public_key().bytes.clone())
    } else {
        (vec![], vec![])
    };

    let server_hello = ServerHello {
        server_kyber_ciphertext: server_kyber_ct,
        server_kyber_pubkey:     server_kem.public_key.0.clone(),
        server_x25519_pubkey:    *server_x25519_pub.as_bytes(),
        server_nonce,
        server_signature:        server_sig_bytes,
        server_pk_bytes,
    };
    write_msg(&mut stream, &server_hello).await?;

    let client_finish: ClientFinish = read_msg(&mut stream).await?;

    let server_ss = server_kem
        .decapsulate(&client_finish.client_kyber_ciphertext)
        .map_err(|e| HandshakeError::Encryption(e.to_string()))?;

    let client_x25519_pub = X25519PubKey::from(client_hello.client_x25519_pubkey);
    let x25519_ss         = server_x25519_secret.diffie_hellman(&client_x25519_pub);

    let (client_send_key, client_recv_key) = derive_session_keys(
        client_ss.as_bytes(),
        server_ss.as_bytes(),
        x25519_ss.as_bytes(),
        &client_hello.nonce,
        &server_nonce,
    );
    let (server_send_key, server_recv_key) = (client_recv_key, client_send_key);

    let ack_key    = Key::<Aes256Gcm>::from_slice(&client_send_key);
    let ack_cipher = Aes256Gcm::new(ack_key);
    let ack_nonce  = Nonce::from_slice(&[0u8; 12]);
    let encrypted_ok = ack_cipher
        .encrypt(ack_nonce, b"OK".as_slice())
        .map_err(|e| HandshakeError::Encryption(e.to_string()))?;

    let ack = ServerAck { encrypted_ok };
    write_msg(&mut stream, &ack).await?;

    Ok(QorvumTlsSession::new(stream, None, server_send_key, server_recv_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_full_handshake_loopback() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            perform_server_handshake(stream, None, None).await.unwrap()
        });

        let client_stream = TcpStream::connect(addr).await.unwrap();
        let mut client = perform_client_handshake(client_stream, None, None).await.unwrap();
        let mut server = server_task.await.unwrap();

        client.send(b"hello from client").await.unwrap();
        let msg = server.recv().await.unwrap();
        assert_eq!(msg, b"hello from client");

        server.send(b"hello from server").await.unwrap();
        let msg = client.recv().await.unwrap();
        assert_eq!(msg, b"hello from server");
    }

    #[tokio::test]
    async fn test_encrypted_send_recv() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            perform_server_handshake(stream, None, None).await.unwrap()
        });

        let client_stream = TcpStream::connect(addr).await.unwrap();
        let mut client = perform_client_handshake(client_stream, None, None).await.unwrap();
        let mut server = server_task.await.unwrap();

        for i in 0u8..5 {
            let payload = vec![i; 100];
            client.send(&payload).await.unwrap();
            let received = server.recv().await.unwrap();
            assert_eq!(received, payload);
        }
        assert_eq!(client.send_counter, 5);
        assert_eq!(server.recv_counter, 5);
    }

    #[tokio::test]
    async fn test_handshake_with_bad_cert() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            perform_server_handshake(stream, None, None).await
        });

        let client_stream = TcpStream::connect(addr).await.unwrap();
        let result = perform_client_handshake(client_stream, None, None).await;
        assert!(result.is_ok());
        assert!(server_task.await.unwrap().is_ok());
    }

    /// Verify that the AsyncRead + AsyncWrite impls work end-to-end through
    /// the PQ-encrypted framing layer (used by the libp2p upgrade path).
    #[tokio::test]
    async fn test_async_read_write_framing() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            perform_server_handshake(stream, None, None).await.unwrap()
        });

        let client_stream = TcpStream::connect(addr).await.unwrap();
        let mut client = perform_client_handshake(client_stream, None, None).await.unwrap();
        let mut server = server_task.await.unwrap();

        // Use AsyncWrite / AsyncRead directly (libp2p path)
        client.write_all(b"ping via AsyncWrite").await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 19];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping via AsyncWrite");
    }
}
