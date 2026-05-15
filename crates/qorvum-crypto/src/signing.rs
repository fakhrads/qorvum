//! Post-quantum digital signatures.
//! Default: Dilithium3 (ML-DSA, NIST FIPS 204)
//! Alternative: Falcon-512 (FN-DSA)

use crate::error::CryptoError;
use crate::hash::hash;
use pqcrypto_traits::sign::{PublicKey as PkTrait, SecretKey as SkTrait, SignedMessage};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SigningAlgorithm {
    Dilithium3,
    Falcon512,
}
impl Default for SigningAlgorithm {
    fn default() -> Self { Self::Dilithium3 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub algorithm: SigningAlgorithm,
    pub bytes:     Vec<u8>,
}
impl PublicKey {
    pub fn to_hex(&self) -> String { hex::encode(&self.bytes) }
    pub fn fingerprint(&self) -> [u8; 8] {
        let h = hash(&self.bytes);
        h[..8].try_into().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: SigningAlgorithm,
    pub bytes:     Vec<u8>,
}

/// Keypair — secret key bytes are zeroed on drop
pub struct PQKeypair {
    pub algorithm:  SigningAlgorithm,
    pub public_key: PublicKey,
    secret_bytes:   Vec<u8>,
}

impl Drop for PQKeypair {
    fn drop(&mut self) {
        self.secret_bytes.zeroize();
    }
}

impl PQKeypair {
    pub fn generate(algorithm: SigningAlgorithm) -> Result<Self, CryptoError> {
        match algorithm {
            SigningAlgorithm::Dilithium3 => {
                use pqcrypto_dilithium::dilithium3;
                let (pk, sk) = dilithium3::keypair();
                Ok(Self {
                    algorithm,
                    public_key: PublicKey { algorithm, bytes: pk.as_bytes().to_vec() },
                    secret_bytes: sk.as_bytes().to_vec(),
                })
            }
            SigningAlgorithm::Falcon512 => {
                use pqcrypto_falcon::falcon512;
                let (pk, sk) = falcon512::keypair();
                Ok(Self {
                    algorithm,
                    public_key: PublicKey { algorithm, bytes: pk.as_bytes().to_vec() },
                    secret_bytes: sk.as_bytes().to_vec(),
                })
            }
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature, CryptoError> {
        let digest = hash(message);
        match self.algorithm {
            SigningAlgorithm::Dilithium3 => {
                use pqcrypto_dilithium::dilithium3;
                let sk = dilithium3::SecretKey::from_bytes(&self.secret_bytes)
                    .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;
                let signed = dilithium3::sign(&digest, &sk);
                Ok(Signature { algorithm: self.algorithm, bytes: signed.as_bytes().to_vec() })
            }
            SigningAlgorithm::Falcon512 => {
                use pqcrypto_falcon::falcon512;
                let sk = falcon512::SecretKey::from_bytes(&self.secret_bytes)
                    .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;
                let signed = falcon512::sign(&digest, &sk);
                Ok(Signature { algorithm: self.algorithm, bytes: signed.as_bytes().to_vec() })
            }
        }
    }

    pub fn public_key(&self) -> &PublicKey { &self.public_key }

    /// Expose secret bytes for serialization (e.g., persisting keypair to disk).
    pub fn secret_bytes(&self) -> Vec<u8> { self.secret_bytes.clone() }

    /// Reconstruct a keypair from previously serialized bytes.
    pub fn from_bytes(algorithm: SigningAlgorithm, pk_bytes: Vec<u8>, sk_bytes: Vec<u8>) -> Self {
        Self {
            algorithm,
            public_key: PublicKey { algorithm, bytes: pk_bytes },
            secret_bytes: sk_bytes,
        }
    }
}

pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
    if public_key.algorithm != signature.algorithm { return false; }
    let digest = hash(message);
    match signature.algorithm {
        SigningAlgorithm::Dilithium3 => {
            use pqcrypto_dilithium::dilithium3;
            use pqcrypto_traits::sign::PublicKey as Pk;
            let pk = match dilithium3::PublicKey::from_bytes(&public_key.bytes) {
                Ok(pk) => pk, Err(_) => return false,
            };
            let sm = match dilithium3::SignedMessage::from_bytes(&signature.bytes) {
                Ok(sm) => sm, Err(_) => return false,
            };
            dilithium3::open(&sm, &pk)
                .map(|msg| {
                    use subtle::ConstantTimeEq;
                    bool::from(msg.as_slice().ct_eq(digest.as_slice()))
                })
                .unwrap_or(false)
        }
        SigningAlgorithm::Falcon512 => {
            use pqcrypto_falcon::falcon512;
            use pqcrypto_traits::sign::PublicKey as Pk;
            let pk = match falcon512::PublicKey::from_bytes(&public_key.bytes) {
                Ok(pk) => pk, Err(_) => return false,
            };
            let sm = match falcon512::SignedMessage::from_bytes(&signature.bytes) {
                Ok(sm) => sm, Err(_) => return false,
            };
            falcon512::open(&sm, &pk)
                .map(|msg| {
                    use subtle::ConstantTimeEq;
                    bool::from(msg.as_slice().ct_eq(digest.as_slice()))
                })
                .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dilithium3_roundtrip() {
        let kp  = PQKeypair::generate(SigningAlgorithm::Dilithium3).unwrap();
        let msg = b"Qorvum Enterprise Blockchain";
        let sig = kp.sign(msg).unwrap();
        assert!(verify(&kp.public_key, msg, &sig));
        assert!(!verify(&kp.public_key, b"tampered", &sig));
    }
    #[test]
    fn test_falcon512_roundtrip() {
        let kp  = PQKeypair::generate(SigningAlgorithm::Falcon512).unwrap();
        let msg = b"Qorvum test";
        let sig = kp.sign(msg).unwrap();
        assert!(verify(&kp.public_key, msg, &sig));
    }
}
