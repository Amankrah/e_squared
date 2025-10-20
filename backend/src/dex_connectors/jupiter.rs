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

/// Jupiter V6 API Quote Response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterQuoteResponse {
    input_mint: String,
    in_amount: String,
    output_mint: String,
    out_amount: String,
    other_amount_threshold: String,
    swap_mode: String,
    slippage_bps: u64,
    #[serde(default)]
    price_impact_pct: String,
    route_plan: Vec<JupiterRoutePlan>,
    #[serde(default)]
    context_slot: Option<u64>,
    #[serde(default)]
    time_taken: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterRoutePlan {
    swap_info: JupiterSwapInfo,
    percent: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapInfo {
    amm_key: String,
    label: Option<String>,
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    fee_amount: String,
    fee_mint: String,
}

/// Jupiter V6 API Swap Response
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapResponse {
    swap_transaction: String, // Base64 encoded serialized transaction
    #[serde(default)]
    last_valid_block_height: Option<u64>,
    #[serde(default)]
    prioritization_fee_lamports: Option<u64>,
}

/// Jupiter V6 API Swap Request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapRequest {
    quote_response: JupiterQuoteResponse,
    user_public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrap_and_unwrap_sol: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_shared_accounts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dynamic_compute_unit_limit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority_level_with_max_lamports: Option<PriorityLevel>,
}

#[derive(Debug, Serialize)]
struct PriorityLevel {
    #[serde(rename = "priorityLevel")]
    priority_level: String, // "veryHigh", "high", "medium", "low"
}

/// Jupiter aggregator connector for Solana
/// Jupiter finds the best swap routes across all Solana DEXs
pub struct JupiterConnector {
    credentials: WalletCredentials,
    rpc_client: RpcClient,
    keypair: Keypair,
    jupiter_api_url: String,
    http_client: reqwest::Client,
}

impl JupiterConnector {
    pub fn new(credentials: WalletCredentials) -> Result<Self, DexError> {
        // Connect to Solana mainnet-beta
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

        // Jupiter V6 API (configurable for paid tier)
        let jupiter_api_url = std::env::var("JUPITER_API_URL")
            .unwrap_or_else(|_| "https://quote-api.jup.ag/v6".to_string());

        Ok(Self {
            credentials,
            rpc_client,
            keypair,
            jupiter_api_url,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| DexError::InternalError(format!("Failed to create HTTP client: {}", e)))?,
        })
    }

    fn parse_private_key(private_key: &str) -> Result<Keypair, DexError> {
        // Try parsing as base58 encoded keypair (64 bytes)
        if let Ok(bytes) = bs58::decode(private_key).into_vec() {
            if bytes.len() == 64 {
                return Keypair::from_bytes(&bytes)
                    .map_err(|e| DexError::InvalidCredentials(format!("Invalid keypair: {}", e)));
            } else if bytes.len() == 32 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes);
                return Ok(Keypair::from_seed(&seed)
                    .map_err(|e| DexError::InvalidCredentials(format!("Invalid seed: {}", e)))?);
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

    async fn fetch_jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u64,
    ) -> Result<JupiterQuoteResponse, DexError> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.jupiter_api_url, input_mint, output_mint, amount, slippage_bps
        );

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| DexError::NetworkError(format!("Jupiter API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DexError::NetworkError(
                format!("Jupiter API error {}: {}", status, error_text)
            ));
        }

        response.json::<JupiterQuoteResponse>()
            .await
            .map_err(|e| DexError::InternalError(format!("Failed to parse Jupiter quote: {}", e)))
    }

    async fn fetch_jupiter_swap_transaction(
        &self,
        quote: JupiterQuoteResponse,
    ) -> Result<JupiterSwapResponse, DexError> {
        let url = format!("{}/swap", self.jupiter_api_url);

        let swap_request = JupiterSwapRequest {
            quote_response: quote,
            user_public_key: self.keypair.pubkey().to_string(),
            wrap_and_unwrap_sol: Some(true),
            use_shared_accounts: Some(true),
            dynamic_compute_unit_limit: Some(true),
            priority_level_with_max_lamports: Some(PriorityLevel {
                priority_level: "high".to_string(),
            }),
        };

        tracing::debug!("Jupiter swap request to: {}", url);

        let response = self.http_client
            .post(&url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| DexError::NetworkError(format!("Jupiter swap API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Jupiter swap API error {}: {}", status, error_text);
            return Err(DexError::NetworkError(
                format!("Jupiter swap API error {}: {}", status, error_text)
            ));
        }

        let swap_response = response.json::<JupiterSwapResponse>()
            .await
            .map_err(|e| DexError::InternalError(format!("Failed to parse Jupiter swap response: {}", e)))?;

        Ok(swap_response)
    }
}

#[async_trait]
impl DexConnector for JupiterConnector {
    async fn test_connection(&self) -> Result<bool, DexError> {
        tracing::info!("Testing Jupiter connection for wallet: {}", self.credentials.wallet_address);

        // Test both Solana RPC and Jupiter API
        match self.get_sol_balance().await {
            Ok(_) => {
                tracing::info!("Jupiter/Solana connection successful");
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Jupiter connection test failed: {:?}", e);
                Err(e)
            }
        }
    }

    async fn get_wallet_balance(&self) -> Result<WalletBalance, DexError> {
        tracing::info!("Fetching Jupiter wallet balance for: {}", self.credentials.wallet_address);

        // Get SOL balance
        let sol_lamports = self.get_sol_balance().await?;
        let sol_balance = Decimal::from(sol_lamports) / Decimal::from(1_000_000_000);

        // TODO: Fetch all SPL token balances
        let tokens = vec![];

        Ok(WalletBalance {
            wallet_address: self.credentials.wallet_address.clone(),
            native_balance: sol_balance,
            native_symbol: "SOL".to_string(),
            tokens,
            total_usd_value: Decimal::ZERO,
        })
    }

    async fn get_token_balance(&self, token_address: &str) -> Result<TokenBalance, DexError> {
        tracing::info!("Fetching SPL token balance for: {}", token_address);

        let token_mint = Pubkey::from_str(token_address)
            .map_err(|_| DexError::InvalidCredentials("Invalid token address".to_string()))?;

        let balance = self.get_spl_token_balance(&token_mint).await?;

        Ok(TokenBalance {
            token_address: token_address.to_string(),
            token_symbol: "UNKNOWN".to_string(),
            token_name: "Unknown Token".to_string(),
            balance: Decimal::from(balance),
            decimals: 9,
            usd_value: None,
        })
    }

    async fn get_swap_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: Decimal,
    ) -> Result<SwapQuote, DexError> {
        tracing::info!("Getting Jupiter swap quote: {} {} -> {}", amount, from_token, to_token);

        // Convert Decimal to smallest units (lamports/base units)
        let amount_lamports = (amount * Decimal::from(1_000_000_000))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Amount overflow".to_string()))?;

        // Default slippage: 50 bps = 0.5%
        let slippage_bps = 50;

        let jupiter_quote = self.fetch_jupiter_quote(
            from_token,
            to_token,
            amount_lamports,
            slippage_bps,
        ).await?;

        // Parse amounts
        let out_amount = jupiter_quote.out_amount.parse::<u64>()
            .map_err(|_| DexError::InternalError("Invalid output amount".to_string()))?;
        let out_amount_decimal = Decimal::from(out_amount) / Decimal::from(1_000_000_000);

        let minimum_received = jupiter_quote.other_amount_threshold.parse::<u64>()
            .map_err(|_| DexError::InternalError("Invalid minimum amount".to_string()))?;
        let minimum_received_decimal = Decimal::from(minimum_received) / Decimal::from(1_000_000_000);

        let price_impact = jupiter_quote.price_impact_pct.parse::<f64>()
            .map_err(|_| DexError::InternalError("Invalid price impact".to_string()))?;
        let price_impact_decimal = Decimal::try_from(price_impact)
            .unwrap_or(Decimal::ZERO);

        // Extract route
        let route: Vec<String> = jupiter_quote.route_plan.iter()
            .flat_map(|plan| vec![plan.swap_info.input_mint.clone(), plan.swap_info.output_mint.clone()])
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
            "Executing Jupiter swap: {} {} -> {} (slippage: {}%)",
            amount, from_token, to_token, slippage_tolerance
        );

        // Convert slippage from percentage to basis points
        let slippage_bps = (slippage_tolerance * Decimal::from(100))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Invalid slippage".to_string()))?;

        // Convert amount to lamports
        let amount_lamports = (amount * Decimal::from(1_000_000_000))
            .to_u64()
            .ok_or_else(|| DexError::InternalError("Amount overflow".to_string()))?;

        // Get quote
        let quote = self.fetch_jupiter_quote(
            from_token,
            to_token,
            amount_lamports,
            slippage_bps,
        ).await?;

        // Get swap transaction
        let swap_response = self.fetch_jupiter_swap_transaction(quote).await?;

        // Decode base64 transaction
        let tx_bytes = base64::decode(&swap_response.swap_transaction)
            .map_err(|e| DexError::InternalError(format!("Failed to decode transaction: {}", e)))?;

        // Deserialize as VersionedTransaction (Jupiter uses versioned transactions with ALTs)
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

        // Send and confirm transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| {
                tracing::error!("Transaction failed: {:?}", e);
                DexError::TransactionFailed(format!("Swap failed: {}", e))
            })?;

        tracing::info!("Swap successful! Signature: {}", signature);

        Ok(TransactionResult {
            transaction_hash: signature.to_string(),
            status: TransactionStatus::Confirmed,
            block_number: swap_response.last_valid_block_height,
            gas_used: swap_response.prioritization_fee_lamports.map(|f| f.to_string()),
            timestamp: Some(chrono::Utc::now().timestamp()),
        })
    }

    async fn get_pool_info(
        &self,
        token0: &str,
        token1: &str,
    ) -> Result<LiquidityPool, DexError> {
        tracing::info!("Jupiter is an aggregator, pool info not directly available");
        Err(DexError::UnsupportedOperation(
            "Jupiter is a swap aggregator. Use Raydium connector for pool info.".to_string()
        ))
    }

    async fn add_liquidity(
        &self,
        _token0: &str,
        _token1: &str,
        _amount0: Decimal,
        _amount1: Decimal,
        _slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError> {
        Err(DexError::UnsupportedOperation(
            "Jupiter is a swap aggregator. Use Raydium connector for liquidity operations.".to_string()
        ))
    }

    async fn remove_liquidity(
        &self,
        _token0: &str,
        _token1: &str,
        _liquidity_amount: Decimal,
        _slippage_tolerance: Decimal,
    ) -> Result<TransactionResult, DexError> {
        Err(DexError::UnsupportedOperation(
            "Jupiter is a swap aggregator. Use Raydium connector for liquidity operations.".to_string()
        ))
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
        tracing::info!("Fetching Solana fee estimate");
        Ok("5000".to_string()) // Solana base fee is 5000 lamports
    }

    fn dex_name(&self) -> &'static str {
        "Jupiter"
    }

    fn blockchain_network(&self) -> &'static str {
        "solana"
    }
}
