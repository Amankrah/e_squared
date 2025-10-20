pub mod traits;
pub mod common_types;
pub mod errors;
pub mod factory;
pub mod uniswap;
pub mod pancakeswap;
pub mod raydium;
pub mod jupiter;

pub use traits::*;
pub use common_types::*;
pub use errors::*;
pub use factory::*;

use serde::{Deserialize, Serialize};

/// Supported DEX platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DEX {
    // EVM DEXs
    Uniswap,
    PancakeSwap,
    // Solana DEXs
    Raydium,
    Jupiter,
}

impl DEX {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "uniswap" => Some(Self::Uniswap),
            "pancakeswap" | "pancake" => Some(Self::PancakeSwap),
            "raydium" => Some(Self::Raydium),
            "jupiter" | "jup" => Some(Self::Jupiter),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uniswap => "uniswap",
            Self::PancakeSwap => "pancakeswap",
            Self::Raydium => "raydium",
            Self::Jupiter => "jupiter",
        }
    }

    pub fn blockchain_network(&self) -> &'static str {
        match self {
            Self::Uniswap => "ethereum",
            Self::PancakeSwap => "bnbchain",
            Self::Raydium => "solana",
            Self::Jupiter => "solana",
        }
    }
}

/// Wallet credentials for DEX operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCredentials {
    pub private_key: String,
    pub wallet_address: String,
}
