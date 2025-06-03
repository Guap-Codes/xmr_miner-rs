// src/main.rs
use crate::miner::algorithm::{cryptonight::CryptoNightAlgo, randomx::RandomX};
use crate::types::AlgorithmType;
use crate::utils::logging::init_bench_logging;
use clap::Parser;
use crossbeam_channel::unbounded;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use xmr_miner_rs::{self, *};

/// Main entry point for XMR miner
///
/// # Returns
/// - `Ok(())` on successful execution
/// - `Err(MinerError)` if any operation fails
///
/// # Flow
/// 1. Parses command line arguments
/// 2. Delegates to appropriate subcommand handler
/// 3. Propagates any errors upward
fn main() -> Result<(), MinerError> {
    let cli = cli::Commands::parse();

    match cli.action {
        cli::Action::Start(opts) => start_mining(opts),
        cli::Action::Benchmark(opts) => run_benchmark(opts),
        cli::Action::Config(opts) => generate_config(opts),
    }
}

/// Starts the mining operation with given configuration options
///
/// # Arguments
/// * `opts` - Command line options for mining operation
///
/// # Operations
/// 1. Initializes logging
/// 2. Loads and validates configuration
/// 3. Sets up statistics reporting
/// 4. Initializes mining scheduler
/// 5. Connects to pool/node based on configuration
fn start_mining(opts: cli::StartOptions) -> Result<(), MinerError> {
    utils::init_logging();

    let mut config = config::load(&opts.config)?;
    // Apply CLI overrides
    if let Some(workers) = opts.workers {
        config.worker_threads = workers;
    }
    if let Some(algo) = opts.algorithm {
        config.algorithm = algo.to_string();
    }

    // Communication channels
    let (share_sender, share_receiver) = unbounded(); // For submitting shares
    let (job_sender, _job_receiver) = unbounded(); // For receiving work (receiver unused)

    // Statistics reporting
    let reporter = stats::StatsReporter::new(Duration::from_secs(60));
    reporter.start_reporting();

    // Mining setup
    let scheduler = miner::Scheduler::new(share_sender.clone(), config.batch_size);
    let algorithm = create_algorithm(&config)?;
    scheduler.start_mining(algorithm, config.worker_threads);

    // Runtime setup
    let rt = Runtime::new()?;
    rt.block_on(async {
        match config.mode {
            config::MiningMode::Pool(pool_cfg) => {
                let pool = network::PoolClient::new(pool_cfg, job_sender, share_receiver);
                pool.connect().await?;
                pool.run().await
            }
            config::MiningMode::Node(node_cfg) => {
                let mut node = network::NodeClient::new(node_cfg);
                node.monitor_chain().await
            }
        }
    })
}

/// Runs mining algorithm benchmarks
///
/// # Arguments
/// * `opts` - Benchmark configuration options
///
/// # Operations
/// 1. Initializes benchmark-specific logging
/// 2. Creates specified algorithm instance
/// 3. Spawns worker threads
/// 4. Collects and reports performance statistics
fn run_benchmark(opts: cli::BenchmarkOptions) -> Result<(), MinerError> {
    init_bench_logging();

    let algorithm = create_bench_algorithm(opts.algorithm)?;
    let reporter = stats::StatsReporter::new(Duration::from_secs(5));
    let hash_sender = reporter.hash_sender();

    log::info!(
        "Starting {} benchmark for {} seconds",
        opts.algorithm,
        opts.duration
    );
    log::logger().flush(); // Ensure final results appear

    let start_time = std::time::Instant::now();
    let handles: Vec<_> = (0..opts.threads)
        .map(|_| {
            let algo = algorithm.clone();
            let sender = hash_sender.clone();
            std::thread::spawn(move || {
                let mut nonce = 0;
                let mut last_log = std::time::Instant::now();
                let mut hashes = 0;

                while start_time.elapsed().as_secs() < opts.duration {
                    let _ = algo.hash(&[0u8; 76], nonce);
                    nonce += 1;
                    hashes += 1;
                    sender.send(1).unwrap();

                    // Log progress every second
                    if last_log.elapsed().as_secs() >= 1 {
                        log::debug!(
                            "Thread {:?}: {:.1} H/s",
                            std::thread::current().id(),
                            hashes as f64 / last_log.elapsed().as_secs_f64()
                        );
                        hashes = 0;
                        last_log = std::time::Instant::now();
                    }
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Report final results
    let stats = reporter.get_stats();
    log::info!("Benchmark results:");
    log::info!("Total hashes: {}", stats.hashes_total);
    log::info!("Average hashrate: {:.2} H/s", stats.avg_hashrate_1m);
    log::logger().flush(); // Ensure final results appear

    Ok(())
}

/// Generates configuration template file
///
/// # Arguments
/// * `opts` - Configuration generation options
///
/// # Operations
/// 1. Generates template content based on options
/// 2. Writes template to specified output file
fn generate_config(opts: cli::ConfigOptions) -> Result<(), MinerError> {
    let config = config::generate_template(opts.pool, opts.node);
    std::fs::write(opts.output, config)?;
    Ok(())
}

/// Creates algorithm instance based on configuration
///
/// # Arguments
/// * `config` - Mining configuration
///
/// # Returns
/// - `Ok(Arc<dyn Algorithm>)` on success
/// - `Err(MinerError)` if algorithm is invalid
fn create_algorithm(config: &config::Config) -> Result<Arc<dyn Algorithm>, MinerError> {
    // Parse string to AlgorithmType
    let algo_type = config
        .algorithm
        .parse()
        .map_err(|_| MinerError::ConfigError(format!("Invalid algorithm: {}", config.algorithm)))?;

    match algo_type {
        AlgorithmType::RandomX => {
            let temp_key = [0u8; 32]; // Placeholder until first job
            Ok(Arc::new(RandomX::new(
                true, // Use fast mode for mining
                &temp_key,
            )))
        }
        AlgorithmType::CryptoNightV7 => Ok(Arc::new(CryptoNightAlgo::new(1))),
        AlgorithmType::CryptoNightR => Ok(Arc::new(CryptoNightAlgo::new(4))),
    }
}

/// Creates algorithm instance for benchmarking
///
/// # Arguments
/// * `algo` - Algorithm type to benchmark
///
/// # Returns
/// - `Ok(Arc<dyn Algorithm>)` on success
/// - `Err(MinerError)` if algorithm is invalid
fn create_bench_algorithm(algo: AlgorithmType) -> Result<Arc<dyn Algorithm>, MinerError> {
    match algo {
        AlgorithmType::RandomX => {
            let temp_key = [0u8; 32];
            Ok(Arc::new(RandomX::new(
                true, // Use fast mode for mining
                &temp_key,
            )))
        }
        AlgorithmType::CryptoNightV7 => Ok(Arc::new(CryptoNightAlgo::new(1))),
        AlgorithmType::CryptoNightR => Ok(Arc::new(CryptoNightAlgo::new(4))),
    }
}
