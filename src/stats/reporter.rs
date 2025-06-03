// src/stats/reporter.rs
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use sysinfo::{Components, System};
//use crate::miner::scheduler::Share;
//use crate::utils::error::MinerError;

/// Statistics related to mining performance
#[derive(Debug, Clone, Default)]
pub struct MiningStats {
    /// Total number of hashes computed
    pub hashes_total: u64,
    /// Number of shares accepted by the mining pool/node
    pub shares_accepted: u64,
    /// Number of shares rejected by the mining pool/node
    pub shares_rejected: u64,
    /// Average hashrate over 1 minute (hashes per second)
    pub avg_hashrate_1m: f64,
    /// Average hashrate over 15 minutes (hashes per second)
    pub avg_hashrate_15m: f64,
}

/// Statistics related to hardware performance
#[derive(Debug, Clone)]
pub struct HardwareStats {
    /// Current CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory currently used by the process (in bytes)
    pub memory_used: u64,
    /// Current CPU temperature in Celsius
    pub temperature: f32,
}

/// Collects and reports mining and hardware statistics
pub struct StatsReporter {
    /// Atomic counters for mining statistics
    stats: Arc<MiningStatsAtomic>,
    /// System information collector
    system: System,
    /// Hardware component information collector
    components: Components,
    /// Interval at which stats are reported
    report_interval: Duration,
}

/// Atomic version of MiningStats for thread-safe operations
struct MiningStatsAtomic {
    hashes: AtomicU64,
    accepted: AtomicU64,
    rejected: AtomicU64,
    start_time: Instant,
}

impl Clone for StatsReporter {
    fn clone(&self) -> Self {
        StatsReporter {
            stats: self.stats.clone(),
            system: System::new_all(),
            components: Components::new_with_refreshed_list(),
            report_interval: self.report_interval,
        }
    }
}

impl StatsReporter {
    /// Creates a new StatsReporter with the specified reporting interval
    ///
    /// # Arguments
    /// * `report_interval` - How often to log statistics
    pub fn new(report_interval: Duration) -> Self {
        StatsReporter {
            stats: Arc::new(MiningStatsAtomic {
                hashes: AtomicU64::new(0),
                accepted: AtomicU64::new(0),
                rejected: AtomicU64::new(0),
                start_time: Instant::now(),
            }),
            system: System::new_all(),
            components: Components::new_with_refreshed_list(),
            report_interval,
        }
    }

    /// Creates and returns a channel sender for share results
    ///
    /// The returned sender can be used to report accepted/rejected shares.
    /// The reporter will automatically listen for these events on a background thread.
    pub fn share_sender(&self) -> Sender<ShareResult> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.start_share_listener(rx);
        tx
    }

    /// Creates and returns a channel sender for hash counts
    ///
    /// The returned sender can be used to report completed hashes.
    /// The reporter will automatically listen for these events on a background thread.
    pub fn hash_sender(&self) -> Sender<u64> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.start_hashrate_listener(rx);
        tx
    }

    /// Gets the current mining statistics
    ///
    /// # Returns
    /// A snapshot of the current mining statistics
    pub fn get_stats(&self) -> MiningStats {
        let total_seconds = self.stats.start_time.elapsed().as_secs() as f64;
        let hashes = self.stats.hashes.load(Ordering::Relaxed);

        MiningStats {
            hashes_total: hashes,
            shares_accepted: self.stats.accepted.load(Ordering::Relaxed),
            shares_rejected: self.stats.rejected.load(Ordering::Relaxed),
            avg_hashrate_1m: hashes as f64 / total_seconds.max(60.0) * 60.0,
            avg_hashrate_15m: hashes as f64 / total_seconds.max(900.0) * 900.0,
        }
    }

    /// Gets the current hardware statistics
    ///
    /// This refreshes system information before returning the stats.
    ///
    /// # Returns
    /// A snapshot of the current hardware statistics
    pub fn get_hardware_stats(&mut self) -> HardwareStats {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.components.refresh(true);

        let cpu_usage = self
            .system
            .cpus()
            .iter()
            .map(|c| c.cpu_usage())
            .sum::<f32>()
            / self.system.cpus().len() as f32;

        let temperature = self
            .components
            .iter()
            .find(|c| c.label().contains("CPU"))
            .and_then(|c| c.temperature())
            .unwrap_or(0.0);

        HardwareStats {
            cpu_usage,
            memory_used: self.system.used_memory(),
            temperature,
        }
    }

    /// Starts the periodic reporting of statistics
    ///
    /// This spawns a background thread that logs stats at the configured interval.
    pub fn start_reporting(&self) {
        let stats = self.stats.clone();
        let interval = self.report_interval;

        std::thread::spawn(move || {
            let mut reporter = StatsReporter {
                stats,
                system: System::new_all(),
                components: Components::new_with_refreshed_list(),
                report_interval: interval,
            };

            loop {
                std::thread::sleep(interval);
                let mining_stats = reporter.get_stats();
                let hw_stats = reporter.get_hardware_stats();

                log::info!(
                    "Hashrate: {:.2} H/s | Accepted/Rejected: {}/{} | CPU: {:.1}% | Temp: {:.1}°C",
                    mining_stats.avg_hashrate_1m,
                    mining_stats.shares_accepted,
                    mining_stats.shares_rejected,
                    hw_stats.cpu_usage,
                    hw_stats.temperature
                );
            }
        });
    }

    /// Starts a listener for share results on a background thread
    fn start_share_listener(&self, receiver: Receiver<ShareResult>) {
        let stats = self.stats.clone();

        std::thread::spawn(move || {
            for result in receiver {
                match result {
                    ShareResult::Accepted => stats.accepted.fetch_add(1, Ordering::Relaxed),
                    ShareResult::Rejected => stats.rejected.fetch_add(1, Ordering::Relaxed),
                };
            }
        });
    }

    /// Starts a listener for hash counts on a background thread
    fn start_hashrate_listener(&self, receiver: Receiver<u64>) {
        let stats = self.stats.clone();

        std::thread::spawn(move || {
            for count in receiver {
                stats.hashes.fetch_add(count, Ordering::Relaxed);
            }
        });
    }
}

/// Result of submitting a share to the mining pool/node
#[derive(Debug, Clone, Copy)]
pub enum ShareResult {
    /// The share was accepted as valid
    Accepted,
    /// The share was rejected (likely invalid)
    Rejected,
}
