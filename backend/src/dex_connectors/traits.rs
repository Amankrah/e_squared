use async_trait::async_trait;
use rust_decimal::Decimal;

use super::common_types::*;
use super::errors::DexError;

/// Core DEX connector trait
#[async_trait]
pub trait DexConnector: Send + Sync {
    /// Test the wallet connection
    async fn test_connection(&self) -> Result<bool, DexError>;

    /// Get wallet balance including native token and ERC20/BEP20 tokens
    async fn get_wallet_balance(&self) -> Result<WalletBalance, DexError>;

    /// Get balance for a specific token
    async fn get_token_balance(&self, token_address: &str) -> Result<TokenBalance, DexError>;

    /// Get quote for a swap
    async fn get_swap_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
    ) -> Result<SwapQuote, DexError>;

    /// Execute a swap transaction
    async fn execute_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError>;

    /// Get liquidity pool information
    async fn get_pool_info(
        &self,
        token0: &str,
        token1: &str,
    ) -> Result<LiquidityPool, DexError>;

    /// Add liquidity to a pool
    async fn add_liquidity(
        &self,
        token0: &str,
        token1: &str,
        amount0: Decimal,
        amount1: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError>;

    /// Remove liquidity from a pool
    async fn remove_liquidity(
        &self,
        token0: &str,
        token1: &str,
        liquidity_amount: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError>;

    /// Get transaction status
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, DexError>;

    /// Get current gas price
    async fn get_gas_price(&self) -> Result<String, DexError>;

    /// Get DEX name
    fn dex_name(&self) -> &'static str;

    /// Get blockchain network
    fn blockchain_network(&self) -> &'static str;
}
