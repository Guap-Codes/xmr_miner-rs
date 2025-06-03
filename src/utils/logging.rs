// src/utils/logging.rs
//! Logging configuration and utilities
//!
//! This module handles logging setup for the miner application, including:
//! - Standard logging configuration
//! - Benchmark-specific logging
//! - Custom log formatting
//!
//! Uses `env_logger` under the hood with custom formatting and filtering.

use env_logger::{Builder, Target};
use log::LevelFilter;
use std::env;

/// Initializes the logging subsystem with sensible defaults
///
/// # Configuration
/// - Logs to stdout
/// - Default log level: Info
/// - Custom timestamp and source location formatting
/// - Respects `RUST_LOG` environment variable if set
pub fn init_logging() {
    common_log_config().filter(None, LevelFilter::Info).init();
}

/// Configures benchmark-specific logging
///
/// # Differences from Standard Logging
/// - Default log level: Debug (if RUST_LOG not set)
/// - More verbose output by default
/// - Same custom formatting as standard logging
pub fn init_bench_logging() {
    let mut builder = common_log_config();

    // Set default to debug level if RUST_LOG not configured
    if env::var("RUST_LOG").is_err() {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.parse_env("RUST_LOG");
    }

    builder.init();
}

/// Creates and configures a base logger builder with common settings
///
/// # Features
/// - Custom log format including:
///   - Timestamp (seconds since epoch)
///   - Log level
///   - Module path
///   - Line number
///   - Message
/// - Output to stdout
///
/// # Returns
/// Partially configured `env_logger::Builder` instance
fn common_log_config() -> Builder {
    let mut builder = Builder::new();

    builder
        .format(|buf, record| {
            use std::io::Write;
            let ts = buf.timestamp_seconds();
            let level = record.level();
            let module = record.module_path().unwrap_or_default();
            let line = record.line().unwrap_or(0);

            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                ts,
                level,
                module,
                line,
                record.args()
            )
        })
        .target(Target::Stdout);

    builder
}
