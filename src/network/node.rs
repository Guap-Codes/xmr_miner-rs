// src/network/node.rs
use crate::AlgorithmType;
use crate::miner::scheduler::{MiningJob, Share};
use crate::utils::error::MinerError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

/// Configuration for connecting to a node's RPC interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// URL of the node's RPC endpoint (e.g., "http://127.0.0.1:18081/json_rpc")
    pub rpc_url: String,
    /// Username for RPC authentication (if required)
    pub rpc_user: String,
    /// Password for RPC authentication (if required)
    pub rpc_password: String,
    /// Wallet address that will receive mining rewards
    pub wallet_address: String,
}

/// Client for interacting with a node's RPC interface
pub struct NodeClient {
    /// Configuration for the node connection
    config: NodeConfig,
    /// HTTP client for making RPC requests
    client: Client,
    /// Current blockchain height known to this client
    current_height: u64,
}

impl NodeClient {
    /// Creates a new NodeClient with the given configuration
    ///
    /// # Arguments
    /// * `config` - Node configuration containing RPC connection details
    pub fn new(config: NodeConfig) -> Self {
        NodeClient {
            config,
            client: Client::new(),
            current_height: 0,
        }
    }

    /// Requests a new block template from the node
    ///
    /// # Returns
    /// * `Ok(MiningJob)` - Contains the job details if successful
    /// * `Err(MinerError)` - If there was an error getting the block template
    pub async fn get_block_template(&mut self) -> Result<MiningJob, MinerError> {
        let response = self
            .rpc_call(
                "getblocktemplate",
                json!({
                    "wallet_address": self.config.wallet_address,
                    "reserve_size": 8
                }),
            )
            .await?;

        let result = response["result"]
            .as_object()
            .ok_or_else(|| MinerError::ProtocolError("Missing result object".to_string()))?;

        Ok(MiningJob {
            job_id: result["job_id"]
                .as_str()
                .ok_or_else(|| MinerError::ProtocolError("Missing job_id".to_string()))?
                .to_string(),
            blob: hex::decode(result["blocktemplate_blob"].as_str().ok_or_else(|| {
                MinerError::ProtocolError("Missing blocktemplate_blob".to_string())
            })?)?,
            target: hex::decode(
                result["target"]
                    .as_str()
                    .ok_or_else(|| MinerError::ProtocolError("Missing target".to_string()))?,
            )?,
            algorithm: AlgorithmType::RandomX,
        })
    }

    /// Submits a solved block to the node
    ///
    /// # Arguments
    /// * `share` - The solved block to submit
    ///
    /// # Returns
    /// * `Ok(())` - If the submission was successful
    /// * `Err(MinerError)` - If there was an error submitting the block
    pub async fn submit_block(&self, share: Share) -> Result<(), MinerError> {
        let _ = self
            .rpc_call(
                "submitblock",
                json!({
                    "block": hex::encode(share.result)
                }),
            )
            .await?;
        Ok(())
    }

    /// Makes an RPC call to the node
    ///
    /// # Arguments
    /// * `method` - The RPC method to call
    /// * `params` - Parameters for the RPC call
    ///
    /// # Returns
    /// * `Ok(Value)` - The JSON-RPC response if successful
    /// * `Err(MinerError)` - If there was an error making the RPC call
    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, MinerError> {
        let response = self
            .client
            .post(&self.config.rpc_url)
            .basic_auth(&self.config.rpc_user, Some(&self.config.rpc_password))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": "0",
                "method": method,
                "params": params
            }))
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Monitors the blockchain for new blocks
    ///
    /// This function runs in a loop, checking for new blocks every 30 seconds.
    /// When a new block is detected, it updates the current height.
    ///
    /// # Returns
    /// * `Ok(())` - If monitoring started successfully
    /// * `Err(MinerError)` - If there was an error getting the current height
    pub async fn monitor_chain(&mut self) -> Result<(), MinerError> {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let height = self.get_current_height().await?;
            if height > self.current_height {
                self.current_height = height;
                // Trigger new job request
            }
        }
    }

    /// Gets the current blockchain height from the node
    ///
    /// # Returns
    /// * `Ok(u64)` - The current blockchain height
    /// * `Err(MinerError)` - If there was an error getting the height
    async fn get_current_height(&self) -> Result<u64, MinerError> {
        let response = self.rpc_call("get_info", json!({})).await?;
        Ok(response["result"]["height"].as_u64().unwrap_or(0))
    }
}
