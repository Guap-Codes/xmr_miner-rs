// src/config/config.rs
use crate::{
    network::{node::NodeConfig, pool::PoolConfig},
    utils::error::MinerError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure for the mining application
///
/// Contains all settings needed to configure mining operations,
/// including algorithm selection, worker configuration, and
/// mining mode (pool or node).
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Mining algorithm to use (e.g., "randomx", "cryptonight-v7")
    #[serde(default = "default_algorithm")]
    pub algorithm: String,

    /// Number of worker threads to use for mining
    /// (default: number of CPU cores)
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,

    /// Size of nonce batches each worker processes at once
    /// (default: 1000)
    #[serde(default = "default_batch_size")]
    pub batch_size: u64,

    /// Mining mode configuration (pool or node)
    pub mode: MiningMode,
}

/// Enum representing different mining modes
///
/// Determines whether the miner connects to a pool
/// or mines directly to a node.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MiningMode {
    /// Pool mining configuration
    Pool(PoolConfig),

    /// Node mining configuration
    Node(NodeConfig),
}

fn default_algorithm() -> String {
    "randomx".into()
}

fn default_worker_threads() -> usize {
    num_cpus::get()
}

fn default_batch_size() -> u64 {
    1000
}

impl Config {
    /// Loads configuration from a file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file (TOML format)
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err(MinerError)` - If file couldn't be read or parsed
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, MinerError> {
        let path = path.into();
        let config_str = std::fs::read_to_string(&path).map_err(|e| {
            MinerError::ConfigError(format!(
                "Failed to read config at {}: {}",
                path.display(),
                e
            ))
        })?;

        toml::from_str(&config_str)
            .map_err(|e| MinerError::ConfigError(format!("Invalid config format: {}", e)))
    }

    /// Generates a configuration template string
    ///
    /// # Arguments
    /// * `pool` - Include pool mining configuration template
    /// * `node` - Include node mining configuration template
    ///
    /// # Returns
    /// String containing a commented TOML configuration template
    pub fn generate_template(pool: bool, node: bool) -> String {
        let mut template = String::new();
        template.push_str("# XMR Miner Configuration\n\n");
        template.push_str("[general]\n");
        template.push_str("# Supported algorithms: randomx, cryptonight-v7, cryptonight-r\n");
        template.push_str("algorithm = \"randomx\"\n");
        template.push_str("# Number of worker threads (0 = auto-detect)\n");
        template.push_str("worker_threads = 0\n");
        template.push_str("# Nonce batch size per worker\n");
        template.push_str("batch_size = 1000\n\n");

        if pool {
            template.push_str("# Pool mining configuration\n");
            template.push_str("[mode.pool]\n");
            template.push_str("url = \"stratum+tcp://pool.example.com:3333\"\n");
            template.push_str("user = \"your_wallet_address\"\n");
            template.push_str("password = \"x\"\n");
            template.push_str("worker_id = \"worker01\"\n");
        }

        if node {
            template.push_str("\n# Node mining configuration\n");
            template.push_str("[mode.node]\n");
            template.push_str("rpc_url = \"http://localhost:18081/json_rpc\"\n");
            template.push_str("rpc_user = \"monero\"\n");
            template.push_str("rpc_password = \"password\"\n");
            template.push_str("wallet_address = \"your_wallet_address\"\n");
        }

        template
    }
}
