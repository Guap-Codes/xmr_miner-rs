# XMR_Miner-RS
A High-performance Monero mining tool developed in Rust.

## ⚠️ Disclaimer
 This project is under active development. APIs, configuration formats, and command-line options may change without notice. CryptoNight variants (V7/R) are deprecated and will be removed in a future release.


## Introduction

XMR Miner RS is a high-performance Monero miner written in Rust. Its goals are:

* CPU-optimized RandomX implementation for modern Monero mining

* (Legacy) support for CryptoNight V7/R for historical or forked chains—deprecated

* Pluggable architecture: you can add or swap proof-of-work algorithms

* Native support for both pool and solo (node) mining modes

* Built-in statistics reporting (hashrate, CPU usage, temperature, etc.)

* Command-line interface powered by Clap, with start, benchmark, and config subcommands

    Note: CryptoNight V7 and CryptoNight R are no longer used by Monero mainnet (post-2019) and are scheduled for removal in a future release. You should prefer RandomX for any modern Monero mining.

## Features

   * RandomX (CPU):

        Full RandomX context initialization (2080 MB RAM in fast mode)

        Thread-local hashers to avoid shared-state bottlenecks

        verify() function to check solutions against a difficulty target

   * CryptoNight (Legacy) ― DEPRECATED:

        V7 (variant 1) and R (variant 4) support (only for legacy or custom forks)

   * Mining Modes:

        Pool (Stratum over WebSocket)

        Node (RPC-based block template / submission)

   * Statistics & Reporting:

        Per‐second hardware stats: CPU usage, temperature, memory usage

        Hash / share counters, accept/reject rates

        Configurable reporting interval

   * Benchmark Mode:

        Quick per-algorithm CPU benchmarking

        Per‐thread H/s logs (optional, at DEBUG level)

   * Config Generation:

        config subcommand to emit a config.toml template for pool or node

   * Extensible Architecture:

        Algorithm trait for easy addition of new PoW algorithms

        Modular code: miner, network, stats, utils, cli, config


## Requirements & Dependencies

 -  Rust toolchain: 1.60+ (nightly not required)

 -  C compiler / linker (for building dependencies, e.g. openssl, libclang for RandomX)

 -  System libraries (Ubuntu/Debian example):

```bash
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev git cmake
    sudo apt-get install -y libhwloc-dev libnuma-dev  # if you use advanced CPU pinning
```

Rust crates (pulled via Cargo.toml):
        
   - rust_randomx (RandomX implementation)
        
   - cryptonight (legacy CryptoNight hashing)

   - tokio, tokio-tungstenite, futures (WebSocket pool client)

   - serde, serde_json, toml (Configuration parsing & JSON)

   - crossbeam-channel (Thread communication)

   - sysinfo (Hardware stats)

   - clap (Command-line parsing)

   - env_logger, log (Logging)

   - thiserror (Custom error types)

   - hex, hex-literal (Hex encoding/decoding)

## Installation

 Clone the repository
```bash
git clone https://github.com/guap-codes/xmr_miner-rs.git
cd xmr_miner-rs
```
Install Rust (if not already installed)
```bash
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
```

Build in release mode
```bash
cargo build --release
```

The resulting binary will be located at ./target/release/xmr_miner-rs.

(Optional) Install to $PATH
```bash
    sudo cp target/release/xmr_miner-rs /usr/local/bin/
```

## Configuration

Configuration is handled via a TOML file (default: config.toml). You can generate a template using:
```bash
./xmr_miner-rs config --output config.toml --pool

or

./xmr_miner-rs config --output config.toml --node
```

### General Options
```toml
[general]
# Supported algorithms: randomx, cryptonight-v7 (deprecated), cryptonight-r (deprecated)
algorithm = "randomx"

# Number of CPU threads to use for mining (0 = auto-detect / 1 thread per logical CPU)
worker_threads = 0

# Nonce batch size per worker (how many nonces each worker picks up at once)
batch_size = 1000
```
algorithm: The PoW algorithm to run.
     -   randomx (default, recommended)
     -   cryptonight-v7 (legacy; deprecated)
     -   cryptonight-r (legacy; deprecated)

worker_threads: Number of threads. If set to 0, the miner will use num_cpus::get().

batch_size: How many nonces each thread fetches in one go (tunable for performance within pools).

### Mining Modes: Pool & Node
* Pool Mining
```toml
[mode.pool]
url       = "stratum+tcp://pool.example.com:3333"
user      = "YOUR_MONERO_ADDRESS"
password  = "x"                    # usually "x" or worker-specific
worker_id = "worker01"             # optional string for pool identification
```
 url: Stratum endpoint, e.g. stratum+tcp://pool.supportxmr.com:443 (TLS) or ws://….

 user: Your wallet address (and optional .worker suffix).

 password: Pool password (often “x” or blank).

 worker_id: Arbitrary label for this worker (max 32 chars).

* Node (Solo) Mining
```toml
[mode.node]
rpc_url       = "http://localhost:18081/json_rpc"
rpc_user      = "username"        # if your node has RPC auth
rpc_password  = "password"
wallet_address = "YOUR_MONERO_ADDRESS"
```
rpc_url: Your Monero node’s JSON RPC endpoint.

rpc_user & rpc_password: Credentials if RPC is locked.

wallet_address: Address to which mined blocks should award coinbase outputs.

* Sample config.toml
```toml
# XMR Miner Configuration

[general]
algorithm       = "randomx"
worker_threads  = 0
batch_size      = 1000

# Pool mining configuration (uncomment if using pool)
[mode.pool]
url       = "stratum+tcp://pool.supportxmr.com:443"
user      = "42...YourPublicAddress..."
password  = "x"
worker_id = "rust-worker-01"

# Node mining configuration (uncomment if using solo)
#[mode.node]
#rpc_url        = "http://127.0.0.1:18081/json_rpc"
#rpc_user       = "monero-rpc"
#rpc_password   = "mysecret"
#wallet_address = "42...YourPublicAddress..."
```
Only one of [mode.pool] or [mode.node] should be uncommented.

If both are present, pool mode takes precedence.


## Command-Line Usage

Run `./xmr_miner-rs --help` to see a summary. High-level syntax:
```bash
xmr_miner-rs <SUBCOMMAND> [OPTIONS]
```

### start Subcommand

Start mining (pool or node) based on your config.toml.
```bash
xmr_miner-rs start [OPTIONS]
```
Options:

    -c, --config <FILE>  Path to configuration file (default: config.toml)

    -w, --workers <N>   Overrides worker_threads in config

    -a, --algorithm <ALGORITHM> Overrides algorithm (one of randomx, cryptonight-v7, cryptonight-r)

Example (pool mining):
```bash
xmr_miner-rs start \
  --config myconfig.toml \
  --workers 4 \
  --algorithm randomx
```

### benchmark Subcommand

Quickly benchmark a PoW algorithm on your CPU. Does not connect to pools or nodes.
```bash
xmr_miner-rs benchmark -a <ALGORITHM> [OPTIONS]
```
Options:

    -a, --algorithm <ALGORITHM> Algorithm to benchmark (randomx, cryptonight-v7, cryptonight-r)

    -d, --duration <SECONDS> Length of benchmark run (default: 60)

    -t, --threads <N>  Number of threads (default: number of logical CPUs)

Example:
```bash
# Short 10-second RandomX benchmark on all CPU threads
xmr_miner-rs benchmark --algorithm randomx --duration 10

# CryptoNight-V7 benchmark on 4 threads for 5 seconds
xmr_miner-rs benchmark --algorithm cryptonight-v7 --duration 5 --threads 4
```
Logging:
    - INFO-level prints “Starting …” and “Benchmark results …” only
    - DEBUG-level prints per-thread H/s every second

Enable INFO only (no per-thread logs):
```bash
RUST_LOG=info xmr_miner-rs benchmark --algorithm randomx --duration 10
```

### config Subcommand

Generate a config.toml template.
```bash
xmr_miner-rs config [OPTIONS]
```
Options:

    -o, --output <FILE>  Path to write the generated template (default: config.toml)

    --pool       Include pool mining section

    --node      Include node mining section

Examples:
```bash
# Generate a pool-only template
xmr_miner-rs config --output pool-config.toml --pool

# Generate both pool & node sections
xmr_miner-rs config --output full-config.toml --pool --node
```

## Algorithms

### RandomX (current default)

* Purpose: Current Monero PoW since November 2019.

* Characteristics:

   CPU‐optimized, memory‐hard (“scratchpad” ~2 MiB)

   JIT compilation of code to resist GPU/ASIC

   Requires ~2080 MB RAM (fast mode) or ~256 MB (light mode)

   Behavior in this Miner:
       - RandomX::new(fast: bool, key: &[u8]) builds a Context (dataset) and Hasher.
       - hash(input, nonce) returns a 32-byte output.
       - verify(input, nonce, target) checks if hash < target.

See `rust_randomx` docs for implementation details.

### CryptoNight V7/R (deprecated)

⚠️ CryptoNight V7 (variant 1) and CryptoNight R (variant 4) are deprecated. Monero stopped using these variants in late 2019, and they will be removed from this project in a future release.

 Variant 1 (V7): Monero’s PoW from March 2018 to March 2019.

 Variant 4 (R): Used briefly during the V8/V9 transition (April – October 2019) as an intermediate step.

  * Implementation:

       Based on the cryptonight crate

       CryptoNightAlgo::new(1) or new(4) selects the variant.

       hash(input, nonce) appends nonce (little-endian) and calls cryptonight(data, len, variant).

       verify(input, nonce, target) does hash < target.

Use V7/R only if you need to mine or verify blocks from legacy Monero forks (pre-RandomX). Otherwise, switch to RandomX.

## Statistics & Reporting

StatsReporter (stats::StatsReporter) manages:

   - MiningStats (hash count, shares accepted/rejected, avg hashrate)

   - HardwareStats (CPU usage, memory usage, temperature for CPU heat sensors)

By default, when you call:
```rust
let reporter = StatsReporter::new(Duration::from_secs(60));
reporter.start_reporting();
```
   A background thread wakes every 60 seconds.

   It logs:

   Hashrate: 1234.56 H/s | Accepted/Rejected: 10/0 | CPU: 12.3% | Temp: 45.1°C

   The miner’s scheduler and share‐receiver threads feed counts into the reporter via channels.

For benchmark mode, you can enable reporting every 5 seconds:
```rust
let mut reporter = StatsReporter::new(Duration::from_secs(5));
reporter.start_reporting();
```

Then you’ll see both per-thread DEBUG logs and periodic INFO stats.


## Project Structure
```bash
xmr_miner_rs/
├── Cargo.toml
├── README.md
├── src/
│   ├── cli/
│   │   ├── commands.rs         # Clap definitions (Commands, Action, Options, AlgorithmType)
│   │   └── mod.rs
│   ├── config/
│   │   ├── config.rs           # Config struct + load/generate_template
│   │   └── mod.rs
│   ├── lib.rs                  # Re-exports: Scheduler, Algorithm, PoolClient, StatsReporter, etc.
│   ├── main.rs                 # CLI entrypoint, subcommand matching (start, benchmark, config)
│   ├── types.rs 
│   ├── miner/
│   │   ├── algorithm/
│   │   │   ├── cryptonight.rs  # Deprecated CryptoNight V7/R
│   │   │   ├── randomx.rs      # Current RandomX implementation
│   │   │   └── mod.rs
│   │   ├── scheduler.rs        # Job scheduling, worker threads, hashing loop
│   │   └── worker.rs           # Worker state, job dispatch
│   ├── network/
│   │   ├── pool.rs             # Pool client (WebSocket, Stratum, share submission)
│   │   ├── node.rs             # Node client (RPC, block templates, share submission)
│   │   └── mod.rs
│   ├── stats/
│   │   ├── reporter.rs         # StatsReporter, MiningStats, HardwareStats
│   │   └── mod.rs
│   └── utils/
│       ├── error.rs            # MinerError enum (From Hex, I/O, JSON, etc.)
│       ├── logging.rs          # `init_logging()` & `init_bench_logging()` with env_logger
│       └── mod.rs
└── ...
```

   * cli/commands.rs: Defines Commands, Action (Start, Benchmark, Config), option structs, and the CLI-facing AlgorithmType enum (with ValueEnum for Clap).

   * config/config.rs: Holds the Config struct, load(), and generate_template().

   * miner/algorithm/:

        randomx.rs: Active RandomX implementation (Context + Hasher).

        cryptonight.rs: Legacy CryptoNight V7/R (marked deprecated).

   * miner/scheduler.rs: Orchestrates worker threads, job distribution (either from the pool or generated by Proof-of-Work).

   * miner/worker.rs: Worker‐level logic, hashing loops, share submission requests.

   * network/:

        pool.rs: Handles Stratum over WebSocket, JSON‐RPC for share submission, keep‐alive, job parsing.

        node.rs: Manages RPC calls (get_block_template, submit_block), chain monitoring for solo mining.

   * stats/reporter.rs: Gathers CPU/memory/temperature via sysinfo, collects hash & share counts via crossbeam_channel, logs periodic stats.

   * utils/:

        error.rs: Centralized MinerError using thiserror with From impls for I/O, JSON, WebSocket, hex, channel errors.

        logging.rs: Sets up env_logger formatting for INFO/DEBUG messages.


## Contributing

 Clone the repository and create a new branch for your feature or bugfix:
```bash
git clone https://github.com/guap-codes/xmr_miner-rs.git
cd xmr_miner-rs
git checkout -b feature/your-feature
```
Ensure tests pass before submitting a PR:
```bash
    cargo test
```
   Follow code style and linting:

   - Use `rustfmt` (the project’s `.rustfmt.toml` is auto-populated).

   - Adhere to the existing error‐handling style (`thiserror`).

   - Keep public APIs documented with doc-comments.

   - Open a pull request against `main` or `develop` (depending on the project workflow).

   - CI checks will run `cargo build`, `cargo test`, and `clippy`.

Notes for new contributors:

  - Feel free to discuss major architectural changes via GitHub Issues first.

  - The CryptoNight implementation is deprecated—do not introduce new CryptoNight features.

  - RandomX core improvements (e.g. GPU offload, further threading optimizations) are welcome.

## License

This project is licensed under the MIT License. See LICENSE for details.

Thank you for using and contributing to XMR_Miner-RS!
