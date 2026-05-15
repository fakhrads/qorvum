//! BLAKE3-based hashing — quantum-safe with 256-bit output.
//! Grover's algorithm requires 2^128 quantum ops to break 256-bit hash.

use blake3::Hasher as Blake3Hasher;

/// Hash a single byte slice → 32-byte digest
#[inline]
pub fn hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Hash multiple slices as one logical stream (no concatenation overhead)
#[inline]
pub fn hash_many(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Blake3Hasher::new();
    for part in parts {
        h.update(part);
    }
    *h.finalize().as_bytes()
}

/// Streaming hasher — useful for hashing large blocks incrementally
pub struct Hasher(Blake3Hasher);

impl Hasher {
    pub fn new() -> Self { Self(Blake3Hasher::new()) }
    pub fn update(&mut self, data: &[u8]) -> &mut Self { self.0.update(data); self }
    pub fn finalize(&self) -> [u8; 32] { *self.0.finalize().as_bytes() }
}

impl Default for Hasher {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_deterministic() {
        let a = hash(b"qorvum");
        let b = hash(b"qorvum");
        assert_eq!(a, b);
    }
    #[test]
    fn test_hash_many_eq_concat() {
        let combined = hash(b"helloworld");
        let parts    = hash_many(&[b"hello", b"world"]);
        assert_eq!(combined, parts);
    }
}
