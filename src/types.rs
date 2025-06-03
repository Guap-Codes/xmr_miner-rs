// src/types.rs
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported mining algorithms for XMR mining
///
/// This enum represents the different proof-of-work algorithms
/// that can be used for Monero mining, each with different
/// performance characteristics and hardware requirements.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum AlgorithmType {
    /// RandomX algorithm (CPU-optimized, ASIC-resistant)
    ///
    /// Uses random code execution and memory-hard techniques.
    /// Recommended for modern CPUs with large caches.
    #[clap(name = "randomx")]
    RandomX,

    /// CryptoNight variant 7 algorithm (legacy)
    ///
    /// Earlier version of Monero's PoW algorithm.
    /// Less memory intensive than RandomX but also less secure.
    #[clap(name = "cryptonight-v7")]
    CryptoNightV7,

    /// CryptoNight-R algorithm (legacy)
    ///
    /// Modified version of CryptoNight with small tweaks.
    /// Used during Monero's algorithm transition period.
    #[clap(name = "cryptonight-r")]
    CryptoNightR,
}

impl fmt::Display for AlgorithmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlgorithmType::RandomX => write!(f, "randomx"),
            AlgorithmType::CryptoNightV7 => write!(f, "cryptonight-v7"),
            AlgorithmType::CryptoNightR => write!(f, "cryptonight-r"),
        }
    }
}

impl FromStr for AlgorithmType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "randomx" => Ok(AlgorithmType::RandomX),
            "cnv7" | "cryptonight-v7" => Ok(AlgorithmType::CryptoNightV7),
            "cnr" | "cryptonight-r" => Ok(AlgorithmType::CryptoNightR),
            _ => Err(format!("Unknown algorithm: {}", s)),
        }
    }
}
