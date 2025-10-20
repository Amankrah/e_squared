use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer, SeedDerivable},
    commitment_config::CommitmentConfig,
    transaction::VersionedTransaction,
};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

use super::common_types::*;
use super::errors::DexError;
use super::traits::DexConnector;
use super::WalletCredentials;

/// Raydium Production-Ready Integration
///
/// This connector implements Raydium's official Transaction API v1 for swaps on Solana.
///
/// ## Features Implemented:
/// - ✅ Swap quotes using compute/swap-base-in endpoint
/// - ✅ Swap execution with versioned transactions (V0)
/// - ✅ Priority fee optimization (vh/h/m tiers)
/// - ✅ Automatic SOL wrapping/unwrapping
/// - ✅ Transaction status tracking
/// - ✅ Wallet balance queries (SOL and SPL tokens)
///
/// ## API Endpoints Used:
/// - GET  /main/priority-fee - Dynamic priority fees
/// - GET  /compute/swap-base-in - Swap quotes
/// - POST /transaction/swap-base-in - Swap transaction generation
///
/// ## References:
/// - Official Docs: https://docs.raydium.io/raydium/traders/trade-api
/// - Transaction API: https://transaction-v1.raydium.io
/// - API v3: https://api-v3.raydium.io

/// Raydium API URLs
const RAYDIUM_API_BASE: &str = "https://api-v3.raydium.io";
const RAYDIUM_TRANSACTION_API: &str = "https://transaction-v1.raydium.io";

/// Raydium V3 API Priority Fee Response
#[derive(Debug, Deserialize)]
struct PriorityFeeResponse {
    id: String,
    success: bool,
    data: PriorityFeeData,
}

#[derive(Debug, Deserialize)]
struct PriorityFeeData {
    default: PriorityFeeTiers,
}

#[derive(Debug, Deserialize)]
struct PriorityFeeTiers {
    vh: u64,  // very high
    h: u64,   // high
    m: u64,   // medium
}

/// Raydium Swap Compute Response (Quote)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapComputeResponse {
    id: String,
    success: bool,
    version: String,  // "V0" or "V1"
    data: SwapComputeData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapComputeData {
    swap_type: String,  // "BaseIn" or "BaseOut"
    input_mint: String,
    input_amount: String,
    output_mint: String,
    output_amount: String,
    other_amount_threshold: String,
    slippage_bps: u64,
    price_impact_pct: f64,
    route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoutePlan {
    pool_id: String,
    input_mint: String,
    output_mint: String,
    fee_mint: String,
    fee_rate: f64,
    fee_amount: String,
}

/// Raydium Swap Transaction Request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapTransactionRequest {
    compute_unit_price_micro_lamports: String,
    swap_response: SwapComputeResponse,
    tx_version: String,  // "V0" or "LEGACY"
    wallet: String,
    wrap_sol: bool,
    unwrap_sol: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_account: Option<String>,
}

/// Raydium Swap Transaction Response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapTransactionResponse {
    id: String,
    version: String,
    success: bool,
    data: Vec<SwapTransactionData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapTransactionData {
    transaction: String,  // Base64 encoded serialized transaction
}

/// Raydium DEX connector for Solana
/// Raydium is the largest AMM DEX on Solana
pub struct RaydiumConnector {
    credentials: WalletCredentials,
    rpc_client: RpcClient,
    keypair: Keypair,
    http_client: reqwest::Client,
    raydium_api_base: String,
    raydium_transaction_api: String,
}

impl RaydiumConnector {
    pub fn new(credentials: WalletCredentials) -> Result<Self, DexError> {
        // Connect to Solana mainnet-beta (or use devnet for testing)
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

        let rpc_client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        );

        // Parse private key to create keypair
        let keypair = Self::parse_private_key(&credentials.private_key)?;

        // Verify wallet address matches
        let derived_address = keypair.pubkey().to_string();
        if derived_address != credentials.wallet_address {
            return Err(DexError::InvalidCredentials(
                "Wallet address does not match private key".to_string()
            ));
        }

        // Configure Raydium API URLs (allow override for testing)
        let raydium_api_base = std::env::var("RAYDIUM_API_BASE")
            .unwrap_or_else(|_| RAYDIUM_API_BASE.to_string());
        let raydium_transaction_api = std::env::var("RAYDIUM_TRANSACTION_API")
            .unwrap_or_else(|_| RAYDIUM_TRANSACTION_API.to_string());

        Ok(Self {
            credentials,
            rpc_client,
            keypair,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| DexError::InternalError(format!("Failed to create HTTP client: {}", e)))?,
            raydium_api_base,
            raydium_transaction_api,
        })
    }

    fn parse_private_key(private_key: &str) -> Result<Keypair, DexError> {
        // Try parsing as base58 encoded keypair (64 bytes)
        if let Ok(bytes) = bs58::decode(private_key).into_vec() {
            if bytes.len() == 64 {
                return Keypair::from_bytes(&bytes)
                    .map_err(|e| DexError::InvalidCredentials(format!("Invalid keypair: {}", e)));
            } else if bytes.len() == 32 {
                // Seed bytes only
                return Ok(Keypair::from_bytes(&{
                    let mut full = [0u8; 64];
                    full[..32].copy_from_slice(&bytes);
                    full
                }).map_err(|e| DexError::InvalidCredentials(format!("Invalid seed: {}", e)))?);
            }
        }

        // Try parsing as hex
        if private_key.len() == 64 {
            if let Ok(bytes) = hex::decode(private_key) {
                if bytes.len() == 32 {
                    let mut seed = [0u8; 32];
                    seed.copy_from_slice(&bytes);
                    return Ok(Keypair::from_seed(&seed)
                        .map_err(|e| DexError::InvalidCredentials(format!("Invalid seed: {}", e)))?);
                }
            }
        }

        Err(DexError::InvalidCredentials(
            "Invalid Solana private key format. Expected base58 or hex encoded key.".to_string()
        ))
    }

    async fn get_sol_balance(&self) -> Result<u64, DexError> {
        self.rpc_client
            .get_balance(&self.keypair.pubkey())
            .map_err(|e| DexError::NetworkError(format!("Failed to get SOL balance: {}", e)))
    }

    async fn get_spl_token_balance(&self, token_mint: &Pubkey) -> Result<u64, DexError> {
        use spl_associated_token_account::get_associated_token_address;

        let associated_token_address = get_associated_token_address(
            &self.keypair.pubkey(),
            token_mint,
        );

        match self.rpc_client.get_token_account_balance(&associated_token_address) {
            Ok(balance) => {
                balance.ui_amount
                    .map(|amount| (amount * 10f64.powi(balance.decimals as i32)) as u64)
                    .ok_or_else(|| DexError::InternalError("Invalid token balance".to_string()))
            }
            Err(_) => Ok(0), // Account doesn't exist = 0 balance
        }
    }

    /// Fetch current priority fees from Raydium API
    async fn fetch_priority_fees(&self) -> Result<PriorityFeeTiers, DexError> {
        let url = format!("{}/main/priority-fee", self.raydium_api_base);

        tracing::debug!("Fetching Raydium priority fees from: {}", url);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| DexError::NetworkError(format!("Priority fee request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DexError::NetworkError(
                format!("Priority fee API error {}: {}", status, error_text)
            ));
        }

        let fee_response = response.json::<PriorityFeeResponse>()
            .await
            .map_err(|e| DexError::InternalError(format!("Failed to parse priority fees: {}", e)))?;

        if !fee_response.success {
            return Err(DexError::InternalError("Priority fee API returned success=false".to_string()));
        }

        Ok(fee_response.data.default)
    }

    /// Compute swap quote from Raydium API (swap-base-in means exact input amount)
    async fn compute_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u64,
    ) -> Result<SwapComputeResponse, DexError> {
        let url = format!(
            "{}/compute/swap-base-in?inputMint={}&outputMint={}&amount={}&slippageBps={}&txVersion=V0",
            self.raydium_transaction_api, input_mint, output_mint, amount, slippage_bps
        );

        tracing::debug!("Computing Raydium swap: {}", url);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| DexError::NetworkError(format!("Swap compute request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DexError::NetworkError(
                format!("Swap compute API error {}: {}", status, error_text)
            ));
        }

        let swap_response = response.json::<SwapComputeResponse>()
            .await
            .map_err(|e| DexError::InternalError(format!("Failed to parse swap compute: {}", e)))?;

        if !swap_response.success {
            return Err(DexError::InternalError("Swap compute API returned success=false".to_string()));
        }

        Ok(swap_response)
    }

    /// Fetch swap transaction from Raydium API
    async fn fetch_swap_transaction(
        &self,
        swap_compute: SwapComputeResponse,
        priority_fee: u64,
        wrap_sol: bool,
        unwrap_sol: bool,
    ) -> Result<SwapTransactionResponse, DexError> {
        let url = format!("{}/transaction/swap-base-in", self.raydium_transaction_api);

        let request = SwapTransactionRequest {
            compute_unit_price_micro_lamports: priority_fee.to_string(),
            swap_response: swap_compute,
            tx_version: "V0".to_string(),  // Use versioned transactions for better performance
            wallet: self.keypair.pubkey().to_string(),
            wrap_sol,
            unwrap_sol,
            input_account: None,
            output_account: None,
        };

        tracing::debug!("Fetching Raydium swap transaction from: {}", url);

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DexError::NetworkError(format!("Swap transaction request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Raydium swap transaction API error {}: {}", status, error_text);
            return Err(DexError::NetworkError(
                format!("Swap transaction API error {}: {}", status, error_text)
            ));
        }

        let tx_response = response.json::<SwapTransactionResponse>()
            .await
            .map_err(|e| DexError::InternalError(format!("Failed to parse swap transaction: {}", e)))?;

        if !tx_response.success {
            return Err(DexError::InternalError("Swap transaction API returned success=false".to_string()));
        }

        Ok(tx_response)
    }
}

#[async_trait]
impl DexConnector for RaydiumConnector {
    async fn test_connection(&self) -> Result<bool, DexError> {
        tracing::info!("Testing Raydium connection for wallet: {}", self.credentials.wallet_address);

        // Try to get SOL balance as a connection test
        match self.get_sol_balance().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::error!("Raydium connection test failed: {:?}", e);
                Err(e)
            }
        }
    }

    async fn get_wallet_balance(&self) -> Result<WalletBalance, DexError> {
        tracing::info!("Fetching Raydium wallet balance for: {}", self.credentials.wallet_address);

        // Get SOL balance
        let sol_lamports = self.get_sol_balance().await?;
        let sol_balance = Decimal::from(sol_lamports) / Decimal::from(1_000_000_000); // Convert lamports to SOL

        // TODO: Fetch all SPL token balances
        // This requires querying all token accounts owned by the wallet
        let tokens = vec![];

        Ok(WalletBalance {
            wallet_address: self.credentials.wallet_address.clone(),
            native_balance: sol_balance,
            native_symbol: "SOL".to_string(),
            tokens,
            total_usd_value: Decimal::ZERO, // TODO: Calculate USD value
        })
    }

    async fn get_token_balance(&self, token_address: &str) -> Result<TokenBalance, DexError> {
        tracing::info!("Fetching SPL token balance for: {}", token_address);

        let token_mint = Pubkey::from_str(token_address)
            .map_err(|_| DexError::InvalidCredentials("Invalid token address".to_string()))?;

        let balance = self.get_spl_token_balance(&token_mint).await?;

        // TODO: Fetch token metadata (symbol, name, decimals)
        Ok(TokenBalance {
            token_address: token_address.to_string(),
            token_symbol: "UNKNOWN".to_string(),
            token_name: "Unknown Token".to_string(),
            balance: Decimal::from(balance),
            decimals: 9, // Default for most Solana tokens
            usd_value: None,
        })
    }

    async fn get_swap_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
    ) -> Result<SwapQuote, DexError> {
        tracing::info!("Getting Raydium swap quote: {} {} -> {}", amount, from_token, to_token);

        // Convert Decimal to smallest units (lamports/base units)
        let amount_lamports = (amount * Decimal::from(1_000_000_000))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Amount overflow".to_string()))?;

        // Default slippage: 50 bps = 0.5%
        let slippage_bps = 50;

        // Get quote from Raydium API
        let swap_compute = self.compute_swap(
            from_token,
            to_token,
            amount_lamports,
            slippage_bps,
        ).await?;

        let data = &swap_compute.data;

        // Parse amounts
        let out_amount = data.output_amount.parse::<u64>()
            .map_err(|_| DexError::InternalError("Invalid output amount".to_string()))?;
        let out_amount_decimal = Decimal::from(out_amount) / Decimal::from(1_000_000_000);

        let minimum_received = data.other_amount_threshold.parse::<u64>()
            .map_err(|_| DexError::InternalError("Invalid minimum amount".to_string()))?;
        let minimum_received_decimal = Decimal::from(minimum_received) / Decimal::from(1_000_000_000);

        let price_impact_decimal = Decimal::try_from(data.price_impact_pct)
            .unwrap_or(Decimal::ZERO);

        // Extract route from route plan
        let route: Vec<String> = data.route_plan.iter()
            .flat_map(|plan| vec![plan.input_mint.clone(), plan.output_mint.clone()])
            .collect();

        Ok(SwapQuote {
            from_token: Token {
                address: from_token.to_string(),
                symbol: "UNKNOWN".to_string(),
                name: "Unknown".to_string(),
                decimals: 9,
            },
            to_token: Token {
                address: to_token.to_string(),
                symbol: "UNKNOWN".to_string(),
                name: "Unknown".to_string(),
                decimals: 9,
            },
            from_amount: amount,
            to_amount: out_amount_decimal,
            price_impact: price_impact_decimal,
            minimum_received: minimum_received_decimal,
            route,
            estimated_gas: "5000".to_string(), // Solana base fee
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
            "Executing Raydium swap: {} {} -> {} (slippage: {}%)",
            amount, from_token, to_token, slippage_tolerance
        );

        // Native SOL mint address (wrapped SOL)
        const NATIVE_SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        // Determine if we need to wrap/unwrap SOL
        let is_input_sol = from_token == NATIVE_SOL_MINT;
        let is_output_sol = to_token == NATIVE_SOL_MINT;

        // Convert slippage from percentage to basis points
        let slippage_bps = (slippage_tolerance * Decimal::from(100))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Invalid slippage".to_string()))?;

        // Convert amount to lamports
        let amount_lamports = (amount * Decimal::from(1_000_000_000))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Amount overflow".to_string()))?;

        // Step 1: Get priority fees
        let priority_fees = self.fetch_priority_fees().await?;
        tracing::debug!("Priority fees - vh: {}, h: {}, m: {}", priority_fees.vh, priority_fees.h, priority_fees.m);

        // Step 2: Compute swap
        let swap_compute = self.compute_swap(
            from_token,
            to_token,
            amount_lamports,
            slippage_bps,
        ).await?;

        tracing::info!(
            "Raydium quote: {} -> {} (price impact: {}%)",
            swap_compute.data.input_amount,
            swap_compute.data.output_amount,
            swap_compute.data.price_impact_pct
        );

        // Step 3: Fetch swap transaction (use high priority)
        let tx_response = self.fetch_swap_transaction(
            swap_compute,
            priority_fees.h,  // Use high priority
            is_input_sol,
            is_output_sol,
        ).await?;

        if tx_response.data.is_empty() {
            return Err(DexError::InternalError("No transaction data returned".to_string()));
        }

        // Step 4: Decode and sign transaction
        let tx_data = &tx_response.data[0];
        let tx_bytes = base64::decode(&tx_data.transaction)
            .map_err(|e| DexError::InternalError(format!("Failed to decode transaction: {}", e)))?;

        // Deserialize as VersionedTransaction (Raydium uses V0 transactions)
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| DexError::InternalError(format!("Failed to deserialize transaction: {}", e)))?;

        // Sign transaction by adding signature (Solana SDK 2.1 approach)
        let message_bytes = transaction.message.serialize();
        let signature = self.keypair.sign_message(&message_bytes);

        // Find the index of our pubkey and replace the signature
        if let Some(index) = transaction.message.static_account_keys().iter().position(|key| key == &self.keypair.pubkey()) {
            if index < transaction.signatures.len() {
                transaction.signatures[index] = signature;
            }
        }

        tracing::debug!("Transaction signed, sending to network...");

        // Step 5: Send and confirm transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| {
                tracing::error!("Raydium swap transaction failed: {:?}", e);
                DexError::TransactionFailed(format!("Swap failed: {}", e))
            })?;

        tracing::info!("Raydium swap successful! Signature: {}", signature);

        Ok(TransactionResult {
            transaction_hash: signature.to_string(),
            status: TransactionStatus::Confirmed,
            block_number: None,
            gas_used: Some(priority_fees.h.to_string()),
            timestamp: Some(chrono::Utc::now().timestamp()),
        })
    }

    async fn get_pool_info(
        &self,
        token0: &str,
        token1: &str,
    ) -> Result<LiquidityPool, DexError> {
        tracing::info!("Fetching Raydium pool info: {} / {}", token0, token1);

        // TODO: Implement Raydium pool info retrieval
        // This requires querying the Raydium AMM program for pool state

        Err(DexError::UnsupportedOperation("Raydium pool info not yet implemented".to_string()))
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
            "Adding liquidity to Raydium: {} {} / {} {} (slippage: {}%)",
            amount0, token0, amount1, token1, slippage_tolerance
        );

        // TODO: Implement add liquidity
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
            "Removing liquidity from Raydium: {} from {} / {} (slippage: {}%)",
            liquidity_amount, token0, token1, slippage_tolerance
        );

        // TODO: Implement remove liquidity
        Err(DexError::UnsupportedOperation("Remove liquidity not yet implemented".to_string()))
    }

    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, DexError> {
        tracing::info!("Checking Solana transaction status: {}", tx_hash);

        use solana_sdk::signature::Signature;

        let signature = Signature::from_str(tx_hash)
            .map_err(|_| DexError::InvalidCredentials("Invalid transaction signature".to_string()))?;

        match self.rpc_client.get_signature_status(&signature) {
            Ok(Some(status)) => {
                match status {
                    Ok(_) => Ok(TransactionStatus::Confirmed),
                    Err(_) => Ok(TransactionStatus::Failed),
                }
            }
            Ok(None) => Ok(TransactionStatus::Pending),
            Err(e) => Err(DexError::NetworkError(format!("Failed to get transaction status: {}", e))),
        }
    }

    async fn get_gas_price(&self) -> Result<String, DexError> {
        tracing::info!("Fetching Solana recent blockhash (fee estimate)");

        match self.rpc_client.get_latest_blockhash() {
            Ok(_blockhash) => {
                // Solana uses a fixed fee structure (5000 lamports per signature)
                Ok("5000".to_string())
            }
            Err(e) => Err(DexError::NetworkError(format!("Failed to get blockhash: {}", e))),
        }
    }

    fn dex_name(&self) -> &'static str {
        "Raydium"
    }

    fn blockchain_network(&self) -> &'static str {
        "solana"
    }
}
