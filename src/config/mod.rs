// src/config/mod.rs
//! Configuration management for the XMR miner
//!
//! This module handles all configuration-related functionality including:
//! - Loading and parsing configuration files
//! - Generating configuration templates
//! - Managing mining mode settings
//!
//! The configuration uses TOML format and supports both pool mining
//! and direct node mining configurations.

/// Core configuration implementation
///
/// Contains the [`Config`] struct and related types that define
/// the miner's configuration structure and behavior.
pub mod config;

// Re-export key items for easy access
pub use config::{Config, MiningMode};

use crate::utils::error::MinerError;
use std::path::PathBuf;

/// Loads miner configuration from a TOML file
///
/// # Arguments
/// * `path` - Path to the configuration file (anything convertible to PathBuf)
///
/// # Returns
/// * `Ok(Config)` - Successfully loaded configuration
/// * `Err(MinerError)` - If the file couldn't be read or parsed
pub fn load(path: impl Into<PathBuf>) -> Result<Config, MinerError> {
    Config::load(path)
}

/// Generates a commented configuration template
///
/// # Arguments
/// * `pool` - Whether to include pool mining configuration section
/// * `node` - Whether to include node mining configuration section
///
/// # Returns
/// String containing a ready-to-use TOML configuration template
pub fn generate_template(pool: bool, node: bool) -> String {
    Config::generate_template(pool, node)
}
