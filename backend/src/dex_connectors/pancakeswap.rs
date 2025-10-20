use async_trait::async_trait;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, H256, U256};
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;

use super::common_types::*;
use super::errors::DexError;
use super::traits::DexConnector;
use super::WalletCredentials;

// PancakeSwap V3 Contract Addresses on BNB Chain (BSC) Mainnet
const SMART_ROUTER: &str = "0x13f4EA83D0bd40E75C8222255bc855a974568Dd4";
const SWAP_ROUTER: &str = "0x1b81D678ffb9C0263b24A97847620C99d213eB14";
const QUOTER_V2: &str = "0xB048Bbc1Ee6b733FFfCFb9e9CeF7375518e25997";
const FACTORY: &str = "0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865";
const WBNB: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";

// Fee tiers for PancakeSwap V3 pools
const FEE_LOW: u32 = 500; // 0.05%
const FEE_MEDIUM: u32 = 2500; // 0.25%
const FEE_HIGH: u32 = 10000; // 1%

/// PancakeSwap V3 connector for BNB Chain (BSC)
///
/// This connector supports:
/// - Token swaps via SmartRouter/SwapRouter
/// - Quote estimation via QuoterV2
/// - BNB and BEP20 balance queries
/// - Transaction status tracking
/// - Gas estimation
pub struct PancakeSwapConnector {
    credentials: WalletCredentials,
    provider: Arc<Provider<Http>>,
    wallet: LocalWallet,
    chain_id: u64,
}

impl PancakeSwapConnector {
    pub fn new(credentials: WalletCredentials) -> Result<Self, DexError> {
        // Get BSC RPC URL from environment or use default
        let rpc_url = std::env::var("BSC_RPC_URL")
            .unwrap_or_else(|_| "https://bsc-dataseed1.binance.org".to_string());

        // Create provider
        let provider = Provider::<Http>::try_from(rpc_url.as_str())
            .map_err(|e| DexError::NetworkError(format!("Failed to create provider: {}", e)))?;

        // Parse private key
        let wallet = Self::parse_private_key(&credentials.private_key)?;

        // Verify wallet address matches
        let derived_address = format!("{:?}", wallet.address());
        if derived_address.to_lowercase() != credentials.wallet_address.to_lowercase() {
            return Err(DexError::InvalidCredentials(
                "Wallet address does not match private key".to_string()
            ));
        }

        Ok(Self {
            credentials,
            provider: Arc::new(provider),
            wallet,
            chain_id: 56, // BSC mainnet
        })
    }

    fn parse_private_key(private_key: &str) -> Result<LocalWallet, DexError> {
        // Remove 0x prefix if present
        let key_str = private_key.strip_prefix("0x").unwrap_or(private_key);

        // Parse as hex
        if key_str.len() != 64 {
            return Err(DexError::InvalidCredentials(
                "Invalid BNB Chain private key length (expected 64 hex characters)".to_string()
            ));
        }

        LocalWallet::from_str(private_key)
            .map_err(|e| DexError::InvalidCredentials(format!("Invalid private key: {}", e)))
    }

    async fn get_bnb_balance(&self, address: Address) -> Result<U256, DexError> {
        self.provider
            .get_balance(address, None)
            .await
            .map_err(|e| DexError::NetworkError(format!("Failed to get BNB balance: {}", e)))
    }

    async fn get_bep20_balance(&self, token_address: Address, wallet_address: Address) -> Result<U256, DexError> {
        // BEP20 balanceOf(address) function selector (same as ERC20)
        let selector = &[0x70, 0xa0, 0x82, 0x31]; // balanceOf(address)

        // Encode the wallet address parameter
        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&[0u8; 12]); // Pad to 32 bytes
        calldata.extend_from_slice(wallet_address.as_bytes());

        let call = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
            ethers::types::TransactionRequest {
                to: Some(token_address.into()),
                data: Some(calldata.into()),
                ..Default::default()
            }
        );

        let result = self.provider
            .call(&call, None)
            .await
            .map_err(|e| DexError::NetworkError(format!("Failed to get token balance: {}", e)))?;

        // Decode U256 from bytes
        Ok(U256::from_big_endian(result.as_ref()))
    }

    async fn get_token_decimals(&self, token_address: Address) -> Result<u8, DexError> {
        // BEP20 decimals() function selector
        let selector = &[0x31, 0x3c, 0xe5, 0x67]; // decimals()

        let call = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
            ethers::types::TransactionRequest {
                to: Some(token_address.into()),
                data: Some(selector.to_vec().into()),
                ..Default::default()
            }
        );

        let result = self.provider
            .call(&call, None)
            .await
            .map_err(|e| DexError::NetworkError(format!("Failed to get token decimals: {}", e)))?;

        if result.len() >= 32 {
            Ok(result[31])
        } else {
            Ok(18) // Default to 18 decimals
        }
    }

    async fn get_quote_exact_input_single(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        fee: u32,
    ) -> Result<(U256, U256, U256), DexError> {
        // QuoterV2.quoteExactInputSingle parameters
        // struct QuoteExactInputSingleParams {
        //     address tokenIn;
        //     address tokenOut;
        //     uint256 amountIn;
        //     uint24 fee;
        //     uint160 sqrtPriceLimitX96;
        // }

        let quoter_address = Address::from_str(QUOTER_V2)
            .map_err(|_| DexError::InternalError("Invalid quoter address".to_string()))?;

        // Function selector for quoteExactInputSingle
        let selector = &[0xc6, 0xe2, 0x01, 0xf8]; // quoteExactInputSingle((address,address,uint256,uint24,uint160))

        let mut calldata = selector.to_vec();

        // Encode struct as tuple
        // offset to struct data
        calldata.extend_from_slice(&[0u8; 31]);
        calldata.push(0x20);

        // tokenIn (address - 32 bytes padded)
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token_in.as_bytes());

        // tokenOut (address - 32 bytes padded)
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token_out.as_bytes());

        // amountIn (uint256)
        let mut amount_bytes = [0u8; 32];
        amount_in.to_big_endian(&mut amount_bytes);
        calldata.extend_from_slice(&amount_bytes);

        // fee (uint24 - 32 bytes padded)
        calldata.extend_from_slice(&[0u8; 29]);
        calldata.extend_from_slice(&fee.to_be_bytes()[1..4]);

        // sqrtPriceLimitX96 (uint160 - set to 0 for no limit)
        calldata.extend_from_slice(&[0u8; 32]);

        let call = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
            ethers::types::TransactionRequest {
                to: Some(quoter_address.into()),
                data: Some(calldata.into()),
                ..Default::default()
            }
        );

        let result = self.provider
            .call(&call, None)
            .await
            .map_err(|e| DexError::NetworkError(format!("Failed to get quote: {}", e)))?;

        // Decode result: (amountOut, sqrtPriceX96After, initializedTicksCrossed, gasEstimate)
        if result.len() >= 128 {
            let amount_out = U256::from_big_endian(&result[0..32]);
            let sqrt_price = U256::from_big_endian(&result[32..64]);
            let gas_estimate = U256::from_big_endian(&result[96..128]);

            Ok((amount_out, sqrt_price, gas_estimate))
        } else {
            Err(DexError::InternalError("Invalid quote response".to_string()))
        }
    }

    async fn execute_swap_exact_input_single(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        amount_out_minimum: U256,
        fee: u32,
    ) -> Result<H256, DexError> {
        let router_address = Address::from_str(SWAP_ROUTER)
            .map_err(|_| DexError::InternalError("Invalid router address".to_string()))?;

        // First, approve the router to spend tokens if it's not BNB
        let wbnb_address = Address::from_str(WBNB)
            .map_err(|_| DexError::InternalError("Invalid WBNB address".to_string()))?;

        if token_in != wbnb_address {
            self.approve_token(token_in, router_address, amount_in).await?;
        }

        // exactInputSingle parameters
        // struct ExactInputSingleParams {
        //     address tokenIn;
        //     address tokenOut;
        //     uint24 fee;
        //     address recipient;
        //     uint256 amountIn;
        //     uint256 amountOutMinimum;
        //     uint160 sqrtPriceLimitX96;
        // }

        // Function selector for exactInputSingle
        let selector = &[0x04, 0xe4, 0x5a, 0xaf]; // exactInputSingle((address,address,uint24,address,uint256,uint256,uint160))

        let mut calldata = selector.to_vec();

        // Offset to struct data
        calldata.extend_from_slice(&[0u8; 31]);
        calldata.push(0x20);

        // tokenIn
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token_in.as_bytes());

        // tokenOut
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(token_out.as_bytes());

        // fee (uint24)
        calldata.extend_from_slice(&[0u8; 29]);
        calldata.extend_from_slice(&fee.to_be_bytes()[1..4]);

        // recipient (this wallet)
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(self.wallet.address().as_bytes());

        // amountIn
        let mut amount_in_bytes = [0u8; 32];
        amount_in.to_big_endian(&mut amount_in_bytes);
        calldata.extend_from_slice(&amount_in_bytes);

        // amountOutMinimum
        let mut amount_out_bytes = [0u8; 32];
        amount_out_minimum.to_big_endian(&mut amount_out_bytes);
        calldata.extend_from_slice(&amount_out_bytes);

        // sqrtPriceLimitX96 (0 for no limit)
        calldata.extend_from_slice(&[0u8; 32]);

        // Create and send transaction
        let tx = ethers::types::TransactionRequest {
            to: Some(router_address.into()),
            data: Some(calldata.into()),
            value: if token_in == wbnb_address { Some(amount_in) } else { None },
            ..Default::default()
        };

        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone().with_chain_id(self.chain_id));

        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| DexError::TransactionFailed(format!("Failed to send transaction: {}", e)))?;

        let receipt = pending_tx
            .await
            .map_err(|e| DexError::TransactionFailed(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| DexError::TransactionFailed("No receipt".to_string()))?;

        Ok(receipt.transaction_hash)
    }

    async fn approve_token(&self, token: Address, spender: Address, amount: U256) -> Result<(), DexError> {
        // BEP20 approve(address,uint256) function
        let selector = &[0x09, 0x5e, 0xa7, 0xb3]; // approve(address,uint256)

        let mut calldata = selector.to_vec();

        // spender address
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(spender.as_bytes());

        // amount
        let mut amount_bytes = [0u8; 32];
        amount.to_big_endian(&mut amount_bytes);
        calldata.extend_from_slice(&amount_bytes);

        let tx = ethers::types::TransactionRequest {
            to: Some(token.into()),
            data: Some(calldata.into()),
            ..Default::default()
        };

        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone().with_chain_id(self.chain_id));

        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| DexError::TransactionFailed(format!("Failed to approve token: {}", e)))?;

        pending_tx
            .await
            .map_err(|e| DexError::TransactionFailed(format!("Approval transaction failed: {}", e)))?;

        Ok(())
    }

    fn wei_to_decimal(wei: U256, decimals: u8) -> Decimal {
        let wei_str = wei.to_string();
        let wei_decimal = Decimal::from_str(&wei_str).unwrap_or(Decimal::ZERO);
        let divisor = Decimal::from(10u64.pow(decimals as u32));
        wei_decimal / divisor
    }

    fn decimal_to_wei(amount: Decimal, decimals: u8) -> U256 {
        let multiplier = Decimal::from(10u64.pow(decimals as u32));
        let wei_decimal = amount * multiplier;
        let wei_string = wei_decimal.to_string();
        let wei_str = wei_string.split('.').next().unwrap_or("0");
        U256::from_dec_str(wei_str).unwrap_or(U256::zero())
    }
}

#[async_trait]
impl DexConnector for PancakeSwapConnector {
    async fn test_connection(&self) -> Result<bool, DexError> {
        tracing::info!("Testing PancakeSwap connection for wallet: {}", self.credentials.wallet_address);

        match self.provider.get_block_number().await {
            Ok(block_number) => {
                tracing::info!("Connected to BNB Chain. Current block: {}", block_number);
                Ok(true)
            }
            Err(e) => {
                tracing::error!("PancakeSwap connection test failed: {:?}", e);
                Err(DexError::NetworkError(format!("Failed to connect: {}", e)))
            }
        }
    }

    async fn get_wallet_balance(&self) -> Result<WalletBalance, DexError> {
        tracing::info!("Fetching PancakeSwap wallet balance for: {}", self.credentials.wallet_address);

        let address = Address::from_str(&self.credentials.wallet_address)
            .map_err(|_| DexError::InvalidCredentials("Invalid wallet address".to_string()))?;

        // Get BNB balance
        let bnb_wei = self.get_bnb_balance(address).await?;
        let bnb_balance = Self::wei_to_decimal(bnb_wei, 18);

        // TODO: Query all BEP20 token balances
        // This would require maintaining a list of tokens or querying from an indexer

        Ok(WalletBalance {
            wallet_address: self.credentials.wallet_address.clone(),
            native_balance: bnb_balance,
            native_symbol: "BNB".to_string(),
            tokens: vec![],
            total_usd_value: Decimal::ZERO,
        })
    }

    async fn get_token_balance(&self, token_address: &str) -> Result<TokenBalance, DexError> {
        tracing::info!("Fetching BEP20 token balance for: {}", token_address);

        let token = Address::from_str(token_address)
            .map_err(|_| DexError::InvalidCredentials("Invalid token address".to_string()))?;

        let wallet = Address::from_str(&self.credentials.wallet_address)
            .map_err(|_| DexError::InvalidCredentials("Invalid wallet address".to_string()))?;

        let balance_wei = self.get_bep20_balance(token, wallet).await?;
        let decimals = self.get_token_decimals(token).await?;
        let balance = Self::wei_to_decimal(balance_wei, decimals);

        Ok(TokenBalance {
            token_address: token_address.to_string(),
            token_symbol: "UNKNOWN".to_string(),
            token_name: "Unknown Token".to_string(),
            balance,
            decimals,
            usd_value: None,
        })
    }

    async fn get_swap_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
    ) -> Result<SwapQuote, DexError> {
        tracing::info!("Getting PancakeSwap swap quote: {} {} -> {}", amount, from_token, to_token);

        let token_in = Address::from_str(from_token)
            .map_err(|_| DexError::InvalidCredentials("Invalid from_token address".to_string()))?;

        let token_out = Address::from_str(to_token)
            .map_err(|_| DexError::InvalidCredentials("Invalid to_token address".to_string()))?;

        // Get token decimals
        let decimals_in = self.get_token_decimals(token_in).await?;
        let amount_in_wei = Self::decimal_to_wei(amount, decimals_in);

        // Try different fee tiers to find the best quote
        let mut best_quote: Option<(U256, U256, u32)> = None;

        for fee in &[FEE_LOW, FEE_MEDIUM, FEE_HIGH] {
            match self.get_quote_exact_input_single(token_in, token_out, amount_in_wei, *fee).await {
                Ok((amount_out, _sqrt_price, gas_estimate)) => {
                    if best_quote.is_none() || amount_out > best_quote.unwrap().0 {
                        best_quote = Some((amount_out, gas_estimate, *fee));
                    }
                }
                Err(_) => continue, // Pool doesn't exist for this fee tier
            }
        }

        let (amount_out_wei, gas_estimate, best_fee) = best_quote
            .ok_or_else(|| DexError::PoolNotFound("No liquidity pool found for this pair".to_string()))?;

        let decimals_out = self.get_token_decimals(token_out).await?;
        let amount_out = Self::wei_to_decimal(amount_out_wei, decimals_out);

        // Calculate price impact (simplified - actual calculation would require pool reserves)
        let price_impact = Decimal::from_str("0.01").unwrap(); // 1% placeholder

        // Calculate minimum received with 0.5% slippage
        let slippage = Decimal::from_str("0.005").unwrap();
        let minimum_received = amount_out * (Decimal::ONE - slippage);

        Ok(SwapQuote {
            from_token: Token {
                address: from_token.to_string(),
                symbol: "UNKNOWN".to_string(),
                name: "Unknown".to_string(),
                decimals: decimals_in,
            },
            to_token: Token {
                address: to_token.to_string(),
                symbol: "UNKNOWN".to_string(),
                name: "Unknown".to_string(),
                decimals: decimals_out,
            },
            from_amount: amount,
            to_amount: amount_out,
            price_impact,
            minimum_received,
            route: vec![from_token.to_string(), to_token.to_string()],
            estimated_gas: gas_estimate.to_string(),
        })
    }

    async fn execute_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError> {
        tracing::info!(
            "Executing PancakeSwap swap: {} {} -> {} (slippage: {}%)",
            amount, from_token, to_token, slippage_tolerance
        );

        let token_in = Address::from_str(from_token)
            .map_err(|_| DexError::InvalidCredentials("Invalid from_token address".to_string()))?;

        let token_out = Address::from_str(to_token)
            .map_err(|_| DexError::InvalidCredentials("Invalid to_token address".to_string()))?;

        // Get quote first to determine best fee tier
        let quote = self.get_swap_quote(from_token, to_token, amount).await?;

        let decimals_in = self.get_token_decimals(token_in).await?;
        let amount_in_wei = Self::decimal_to_wei(amount, decimals_in);

        // Calculate minimum output with slippage
        let minimum_output = quote.to_amount * (Decimal::ONE - slippage_tolerance / Decimal::from(100));
        let decimals_out = self.get_token_decimals(token_out).await?;
        let amount_out_minimum_wei = Self::decimal_to_wei(minimum_output, decimals_out);

        // Execute swap with medium fee tier (most common on PancakeSwap)
        let tx_hash = self.execute_swap_exact_input_single(
            token_in,
            token_out,
            amount_in_wei,
            amount_out_minimum_wei,
            FEE_MEDIUM,
        ).await?;

        Ok(TransactionResult {
            transaction_hash: format!("{:?}", tx_hash),
            status: TransactionStatus::Confirmed,
            block_number: None,
            gas_used: Some(quote.estimated_gas),
            timestamp: Some(chrono::Utc::now().timestamp()),
        })
    }

    async fn get_pool_info(
        &self,
        token0: &str,
        token1: &str,
    ) -> Result<LiquidityPool, DexError> {
        tracing::info!("Fetching PancakeSwap pool info: {} / {}", token0, token1);

        // TODO: Implement pool info by querying Factory contract
        Err(DexError::UnsupportedOperation("Pool info not yet implemented".to_string()))
    }

    async fn add_liquidity(
        &self,
        token0: &str,
        token1: &str,
        amount0: Decimal,
        amount1: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError> {
        tracing::info!(
            "Adding liquidity to PancakeSwap: {} {} / {} {} (slippage: {}%)",
            amount0, token0, amount1, token1, slippage_tolerance
        );

        // TODO: Implement add liquidity via NonfungiblePositionManager
        Err(DexError::UnsupportedOperation("Add liquidity not yet implemented".to_string()))
    }

    async fn remove_liquidity(
        &self,
        token0: &str,
        token1: &str,
        liquidity_amount: Decimal,
        slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError> {
        tracing::info!(
            "Removing liquidity from PancakeSwap: {} from {} / {} (slippage: {}%)",
            liquidity_amount, token0, token1, slippage_tolerance
        );

        // TODO: Implement remove liquidity via NonfungiblePositionManager
        Err(DexError::UnsupportedOperation("Remove liquidity not yet implemented".to_string()))
    }

    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, DexError> {
        tracing::info!("Checking BNB Chain transaction status: {}", tx_hash);

        let hash = H256::from_str(tx_hash)
            .map_err(|_| DexError::InvalidCredentials("Invalid transaction hash".to_string()))?;

        match self.provider.get_transaction_receipt(hash).await {
            Ok(Some(receipt)) => {
                if receipt.status == Some(1.into()) {
                    Ok(TransactionStatus::Confirmed)
                } else {
                    Ok(TransactionStatus::Failed)
                }
            }
            Ok(None) => Ok(TransactionStatus::Pending),
            Err(e) => Err(DexError::NetworkError(format!("Failed to get transaction status: {}", e))),
        }
    }

    async fn get_gas_price(&self) -> Result<String, DexError> {
        tracing::info!("Fetching current BNB Chain gas price");

        let gas_price = self.provider
            .get_gas_price()
            .await
            .map_err(|e| DexError::NetworkError(format!("Failed to get gas price: {}", e)))?;

        Ok(gas_price.to_string())
    }

    fn dex_name(&self) -> &'static str {
        "PancakeSwap V3"
    }

    fn blockchain_network(&self) -> &'static str {
        "bnbchain"
    }
}
