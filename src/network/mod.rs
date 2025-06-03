// src/network/mod.rs
//! Network communication components
//!
//! This module handles all network interactions with mining pools and Monero nodes.
//! It provides two main client implementations:
//! - `PoolClient`: For connecting to mining pools using Stratum protocol
//! - `NodeClient`: For solo mining against a local Monero node

/// Mining pool client implementation
///
/// Handles communication with mining pools using the Stratum protocol.
/// Manages WebSocket connections, job distribution, and share submission.
pub mod pool;

/// Monero node client implementation
///
/// Handles communication with a local Monero node for solo mining.
/// Uses JSON-RPC to interact with the node's mining API.
pub mod node;

// Re-export main components for cleaner imports
pub use node::NodeClient;
pub use pool::PoolClient;
