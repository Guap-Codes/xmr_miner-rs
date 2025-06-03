// src/miner/scheduler.rs
//! Mining job scheduler implementation
//!
//! Manages the distribution of mining jobs to workers and collection of shares.
//! Handles job updates, nonce distribution, and worker coordination.

use crate::miner::algorithm::Algorithm;
use crate::types::AlgorithmType;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
//use crate::utils::error::MinerError;

/// Represents a mining job received from the pool or node
#[derive(Debug, Clone)]
pub struct MiningJob {
    /// Unique identifier for the job
    pub job_id: String,
    /// Block data blob to be hashed
    pub blob: Vec<u8>,
    /// Target difficulty for this job
    pub target: Vec<u8>,
    /// Algorithm to use for this job
    pub algorithm: AlgorithmType,
}

/// Represents a valid share found by a worker
#[derive(Debug, Clone)]
pub struct Share {
    /// Job ID this share belongs to
    pub job_id: String,
    /// Nonce that produced the valid hash
    pub nonce: u64,
    /// Resulting hash that meets the target
    pub result: [u8; 32],
}

/// Coordinates mining jobs across worker threads
pub struct Scheduler {
    /// Current active job (atomically swappable)
    current_job: Arc<ArcSwap<Option<MiningJob>>>,
    /// Atomic counter for nonce distribution
    nonce_counter: Arc<AtomicU64>,
    /// Channel for sending valid shares
    share_sender: Sender<Share>,
    /// Flag to control worker threads
    active: Arc<AtomicBool>,
    /// Number of nonces each worker processes per batch
    batch_size: u64,
}

impl Scheduler {
    /// Creates a new Scheduler instance
    ///
    /// # Arguments
    /// * `share_sender` - Channel for sending valid shares
    /// * `batch_size` - Number of nonces each worker processes at once
    pub fn new(share_sender: Sender<Share>, batch_size: u64) -> Self {
        Scheduler {
            current_job: Arc::new(ArcSwap::from_pointee(None)),
            nonce_counter: Arc::new(AtomicU64::new(0)),
            share_sender,
            active: Arc::new(AtomicBool::new(true)),
            batch_size,
        }
    }

    /// Updates the current mining job
    ///
    /// # Arguments
    /// * `new_job` - The new job to replace the current one
    pub fn update_job(&self, new_job: MiningJob) {
        self.current_job.store(Arc::new(Some(new_job)));
        self.nonce_counter.store(0, Ordering::SeqCst);
    }

    /// Starts the mining process with the given algorithm
    ///
    /// # Arguments
    /// * `algorithm` - The mining algorithm to use
    /// * `workers` - Number of worker threads to spawn
    pub fn start_mining(&self, algorithm: Arc<dyn Algorithm + Send + Sync>, workers: usize) {
        (0..workers).for_each(|_| {
            let job_arc = self.current_job.clone();
            let nonce_ctr = self.nonce_counter.clone();
            let sender = self.share_sender.clone();
            let active = self.active.clone();
            let batch = self.batch_size;
            let algo = algorithm.clone();

            std::thread::spawn(move || {
                while active.load(Ordering::Relaxed) {
                    let current_job = job_arc.load();
                    if let Some(job) = &**current_job {
                        let start_nonce = nonce_ctr.fetch_add(batch, Ordering::SeqCst);
                        (start_nonce..start_nonce + batch)
                            .into_par_iter()
                            .for_each(|nonce| match algo.hash(&job.blob, nonce) {
                                Ok(hash) => {
                                    if hash.as_ref() < job.target.as_slice() {
                                        let _ = sender.send(Share {
                                            job_id: job.job_id.clone(),
                                            nonce,
                                            result: hash,
                                        });
                                    }
                                }
                                Err(e) => log::error!("Hashing failed: {}", e),
                            });
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            });
        });
    }

    /// Stops all mining workers
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}
