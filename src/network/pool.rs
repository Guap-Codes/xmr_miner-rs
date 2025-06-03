// src/network/pool.rs

//! Mining pool client implementation
//!
//! Handles communication with mining pools using the Stratum protocol over WebSocket.
//! Manages connection lifecycle, job distribution, and share submission.
use crate::miner::scheduler::{MiningJob, Share};
use crate::types::AlgorithmType;
use crate::utils::error::MinerError;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message;
use url::Url;

/// Configuration for connecting to a mining pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Pool connection URL (e.g., "stratum+tcp://pool.example.com:3333")
    pub url: String,
    /// Wallet address or pool username
    pub user: String,
    /// Worker password (often "x" if not required)
    pub password: String,
    /// Worker identifier for statistics tracking
    pub worker_id: String,
}

/// Client for communicating with a mining pool
///
/// Handles all pool protocol interactions including:
/// - Connection management
/// - Job distribution to miners
/// - Share submission
/// - Keepalive messages
pub struct PoolClient {
    /// Pool connection configuration
    config: PoolConfig,
    /// Thread-safe WebSocket connection handle
    connection: Mutex<Option<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>>,
    /// Channel for sending received jobs to miners
    job_sender: crossbeam_channel::Sender<MiningJob>,
    /// Channel for receiving shares from miners (wrapped in Arc for thread safety)
    share_receiver: Arc<crossbeam_channel::Receiver<Share>>,
}

impl PoolClient {
    /// Creates a new PoolClient instance
    ///
    /// # Arguments
    /// * `config` - Pool connection configuration
    /// * `job_sender` - Channel for sending jobs to miner workers
    /// * `share_receiver` - Channel for receiving shares from miners
    pub fn new(
        config: PoolConfig,
        job_sender: crossbeam_channel::Sender<MiningJob>,
        share_receiver: crossbeam_channel::Receiver<Share>,
    ) -> Self {
        PoolClient {
            config,
            connection: Mutex::new(None),
            job_sender,
            share_receiver: Arc::new(share_receiver),
        }
    }

    /// Establishes connection to the mining pool
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - URL is invalid
    /// - DNS resolution fails
    /// - WebSocket handshake fails
    pub async fn connect(&self) -> Result<(), MinerError> {
        let url_str = &self.config.url;
        let url = Url::parse(url_str)
            .map_err(|e| MinerError::ConfigError(format!("Invalid URL '{}': {}", url_str, e)))?;

        if url.scheme() != "ws" && url.scheme() != "wss" {
            log::warn!(
                "Pool URL '{}' uses non-WebSocket scheme. Consider using 'ws://' or 'wss://'",
                url_str
            );
        }

        match tokio_tungstenite::connect_async(url_str).await {
            Ok((ws_stream, _)) => {
                let mut conn = self.connection.lock().await;
                *conn = Some(ws_stream);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Connection to '{}' failed: {}", url_str, e);
                if e.to_string().contains("dns error") {
                    Err(MinerError::ConnectionError(format!(
                        "DNS resolution failed. Check pool URL: {}",
                        url_str
                    )))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Main event loop for pool communication
    ///
    /// Handles:
    /// - Receiving jobs from pool
    /// - Submitting shares to pool
    /// - Sending keepalive messages
    ///
    /// # Errors
    /// Returns `MinerError` if communication fails
    pub async fn run(&self) -> Result<(), MinerError> {
        self.login().await?;
        self.subscribe().await?;

        let mut interval = time::interval(Duration::from_secs(30));
        let mut conn = self.connection.lock().await;
        let ws = conn
            .as_mut()
            .ok_or(MinerError::ConnectionError("Not connected".into()))?;

        loop {
            let receiver = Arc::clone(&self.share_receiver);
            tokio::select! {
                msg = ws.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => self.handle_message(&text).await?,
                        Some(Err(e)) => return Err(e.into()),
                        None => return Ok(()),
                        _ => {}
                    }
                }
                _ = interval.tick() => {
                    self.keep_alive().await?;
                }
                share = tokio::task::spawn_blocking(move || receiver.recv()) => {
                    if let Ok(share) = share? {
                        self.submit_share(&share).await?;
                    }
                }
            }
        }
    }

    /// Handles incoming WebSocket messages from the pool
    ///
    /// # Arguments
    /// * `message` - The raw JSON message received from pool
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - Message parsing fails
    /// - Job handling fails
    async fn handle_message(&self, message: &str) -> Result<(), MinerError> {
        let json: Value = serde_json::from_str(message)?;

        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
            match method {
                "job" => self.handle_job(&json).await?,
                _ => log::warn!("Unknown method received: {}", method),
            }
        }

        Ok(())
    }

    /// Processes incoming mining job notifications
    ///
    /// # Arguments
    /// * `json` - Parsed JSON message containing job details
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - Required fields are missing
    /// - Hex decoding fails
    /// - Algorithm parsing fails
    /// - Job channel send fails
    async fn handle_job(&self, json: &Value) -> Result<(), MinerError> {
        let params = json["params"]
            .as_object()
            .ok_or_else(|| MinerError::ProtocolError("Missing params object".to_string()))?;

        let job = MiningJob {
            job_id: params["job_id"]
                .as_str()
                .ok_or_else(|| MinerError::ProtocolError("Missing job_id".to_string()))?
                .to_string(),
            blob: hex::decode(
                params["blob"]
                    .as_str()
                    .ok_or_else(|| MinerError::ProtocolError("Missing blob".to_string()))?,
            )?,
            target: hex::decode(
                params["target"]
                    .as_str()
                    .ok_or_else(|| MinerError::ProtocolError("Missing target".to_string()))?,
            )?,
            algorithm: AlgorithmType::from_str(
                params["algo"]
                    .as_str()
                    .ok_or_else(|| MinerError::ProtocolError("Missing algo".to_string()))?,
            )
            .map_err(|e| MinerError::ProtocolError(e))?,
        };

        self.job_sender.send(job)?;
        Ok(())
    }

    /// Sends login request to the mining pool
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - WebSocket communication fails
    async fn login(&self) -> Result<(), MinerError> {
        let message = json!({
            "method": "login",
            "params": {
                "login": self.config.user,
                "pass": self.config.password,
                "agent": format!("xmr_miner-rs/{}", env!("CARGO_PKG_VERSION"))
            },
            "id": 1
        });

        self.send(message).await
    }

    /// Sends subscription request to the mining pool
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - WebSocket communication fails
    async fn subscribe(&self) -> Result<(), MinerError> {
        let message = json!({
            "method": "subscribe",
            "params": {
                "worker_id": self.config.worker_id
            },
            "id": 2
        });

        self.send(message).await
    }

    /// Submits a completed share to the mining pool
    ///
    /// # Arguments
    /// * `share` - The share to submit
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - WebSocket communication fails
    async fn submit_share(&self, share: &Share) -> Result<(), MinerError> {
        let message = json!({
            "method": "submit",
            "params": {
                "id": self.config.worker_id,
                "job_id": share.job_id,
                "nonce": format!("{:08x}", share.nonce),
                "result": hex::encode(share.result)
            },
            "id": 3
        });

        self.send(message).await
    }

    /// Sends keepalive message to maintain connection
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - WebSocket communication fails
    async fn keep_alive(&self) -> Result<(), MinerError> {
        self.send(json!({"method": "keepalived"})).await
    }

    /// Internal helper for sending JSON messages over WebSocket
    ///
    /// # Arguments
    /// * `value` - The JSON value to send
    ///
    /// # Errors
    /// Returns `MinerError` if:
    /// - Not connected to pool
    /// - WebSocket send fails
    async fn send(&self, value: Value) -> Result<(), MinerError> {
        let mut conn = self.connection.lock().await;
        let ws = conn
            .as_mut()
            .ok_or(MinerError::ConnectionError("Not connected".into()))?;
        ws.send(Message::Text(value.to_string().into())).await?;
        Ok(())
    }
}
