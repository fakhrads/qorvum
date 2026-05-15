//! # qorvum-crypto
//! Post-quantum cryptographic primitives for the Qorvum blockchain.
//!
//! ## Algorithms
//! - **Signing**:  ML-DSA (Dilithium3) — NIST FIPS 204
//! - **KEM**:      ML-KEM (Kyber768)   — NIST FIPS 203
//! - **Hashing**:  BLAKE3              — quantum-safe (256-bit output)

pub mod hash;
pub mod signing;
pub mod kem;
pub mod error;

pub use error::CryptoError;
pub use hash::{hash, hash_many, Hasher as QHasher};
pub use signing::{PQKeypair, SigningAlgorithm, Signature, PublicKey};
