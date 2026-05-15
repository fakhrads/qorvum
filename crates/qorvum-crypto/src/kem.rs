//! ML-KEM (Kyber768) key encapsulation — NIST FIPS 203
//! Used for establishing shared secrets during peer TLS handshakes.

use crate::error::CryptoError;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{PublicKey as PkTrait, SecretKey as SkTrait,
                            SharedSecret as SsTrait, Ciphertext as CtTrait};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub struct KemKeypair {
    pub public_key: KemPublicKey,
    secret_bytes:  Vec<u8>,
}

#[derive(Clone)]
pub struct KemPublicKey(pub Vec<u8>);

#[derive(ZeroizeOnDrop)]
pub struct SharedSecret(Vec<u8>);

impl SharedSecret {
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
}

impl Zeroize for KemKeypair {
    fn zeroize(&mut self) { self.secret_bytes.zeroize(); }
}

impl KemKeypair {
    pub fn generate() -> Self {
        let (pk, sk) = kyber768::keypair();
        Self {
            public_key: KemPublicKey(pk.as_bytes().to_vec()),
            secret_bytes: sk.as_bytes().to_vec(),
        }
    }

    /// Decapsulate ciphertext → shared secret
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<SharedSecret, CryptoError> {
        let sk = kyber768::SecretKey::from_bytes(&self.secret_bytes)
            .map_err(|e| CryptoError::KemFailed(e.to_string()))?;
        let ct = kyber768::Ciphertext::from_bytes(ciphertext)
            .map_err(|e| CryptoError::KemFailed(e.to_string()))?;
        let ss = kyber768::decapsulate(&ct, &sk);
        Ok(SharedSecret(ss.as_bytes().to_vec()))
    }
}

/// Encapsulate to a public key → (ciphertext, shared_secret)
pub fn encapsulate(public_key: &KemPublicKey) -> Result<(Vec<u8>, SharedSecret), CryptoError> {
    let pk = kyber768::PublicKey::from_bytes(&public_key.0)
        .map_err(|e| CryptoError::KemFailed(e.to_string()))?;
    let (ss, ct) = kyber768::encapsulate(&pk);
    Ok((ct.as_bytes().to_vec(), SharedSecret(ss.as_bytes().to_vec())))
}
