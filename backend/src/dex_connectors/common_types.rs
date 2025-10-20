use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_address: String,
    pub token_symbol: String,
    pub token_name: String,
    pub balance: Decimal,
    pub decimals: u8,
    pub usd_value: Option<Decimal>,
}

/// Wallet balance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub wallet_address: String,
    pub native_balance: Decimal, // ETH, BNB, SOL
    pub native_symbol: String,
    pub tokens: Vec<TokenBalance>,
    pub total_usd_value: Decimal,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

/// Liquidity pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pool_address: String,
    pub token0: Token,
    pub token1: Token,
    pub reserve0: Decimal,
    pub reserve1: Decimal,
    pub total_liquidity: Decimal,
}

/// Swap quote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: Token,
    pub to_token: Token,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub price_impact: Decimal,
    pub minimum_received: Decimal,
    pub route: Vec<String>, // Token addresses in the swap route
    pub estimated_gas: String,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction_hash: String,
    pub status: TransactionStatus,
    pub block_number: Option<u64>,
    pub gas_used: Option<String>,
    pub timestamp: Option<i64>,
}
