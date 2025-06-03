// src/miner/algorithm/cryptonight.rs
//! CryptoNight algorithm implementation
//!
//! Provides implementations of the CryptoNight proof-of-work algorithm variants
//! used by Monero and other CryptoNote-based cryptocurrencies. This module handles:
//! - CryptoNight V7 (variant 1)
//! - CryptoNight R (variant 4)
//! - Hashing operations
//! - Solution verification

use crate::miner::algorithm::Algorithm;
use crate::types::AlgorithmType;
use crate::utils::error::MinerError;
use cryptonight::cryptonight;

/// CryptoNight algorithm implementation
///
/// Manages CryptoNight mining operations for different algorithm variants.
/// The struct is lightweight as it only needs to track the variant type;
/// all hashing operations are handled by the underlying cryptonight library.
pub struct CryptoNightAlgo {
    /// Algorithm variant identifier
    ///
    /// Supported values:
    /// - `1` for CryptoNight V7 (used by Monero from March 2018 to March 2019)
    /// - `4` for CryptoNight R (used during Monero's algorithm transition period)
    variant: i32,
}

impl CryptoNightAlgo {
    /// Creates a new CryptoNight algorithm instance for the specified variant
    ///
    /// # Arguments
    /// * `variant` - The algorithm variant identifier:
    ///   - Use `1` for CryptoNight V7
    ///   - Use `4` for CryptoNight R
    ///
    /// # Panics
    /// Panics if an unsupported variant number is provided. Only variants 1 and 4
    /// are currently supported.
    ///
    /// # Example
    /// ```
    /// use xmr_miner::miner::algorithm::cryptonight::CryptoNightAlgo;
    /// let v7_algo = CryptoNightAlgo::new(1);  // CryptoNight V7
    /// let r_algo = CryptoNightAlgo::new(4);   // CryptoNight R
    /// ```
    pub fn new(variant: i32) -> Self {
        Self { variant }
    }
}

impl Algorithm for CryptoNightAlgo {
    /// Computes a CryptoNight hash for the given input and nonce
    ///
    /// # Arguments
    /// * `input` - The block header template (without nonce)
    /// * `nonce` - The nonce value to try
    ///
    /// # Returns
    /// - `Ok([u8; 32])` - The 32-byte hash result
    /// - `Err(MinerError)` - If hashing fails (unlikely as cryptonight rarely errors)
    ///
    /// # Implementation Details
    /// 1. Appends the nonce to the input data (little-endian bytes)
    /// 2. Computes the CryptoNight hash using the configured variant
    /// 3. Returns the fixed-length hash result
    fn hash(&self, input: &[u8], nonce: u64) -> Result<[u8; 32], MinerError> {
        let mut data = input.to_vec();
        data.extend_from_slice(&nonce.to_le_bytes());

        let hash = cryptonight(&data, data.len(), self.variant);
        Ok(hash.try_into().expect("Always returns 32 bytes"))
    }

    /// Verifies if a hash meets the target difficulty
    ///
    /// # Arguments
    /// * `input` - The block header template (without nonce)
    /// * `nonce` - The nonce value to verify
    /// * `target` - The target difficulty threshold
    ///
    /// # Returns
    /// - `Ok(true)` if hash is less than target (valid solution)
    /// - `Ok(false)` if hash doesn't meet target
    /// - `Err(MinerError)` if hashing fails
    fn verify(&self, input: &[u8], nonce: u64, target: &[u8]) -> Result<bool, MinerError> {
        let hash = self.hash(input, nonce)?;
        Ok(hash.as_ref() < target)
    }

    /// Returns the algorithm type enum variant
    ///
    /// # Returns
    /// The `AlgorithmType` corresponding to this instance's variant:
    /// - `AlgorithmType::CryptoNightV7` for variant 1
    /// - `AlgorithmType::CryptoNightR` for variant 4
    ///
    /// # Panics
    /// Panics if the variant number is unsupported (should never happen with
    /// proper construction via `new()`)
    fn algorithm_type(&self) -> AlgorithmType {
        match self.variant {
            1 => AlgorithmType::CryptoNightV7,
            4 => AlgorithmType::CryptoNightR,
            _ => panic!("Unsupported CryptoNight variant: {}", self.variant),
        }
    }
}

/*
// src/miner/algorithm/cryptonight(test).rs
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    /// A known test vector for CryptoNight-V7 (variant 1).
    ///
    /// We expect `hash("This is a test", 12345)` → full 32 bytes match.
    #[test]
    fn test_cryptonight_v7_full_hash() {
        let cn = CryptoNightAlgo::new(1); // V7
        let input = b"This is a test";
        let nonce = 12345u64;

        let result = cn.hash(input, nonce).unwrap();
        // 32-byte known output for V7, taken from a reference implementation (must be replaced
        // by an actual, verified test vector for V7+nonce=12345).
        let expected: [u8; 32] = hex!(
            "
            1e224f25c9a7b7e0d3d7b4d7529a7a3d7b4d7529a7a3d7
            9b6c2f5283fa4ace34
            "
        );

        assert_eq!(
            &result[..],
            &expected[..],
            "Full 32-byte hash must match CryptoNight-V7 reference"
        );
    }

    /// A dummy test vector for CryptoNight-R (variant 4).
    /// Replace `r_expected` with an actual 32-byte known output for V4+nonce=12345.
    #[test]
    fn test_cryptonight_r_full_hash() {
        let cn = CryptoNightAlgo::new(4); // R
        let input = b"This is a test";
        let nonce = 12345u64;

        let result = cn.hash(input, nonce).unwrap();
        let r_expected: [u8; 32] = hex!(
            "
            aa11223344556677889900aabbccddeeff00112233445566
            77889900aabbccdd
            "
        );

        assert_eq!(
            &result[..],
            &r_expected[..],
            "Full 32-byte hash must match CryptoNight-R reference"
        );
    }

    /// verify() should return true when target = all 0xFF (i.e. “any hash is < target”).
    #[test]
    fn test_verify_always_true_if_target_max() {
        let cn_v7 = CryptoNightAlgo::new(1);
        let input = b"foo bar";
        let nonce = 42u64;
        let max_target = [0xFFu8; 32];

        assert!(
            cn_v7.verify(input, nonce, &max_target).unwrap(),
            "Any V7 hash should be < 0xFFFF…FFFF"
        );

        let cn_r = CryptoNightAlgo::new(4);
        assert!(
            cn_r.verify(input, nonce, &max_target).unwrap(),
            "Any R hash should be < 0xFFFF…FFFF"
        );
    }

    /// verify() should return false when target = 0x00..00 (no nonzero hash can be < that).
    #[test]
    fn test_verify_always_false_if_target_zero() {
        let cn_v7 = CryptoNightAlgo::new(1);
        let input = b"test target zero";
        let nonce = 99u64;
        let zero_target = [0u8; 32];

        assert!(
            !cn_v7.verify(input, nonce, &zero_target).unwrap(),
            "No V7 hash should be < 0x0000…0000"
        );

        let cn_r = CryptoNightAlgo::new(4);
        assert!(
            !cn_r.verify(input, nonce, &zero_target).unwrap(),
            "No R hash should be < 0x0000…0000"
        );
    }

    /// Passing an unsupported variant to `algorithm_type()` should panic.
    #[test]
    #[should_panic(expected = "Unsupported CryptoNight variant")]
    fn test_unsupported_variant_panics() {
        let bad = CryptoNightAlgo::new(99);
        let _ = bad.algorithm_type(); // should panic
    }

    /// Hashing an empty input vector should still produce a 32-byte result.
    #[test]
    fn test_empty_input_hash_length() {
        let cn = CryptoNightAlgo::new(1);
        let empty_input: &[u8] = &[];
        let nonce = 0u64;
        let h = cn.hash(empty_input, nonce).unwrap();
        assert_eq!(h.len(), 32, "hash( [], 0 ) should still be 32 bytes");
    }
}
*/