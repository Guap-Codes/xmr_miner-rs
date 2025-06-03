//! XMR Miner - High performance Monero mining implementation in Rust
//!
//! This crate provides a complete implementation of a Monero (XMR) miner with support for:
//! - Multiple mining algorithms (RandomX, CryptoNight variants)
//! - Both pool and solo mining modes
//! - Performance benchmarking
//! - Hardware monitoring

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Miner core implementation including algorithms and scheduling
pub mod miner;

/// Network communication components for pool and node connections
pub mod network;

/// Statistics collection and reporting functionality
pub mod stats;

/// Utility functions and error handling
pub mod utils;

/// Command-line interface definitions
pub mod cli;

/// Configuration management
pub mod config;

/// Shared type definitions
pub mod types;

// Core exports
pub use cli::Commands;
pub use config::Config;
pub use miner::{Algorithm, MiningJob, Scheduler, Share, Worker};
pub use network::{NodeClient, PoolClient};
pub use stats::{HardwareStats, MiningStats, StatsReporter};
pub use types::AlgorithmType;
pub use utils::{MinerError, init_logging};
