// src/miner/worker.rs
//! Worker thread implementation
//!
//! Handles the actual mining work by processing assigned nonce ranges
//! using the specified algorithm. Reports found shares back to the scheduler.

use crate::miner::algorithm::Algorithm;
use crate::miner::scheduler::{MiningJob, Share};
use crossbeam_channel::Sender;
use rayon::prelude::*;
use std::sync::Arc;

/// Worker thread that performs mining computations
///
/// Each worker is responsible for processing a specific range of nonces
/// using the assigned algorithm and reporting any valid shares found.
pub struct Worker<A: Algorithm> {
    /// The mining algorithm implementation to use
    algorithm: Arc<A>,
    /// Current mining job being processed
    job: Option<Arc<MiningJob>>,
    /// Starting nonce for this worker's range
    nonce_start: u64,
    /// Ending nonce for this worker's range
    nonce_end: u64,
    /// Channel for sending valid shares back to scheduler
    share_sender: Sender<Share>,
}

impl<A: Algorithm + Send + Sync> Worker<A> {
    /// Creates a new Worker instance
    ///
    /// # Arguments
    /// * `algorithm` - The mining algorithm to use
    /// * `share_sender` - Channel for sending found shares
    pub fn new(algorithm: Arc<A>, share_sender: Sender<Share>) -> Self {
        Worker {
            algorithm,
            job: None,
            nonce_start: 0,
            nonce_end: 0,
            share_sender,
        }
    }

    /// Updates the worker's current job and nonce range
    ///
    /// # Arguments
    /// * `new_job` - The new mining job to process
    /// * `start` - Starting nonce for this worker
    /// * `end` - Ending nonce for this worker (exclusive)
    pub fn update_job(&mut self, new_job: Arc<MiningJob>, start: u64, end: u64) {
        self.job = Some(new_job);
        self.nonce_start = start;
        self.nonce_end = end;
    }

    /// Starts processing the assigned nonce range
    ///
    /// Uses rayon's parallel iterator to efficiently process the nonce range.
    /// Automatically sends any valid shares found through the share channel.
    pub fn run(&self) {
        if let Some(job) = &self.job {
            (self.nonce_start..self.nonce_end)
                .into_par_iter()
                .for_each(|nonce| match self.algorithm.hash(&job.blob, nonce) {
                    Ok(hash) => {
                        if hash.as_ref() < job.target.as_slice() {
                            let _ = self.share_sender.send(Share {
                                job_id: job.job_id.clone(),
                                nonce,
                                result: hash,
                            });
                        }
                    }
                    Err(e) => log::error!("Hashing error: {}", e),
                });
        }
    }
}
