// src/miner/mod.rs
//! Core mining functionality
//!
//! This module contains all components related to the mining process:
//! - Algorithm implementations (RandomX, CryptoNight)
//! - Job scheduling and distribution
//! - Worker thread management

/// Mining algorithm implementations
///
/// Contains implementations of supported mining algorithms:
/// - RandomX (for Monero's current algorithm)
/// - CryptoNight variants (for historical/alternative chains)
pub mod algorithm;

/// Mining job scheduler
///
/// Handles distribution of mining jobs to workers and collection of shares.
/// Manages the current active job and nonce distribution.
pub mod scheduler;

/// Worker thread implementation
///
/// Contains the worker thread logic that performs actual hash computations.
/// Workers receive jobs from the scheduler and submit found shares.
pub mod worker;

// Re-export main components for cleaner imports
pub use self::algorithm::Algorithm;
pub use self::scheduler::{MiningJob, Scheduler, Share};
pub use self::worker::Worker;
