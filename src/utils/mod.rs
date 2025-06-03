// src/utils/mod.rs
//! Utilities module for common functionality
//!
//! This module contains shared utilities used throughout the mining application,
//! including error handling and logging infrastructure.

/// Error types and handling utilities
///
/// Contains the [`MinerError`] enum which defines all possible error conditions
/// for the mining application, along with conversion implementations.
pub mod error;

/// Logging configuration and utilities
///
/// Provides logging initialization and configuration for the application,
/// including formatting and output destinations.
pub mod logging;

// Re-export for easier access
pub use error::MinerError;
pub use logging::init_logging;
