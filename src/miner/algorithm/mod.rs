// src/miner/algorithm/mod.rs
//! Mining algorithm implementations
//!
//! This module contains all supported mining algorithms and their common interface.
//! Currently implements:
//! - RandomX (Monero's current algorithm)
//! - CryptoNight variants (for historical/alternative chains)

/// RandomX algorithm implementation
///
/// Implements the RandomX proof-of-work algorithm used by Monero since 2019.
/// Requires significant memory allocation for the dataset.
pub mod randomx;

/// CryptoNight algorithm implementations
///
/// Contains variants of the original CryptoNight algorithm:
/// - CryptoNightV7 (Monero's 2018-2019 algorithm)
/// - CryptoNightR (Monero's 2019 variant)
pub mod cryptonight;

use crate::types::AlgorithmType;
use crate::utils::error::MinerError;

/// Common interface for all mining algorithms
///
/// All mining algorithm implementations must provide these basic operations
/// to be compatible with the mining scheduler.
pub trait Algorithm: Send + Sync {
    /// Compute the hash for given input data and nonce
    ///
    /// # Arguments
    /// * `input` - The block header or other data to be hashed
    /// * `nonce` - The nonce value to use in the hash computation
    ///
    /// # Returns
    /// 32-byte hash result or error if computation fails
    fn hash(&self, input: &[u8], nonce: u64) -> Result<[u8; 32], MinerError>;

    /// Verify if a hash meets the target difficulty
    ///
    /// # Arguments
    /// * `input` - The original input data
    /// * `nonce` - The nonce that produced the hash
    /// * `target` - The target difficulty to compare against
    ///
    /// # Returns
    /// `true` if hash is valid (less than target), `false` otherwise
    fn verify(&self, input: &[u8], nonce: u64, target: &[u8]) -> Result<bool, MinerError>;

    /// Get the algorithm type
    ///
    /// # Returns
    /// The specific algorithm variant being used
    fn algorithm_type(&self) -> AlgorithmType;
}
/*
Recommended Optimizations:

    Add GPU support using OpenCL/CUDA

    Implement automatic algorithm selection

    Add hardware detection for SIMD instructions

    Implement warm-up routines for RandomX

    Add benchmark tests for performance monitoring

    Implement cache/dataset pre-allocation

    Add memory locking for large allocations
*/
