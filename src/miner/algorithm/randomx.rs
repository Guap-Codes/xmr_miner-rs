// src/miner/algorithm/randomx.rs
//! RandomX algorithm implementation
//!
//! Provides the RandomX proof-of-work algorithm implementation used by Monero.
//! This module handles all RandomX-specific mining operations including:
//! - Dataset initialization
//! - Hashing operations
//! - Verification of solutions

use crate::miner::algorithm::Algorithm;
use crate::types::AlgorithmType;
use crate::utils::error::MinerError;
use rust_randomx::{Context, Hasher};
use std::sync::Arc;

/// RandomX algorithm implementation
///
/// Manages the RandomX context (dataset) and provides thread-safe hashing operations.
/// The implementation uses reference-counted pointers to share the heavy dataset
/// between threads while allowing each thread to maintain its own lightweight hasher.
#[derive(Clone)]
pub struct RandomX {
    /// Shared RandomX context containing the dataset
    ///
    /// This is the memory-intensive component that's shared across all threads.
    /// Wrapped in Arc for thread-safe reference counting.
    context: Arc<Context>,

    /// Thread-safe hasher instance
    ///
    /// Each RandomX instance maintains its own hasher to avoid contention.
    /// Wrapped in Arc to support cloning across threads.
    hasher: Arc<Hasher>,
}

impl RandomX {
    /// Creates a new RandomX instance with initialized dataset
    ///
    /// # Arguments
    /// * `fast` - Enables fast mode when true (uses more memory but better performance)
    /// * `key` - The key/seed used to initialize the dataset (typically block seed)
    ///
    /// # Panics
    /// May panic if:
    /// - Key length is invalid (not 32 bytes)
    /// - Memory allocation for dataset fails
    ///
    /// # Performance Notes
    /// - Initialization is expensive (dataset generation takes several seconds)
    /// - Fast mode requires ~2080MB RAM vs ~256MB in light mode
    pub fn new(fast: bool, key: &[u8]) -> Self {
        // Create Arc-wrapped Context first
        let context = Arc::new(Context::new(key, fast));

        // Hasher needs Arc<Context>
        let hasher = Arc::new(Hasher::new(Arc::clone(&context)));

        Self { context, hasher }
    }

    /// Creates a new thread-local hasher instance
    ///
    /// Used internally to provide thread-safe hashing operations without
    /// requiring mutex locks on the hasher.
    fn create_hasher(&self) -> Hasher {
        Hasher::new(Arc::clone(&self.context))
    }
}

impl Algorithm for RandomX {
    /// Computes a RandomX hash for the given input and nonce
    ///
    /// # Arguments
    /// * `input` - The block header template (without nonce)
    /// * `nonce` - The nonce value to try
    ///
    /// # Returns
    /// - `Ok([u8; 32])` - The computed hash
    /// - `Err(MinerError)` - If hashing fails
    ///
    /// # Implementation Details
    /// 1. Appends nonce to input (little-endian bytes)
    /// 2. Computes RandomX hash
    /// 3. Converts output to fixed-size array
    fn hash(&self, input: &[u8], nonce: u64) -> Result<[u8; 32], MinerError> {
        let hasher = self.create_hasher();
        let mut full_input = input.to_vec();
        full_input.extend_from_slice(&nonce.to_le_bytes());

        // Correct hash usage - returns Output struct
        let output = hasher.hash(&full_input);

        // Convert Output to byte array using AsRef<[u8]> implementation
        Ok(output.as_ref().try_into().unwrap())
    }

    /// Verifies if a hash meets the target difficulty
    ///
    /// # Arguments
    /// * `input` - The block header template (without nonce)
    /// * `nonce` - The nonce value to verify
    /// * `target` - The target difficulty threshold
    ///
    /// # Returns
    /// - `Ok(true)` if hash < target
    /// - `Ok(false)` otherwise
    /// - `Err(MinerError)` if hashing fails
    fn verify(&self, input: &[u8], nonce: u64, target: &[u8]) -> Result<bool, MinerError> {
        let hash = self.hash(input, nonce)?;
        Ok(hash.as_ref() < target)
    }

    /// Returns the algorithm type (RandomX)
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::RandomX
    }
}

/*
// src/miner/algorithm/randomx(test).rs
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    /// Known (key, input, nonce) → 32-byte RandomX output from a reference impl.
    /// Replace these with real vectors.
    const KEY: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEF"; // exactly 32 bytes
    const INPUT1: &[u8] = b"Hello, World!";
    const NONCE1: u64 = 0;
    const EXPECTED1: [u8; 32] =
        hex!("1a3ffbee270b222f6c0edf7c7a3150c021f2d7d8c4e50fcd9047c0d108d717f1");

    const INPUT2: &[u8] = b"The quick brown fox jumps over the lazy dog";
    const NONCE2: u64 = 1;
    const EXPECTED2: [u8; 32] =
        hex!("e4b9f4d5c1a178b2a3e814f77bf16b2f8c970b6aee23eb9c4790fa9a0fbb7da5");

    #[test]
    fn test_randomx_hash_basic() {
        let rx = RandomX::new(true, KEY);
        let output = rx.hash(INPUT1, NONCE1).unwrap();
        assert_eq!(
            output,
            EXPECTED1,
            "RandomX hash({}, {}) did not match the reference vector",
            String::from_utf8_lossy(INPUT1),
            NONCE1
        );

        // Check a second vector to ensure different nonces/inputs work.
        let output2 = rx.hash(INPUT2, NONCE2).unwrap();
        assert_eq!(
            output2,
            EXPECTED2,
            "RandomX hash({}, {}) did not match the second reference vector",
            String::from_utf8_lossy(INPUT2),
            NONCE2
        );
    }

    #[test]
    fn test_randomx_verify_true_false() {
        let rx = RandomX::new(true, KEY);
        // First, verify with a max-target (all 0xFF) → always true.
        let max_target = [0xFFu8; 32];
        assert!(
            rx.verify(INPUT1, NONCE1, &max_target).unwrap(),
            "hash < max_target must be true"
        );

        // Now use zero target → always false.
        let zero_target = [0u8; 32];
        assert!(
            !rx.verify(INPUT1, NONCE1, &zero_target).unwrap(),
            "hash < zero_target must be false"
        );

        // Finally create a real “boundary” target exactly equal to EXPECTED1, so verify() is false:
        //   h = EXPECTED1; if target == EXPECTED1 then (h < target) is false, (h <= target) would be true.
        let boundary_target = EXPECTED1;
        assert!(
            !rx.verify(INPUT1, NONCE1, &boundary_target).unwrap(),
            "hash == target should be false when using `< target` logic"
        );
    }

    #[test]
    #[should_panic(expected = "Panicked at")]
    fn test_randomx_new_wrong_key_length() {
        // Key must be exactly 32 bytes, so a shorter slice should cause a panic internally.
        let short_key = b"short";
        let _ = RandomX::new(true, short_key);
    }

    #[test]
    fn test_randomx_empty_input() {
        let rx = RandomX::new(true, KEY);
        // Even with empty input and nonce = 0, the hasher should return 32 bytes.
        let empty = &[];
        let result = rx.hash(empty, 0).unwrap();
        assert_eq!(
            result.len(),
            32,
            "hash([], 0) should still produce a 32-byte output"
        );
    }

    #[test]
    fn test_randomx_algorithm_type() {
        let rx = RandomX::new(true, KEY);
        assert_eq!(
            rx.algorithm_type(),
            AlgorithmType::RandomX,
            "algorithm_type() must return AlgorithmType::RandomX"
        );
    }

    #[test]
    fn test_randomx_thread_safety() {
        use std::thread;

        let rx = Arc::new(RandomX::new(true, KEY));
        let mut handles = vec![];

        // Spawn 4 threads, each hashing a different nonce.
        for i in 0u64..4 {
            let rx_clone = rx.clone();
            handles.push(thread::spawn(move || {
                let res = rx_clone.hash(INPUT1, i).unwrap();
                // We won't assert a specific value here—just ensure no panic/data race.
                assert_eq!(res.len(), 32);
            }));
        }

        for h in handles {
            h.join().expect("Thread panicked or resulted in Err");
        }
    }
}
*/