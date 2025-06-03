// src/cli/commands.rs
use crate::types::AlgorithmType;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// XMR Miner CLI - Monero mining implementation in Rust
#[derive(Parser, Debug)]
#[command(name = "xmr-miner-rs")]
#[command(version, about, long_about = None)]
pub struct Commands {
    /// The action to perform (start mining, run benchmarks, or generate config)
    #[command(subcommand)]
    pub action: Action,
}

/// Top-level commands for the miner application
#[derive(Subcommand, Debug)]
pub enum Action {
    /// Start mining operation with specified options
    Start(StartOptions),

    /// Run performance benchmarks for mining algorithms
    Benchmark(BenchmarkOptions),

    /// Generate configuration file template
    Config(ConfigOptions),
}

/// Options for starting the mining operation
#[derive(Parser, Debug)]
pub struct StartOptions {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Number of worker threads to use (overrides config)
    #[arg(short, long)]
    pub workers: Option<usize>,

    /// Mining algorithm to use (overrides config)
    #[arg(short, long)]
    pub algorithm: Option<AlgorithmType>,
}

/// Options for running mining benchmarks
#[derive(Parser, Debug)]
pub struct BenchmarkOptions {
    /// Algorithm to benchmark
    #[arg(short, long)]
    pub algorithm: AlgorithmType,

    /// Duration of benchmark in seconds
    #[arg(short, long, default_value_t = 60)]
    pub duration: u64,

    /// Number of threads to use
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub threads: usize,
}

/// Options for generating configuration files
#[derive(Parser, Debug)]
pub struct ConfigOptions {
    /// Output file path
    #[arg(short, long, default_value = "config.toml")]
    pub output: PathBuf,

    /// Include pool mining configuration template
    #[arg(short, long)]
    pub pool: bool,

    /// Include node mining configuration template
    #[arg(short, long)]
    pub node: bool,
}
