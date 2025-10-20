# DEX Integration Guide

## Supported DEX Platforms

### Ethereum
- **Uniswap** - Largest DEX on Ethereum
  - Swap tokens
  - Add/Remove liquidity
  - Pool information

### BNB Chain (BSC)
- **PancakeSwap** - Largest DEX on BNB Chain
  - Swap tokens
  - Add/Remove liquidity
  - Pool information

### Solana
- **Raydium** - Largest AMM DEX on Solana
  - Swap tokens
  - Add/Remove liquidity
  - Pool information
  - SPL token support

- **Jupiter** - Leading swap aggregator on Solana
  - Best price routing across all Solana DEXs
  - Advanced swap execution
  - Real-time quotes with price impact
  - Automatic slippage protection

## API Endpoints

### Wallet Connection Management

```bash
# Connect a new wallet (supports Ethereum, BNB Chain, Solana)
POST /api/v1/wallets/connections
{
  "blockchain_network": "solana",  # or "ethereum", "bnbchain"
  "display_name": "My Trading Wallet",
  "private_key": "YOUR_PRIVATE_KEY",
  "password": "YOUR_PASSWORD"
}

# List all connected wallets
GET /api/v1/wallets/connections

# Get specific wallet details
GET /api/v1/wallets/connections/{wallet_id}

# Update wallet
PUT /api/v1/wallets/connections/{wallet_id}
{
  "display_name": "Updated Name",
  "password": "YOUR_PASSWORD"
}

# Delete wallet connection
DELETE /api/v1/wallets/connections/{wallet_id}

# Get wallet balance
POST /api/v1/wallets/connections/{wallet_id}/balance
{
  "password": "YOUR_PASSWORD"
}
```

## DEX Connector Features

### Common Operations (All DEXs)

```rust
// Test wallet connection
async fn test_connection(&self) -> Result<bool, DexError>

// Get wallet balance (native + tokens)
async fn get_wallet_balance(&self) -> Result<WalletBalance, DexError>

// Get specific token balance
async fn get_token_balance(&self, token_address: &str) -> Result<TokenBalance, DexError>

// Get swap quote
async fn get_swap_quote(
    &self,
    from_token: &str,
    to_token: &str,
    amount: Decimal,
) -> Result<SwapQuote, DexError>

// Execute swap
async fn execute_swap(
    &self,
    from_token: &str,
    to_token: &str,
    amount: Decimal,
    slippage_tolerance: Decimal,
) -> Result<TransactionResult, DexError>

// Get transaction status
async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, DexError>

// Get current gas/fee price
async fn get_gas_price(&self) -> Result<String, DexError>
```

### Liquidity Operations (Raydium, Uniswap, PancakeSwap)

```rust
// Get pool information
async fn get_pool_info(
    &self,
    token0: &str,
    token1: &str,
) -> Result<LiquidityPool, DexError>

// Add liquidity to pool
async fn add_liquidity(
    &self,
    token0: &str,
    token1: &str,
    amount0: Decimal,
    amount1: Decimal,
    slippage_tolerance: Decimal,
) -> Result<TransactionResult, DexError>

// Remove liquidity from pool
async fn remove_liquidity(
    &self,
    token0: &str,
    token1: &str,
    liquidity_amount: Decimal,
    slippage_tolerance: Decimal,
) -> Result<TransactionResult, DexError>
```

## Solana DEX Details

### Raydium Connector
- Direct integration with Raydium AMM
- Access to all Raydium liquidity pools
- LP token operations
- RPC endpoint: `https://api.mainnet-beta.solana.com`

### Jupiter Connector
- **Recommended for swaps** - finds best prices across all Solana DEXs
- Aggregates liquidity from:
  - Raydium
  - Orca
  - Serum
  - Saber
  - And 20+ other protocols
- Features:
  - Smart routing
  - Price impact calculation
  - Automatic slippage protection
  - Transaction simulation
- API: `https://quote-api.jup.ag/v6`

## Token Addresses

### Solana Common Tokens
```
SOL (Native): So11111111111111111111111111111111111111112
USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
RAY: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R
```

### Ethereum Common Tokens
```
ETH (Native): 0x0000000000000000000000000000000000000000
USDC: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
USDT: 0xdAC17F958D2ee523a2206206994597C13D831ec7
DAI: 0x6B175474E89094C44Da98b954EedeAC495271d0F
```

### BNB Chain Common Tokens
```
BNB (Native): 0x0000000000000000000000000000000000000000
BUSD: 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56
USDT: 0x55d398326f99059fF775485246999027B3197955
CAKE: 0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82
```

## Security Features

✅ **Private keys encrypted** with AES-GCM using user password
✅ **Wallet address validation** - derived from private key
✅ **Session-based authentication** required
✅ **Transaction signing** done server-side with encrypted keys
✅ **Slippage protection** on all swaps
✅ **Price impact warnings** from Jupiter

## Environment Variables

```bash
# Solana RPC endpoint (optional, defaults to public mainnet)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Or use a faster RPC provider (recommended for production)
# SOLANA_RPC_URL=https://solana-mainnet.g.alchemy.com/v2/YOUR_KEY
# SOLANA_RPC_URL=https://rpc.ankr.com/solana

# Ethereum RPC endpoint
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# BNB Chain RPC endpoint
BSC_RPC_URL=https://bsc-dataseed1.binance.org
```

## Implementation Status

### ✅ Completed
- Wallet connection management
- Database schema and migrations
- Encrypted private key storage
- Wallet address derivation (EVM + Solana)
- DEX connector framework
- Raydium connector (balance queries, transaction status)
- Jupiter connector (quotes, swaps, balance queries)
- Factory pattern for connector creation

### 🚧 In Progress
- Uniswap swap execution (quote/execute)
- PancakeSwap swap execution (quote/execute)
- Raydium swap execution (quote/execute)
- Liquidity operations (add/remove)
- Pool information queries

### 📋 Todo
- Token metadata caching
- USD price fetching for portfolio value
- Multi-hop swap optimization
- Transaction history tracking
- Gas estimation improvements
- WebSocket support for real-time updates

## Example Usage

### Connect Solana Wallet
```bash
curl -X POST http://localhost:8080/api/v1/wallets/connections \
  -H "Content-Type: application/json" \
  -d '{
    "blockchain_network": "solana",
    "display_name": "My Solana Wallet",
    "private_key": "YOUR_BASE58_PRIVATE_KEY",
    "password": "your_secure_password"
  }'
```

### Get Jupiter Swap Quote
```rust
use dex_connectors::{DexFactory, DEX, WalletCredentials};

let credentials = WalletCredentials {
    private_key: "YOUR_PRIVATE_KEY".to_string(),
    wallet_address: "YOUR_WALLET_ADDRESS".to_string(),
};

let jupiter = DexFactory::create(DEX::Jupiter, credentials)?;

// Get quote for swapping 1 SOL to USDC
let quote = jupiter.get_swap_quote(
    "So11111111111111111111111111111111111111112", // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    Decimal::from(1),
).await?;

println!("You'll receive: {} USDC", quote.to_amount);
println!("Price impact: {}%", quote.price_impact);

// Execute the swap with 0.5% slippage
let result = jupiter.execute_swap(
    "So11111111111111111111111111111111111111112",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    Decimal::from(1),
    Decimal::from_str("0.5")?,
).await?;

println!("Transaction hash: {}", result.transaction_hash);
```

## Next Steps

1. **Test with devnet/testnet wallets first**
2. **Implement remaining swap execution** for EVM chains
3. **Add token metadata service** for better UX
4. **Create frontend integration** for wallet connect UI
5. **Add portfolio tracking** across all chains
6. **Implement strategy execution** using DEX connectors
