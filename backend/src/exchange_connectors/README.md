# Exchange Connectors Module

## Overview

The Exchange Connectors module provides a modular, extensible architecture for connecting to multiple cryptocurrency exchanges. This system allows users to connect their exchange accounts via API keys and access account data, orders, and execute trades across different exchanges through a unified interface.

## Architecture

### Core Components

1. **Traits** (`traits.rs`)
   - `ExchangeConnector`: Base trait for all exchange connectors
   - `AccountAPI`: Interface for accessing account balances (spot, margin, futures)
   - `OrderAPI`: Interface for managing orders and order history
   - `TradeExecutionAPI`: Interface for executing trades (market, limit, stop-loss, take-profit)
   - `MarketDataAPI`: Interface for accessing market data (tickers, order books, klines)

2. **Types** (`types.rs`)
   - Unified data structures used across all exchanges
   - Account types: `SpotAccount`, `MarginAccount`, `FuturesAccount`
   - Order types: `Order`, `OcoOrder`, `Trade`
   - Market data types: `Ticker`, `OrderBook`, `Kline`

3. **Factory** (`factory.rs`)
   - `ExchangeFactory`: Creates exchange connector instances
   - Manages supported exchanges
   - Handles credential validation

4. **Error Handling** (`errors.rs`)
   - `ExchangeError`: Comprehensive error types for exchange operations
   - Automatic conversion from common error types

## Supported Exchanges

Currently implemented:
- **Binance** (Full support for spot, futures USDM)

Planned:
- Bybit
- Coinbase
- Kraken
- Kucoin
- OKX

## Usage

### Creating a Connector

```rust
use exchange_connectors::{Exchange, ExchangeFactory, ExchangeCredentials};

let credentials = ExchangeCredentials {
    api_key: "your_api_key".to_string(),
    api_secret: "your_api_secret".to_string(),
    passphrase: None, // Some exchanges like Kucoin require a passphrase
};

let connector = ExchangeFactory::create(Exchange::Binance, credentials)?;
```

### Accessing Account Data

```rust
// Get all account balances
let all_balances = connector.get_all_balances().await?;

// Get specific account types
let spot_account = connector.get_spot_account().await?;
let futures_account = connector.get_futures_account(FuturesType::USDM).await?;

// Get specific asset balance
let btc_balance = connector.get_asset_balance("BTC", WalletType::Spot).await?;
```

### Managing Orders

```rust
// Get open orders
let open_orders = connector.get_open_orders(Some("BTCUSDT"), WalletType::Spot).await?;

// Get order history
let history = connector.get_order_history(
    Some("BTCUSDT"),
    WalletType::Spot,
    Some(start_time),
    Some(end_time),
    Some(100)
).await?;

// Cancel an order
let canceled = connector.cancel_order("order_id", "BTCUSDT", WalletType::Spot).await?;
```

### Executing Trades

```rust
// Place market order
let order = connector.place_market_order(
    "BTCUSDT",
    OrderSide::Buy,
    Some(Decimal::from_str("0.001")?), // quantity
    None, // or use quote_quantity
    WalletType::Spot
).await?;

// Place limit order
let order = connector.place_limit_order(
    "BTCUSDT",
    OrderSide::Sell,
    Decimal::from_str("50000.0")?, // price
    Decimal::from_str("0.001")?, // quantity
    TimeInForce::GTC,
    WalletType::Spot
).await?;

// Place stop-loss order
let order = connector.place_stop_loss_order(
    "BTCUSDT",
    OrderSide::Sell,
    Decimal::from_str("45000.0")?, // stop price
    Decimal::from_str("0.001")?, // quantity
    None, // optional limit price
    WalletType::Spot
).await?;
```

### Accessing Market Data

```rust
// Get ticker
let ticker = connector.get_ticker("BTCUSDT").await?;

// Get order book
let order_book = connector.get_order_book("BTCUSDT", Some(100)).await?;

// Get klines/candlesticks
let klines = connector.get_klines(
    "BTCUSDT",
    KlineInterval::OneHour,
    Some(start_time),
    Some(end_time),
    Some(100)
).await?;

// Get exchange info
let info = connector.get_exchange_info().await?;
```

## API Endpoints

The module provides REST API endpoints for accessing exchange accounts:

- `GET /api/v1/exchange-accounts/all` - Get all account balances across exchanges
- `GET /api/v1/exchange-accounts/spot?connection_id={id}` - Get spot account for specific connection
- `GET /api/v1/exchange-accounts/margin?connection_id={id}` - Get margin account
- `GET /api/v1/exchange-accounts/futures?connection_id={id}` - Get futures account
- `GET /api/v1/exchange-accounts/test?connection_id={id}` - Test exchange connection

All endpoints require authentication via JWT token.

## Security

- API credentials are encrypted using AES-256-GCM before storage
- Each credential has unique salt and nonce
- Credentials are never included in API responses
- User authentication required for all operations
- Users can only access their own connections

## Adding New Exchanges

To add support for a new exchange:

1. Create a new module in `exchange_connectors/{exchange_name}/`
2. Implement all required traits:
   - `ExchangeConnector`
   - `AccountAPI`
   - `OrderAPI`
   - `TradeExecutionAPI`
   - `MarketDataAPI`
3. Add the exchange to the `Exchange` enum
4. Update the `ExchangeFactory::create()` method
5. Add converter functions for exchange-specific data types
6. Write tests for the new connector

## Error Handling

The module uses a comprehensive error system that covers:
- Authentication errors
- Network errors
- Rate limiting
- Invalid parameters
- Insufficient balance
- Order errors
- Parse errors

All errors are properly typed and can be matched for specific handling.

## Rate Limiting

Each exchange connector should implement proper rate limiting to comply with exchange API limits. The Binance connector includes rate limit information from the exchange and respects these limits.

## Testing

Run tests with:
```bash
cargo test --package e_squared_backend --lib exchange_connectors
```

## Future Enhancements

- WebSocket support for real-time data
- Margin trading APIs
- Options trading support
- Advanced order types (trailing stop, iceberg)
- Portfolio analytics
- Cross-exchange arbitrage detection
- Automated trading strategies
- Historical data aggregation