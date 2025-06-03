//! Statistics collection and reporting module
//!
//! This module provides functionality for tracking and reporting mining statistics,
//! including:
//! - Hashrate calculations
//! - Share acceptance/rejection tracking
//! - Hardware monitoring (CPU, memory, temperature)
//!
//! The main component is [`StatsReporter`] which collects data and can periodically
//! report statistics to logs or other outputs.
//!

/// Submodule containing the statistics reporter implementation
///
/// The reporter handles:
/// - Atomic collection of mining statistics
/// - Hardware monitoring
/// - Periodic reporting of stats
/// - Thread-safe communication channels for receiving data
pub mod reporter;

// Re-export main components
pub use reporter::{HardwareStats, MiningStats, StatsReporter};
