// src/utils/error.rs
use crate::miner::scheduler;
use serde_json;
use std::io;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use url;

/// Main error type for the mining application
///
/// This enum represents all possible error conditions that can occur
/// during mining operations, including network, I/O, protocol, and
/// configuration errors.
#[derive(Error, Debug)]
pub enum MinerError {
    /// Errors related to mining algorithms (e.g., unsupported algorithm)
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),

    /// Errors related to network connectivity
    #[error("Network connection error: {0}")]
    ConnectionError(String),

    /// Errors in protocol handling or invalid protocol messages
    #[error("Protocol violation: {0}")]
    ProtocolError(String),

    /// Standard I/O operation errors
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// URL parsing errors
    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    /// WebSocket communication errors
    #[error("WebSocket error: {0}")]
    WsError(#[from] tungstenite::Error),

    /// HTTP request/response errors
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Configuration file or parameter errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Thread communication channel errors
    #[error("Thread communication error: {0}")]
    ChannelError(String),

    /// Invalid user input or parameter errors
    #[error("Invalid input: {0}")]
    InputError(String),

    /// Cryptographic operation errors
    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    /// Async task execution errors
    #[error("Task execution error: {0}")]
    TaskError(String),
}

/// Converts crossbeam channel send errors for Shares into MinerError
///
/// Used when failing to send mining shares through inter-thread channels.
/// Wraps the original error in a `ChannelError` variant with context.
impl From<crossbeam_channel::SendError<scheduler::Share>> for MinerError {
    fn from(e: crossbeam_channel::SendError<scheduler::Share>) -> Self {
        MinerError::ChannelError(format!("Share send failed: {}", e))
    }
}

/// Converts crossbeam channel send errors for MiningJobs into MinerError
///
/// Used when failing to send new mining jobs through inter-thread channels.
/// Wraps the original error in a `ChannelError` variant with context.
impl From<crossbeam_channel::SendError<scheduler::MiningJob>> for MinerError {
    fn from(e: crossbeam_channel::SendError<scheduler::MiningJob>) -> Self {
        MinerError::ChannelError(format!("Job send failed: {}", e))
    }
}

/// Converts hex decoding errors into MinerError
///
/// Used when invalid hex data is encountered during:
/// - Block template processing
/// - Share verification
/// - Configuration parsing
/// Wraps the original error in an `InputError` variant.
impl From<hex::FromHexError> for MinerError {
    fn from(e: hex::FromHexError) -> Self {
        MinerError::InputError(format!("Hex conversion failed: {}", e))
    }
}

/// Converts async task join errors into MinerError
///
/// Used when background tasks fail unexpectedly, including:
/// - Network operations
/// - Mining threads
/// - Monitoring tasks
/// Wraps the original error in a `TaskError` variant.
impl From<tokio::task::JoinError> for MinerError {
    fn from(e: tokio::task::JoinError) -> Self {
        MinerError::TaskError(format!("Async task failed: {}", e))
    }
}
