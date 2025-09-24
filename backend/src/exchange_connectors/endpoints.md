# Exchange Connector Endpoints

## Core Functions

### 1. Connect to Exchange
**Purpose**: Establish connection to exchange with API key and secret
**Endpoint**: `POST /api/v1/exchanges/connections`
**Payload**:
```json
{
  "exchange_name": "binance",
  "display_name": "My Binance Account", 
  "api_key": "your_api_key",
  "api_secret": "your_api_secret",
  "password": "your_password_for_encryption"
}
```

### 2. Read Live Balances
**Purpose**: Fetch real-time balance data from exchange
**Endpoints**:
- All balances: `POST /api/v1/exchanges/connections/{connection_id}/live-balances`
- All user balances: `POST /api/v1/exchanges/live-balances`

### 3. Automated Trading
**Purpose**: Execute trades through selected strategy
**Endpoints**:
- Execute DCA strategy: `POST /api/v1/dca/strategies/{strategy_id}/execute`
- Create DCA strategy: `POST /api/v1/dca/strategies`

## Wallet Types Supported
- **Spot**: Basic trading account
- **Margin**: Leveraged trading account
- **Futures**: Futures trading (USDM and COINM)

## Architecture
- Each exchange has its own types (e.g., `BinanceWalletType`, `BinanceSpotAccount`)
- Common interfaces defined in `traits.rs`
- Shared types like `Kline`, `Ticker` in `shared_types.rs`
- Exchange credentials simplified to `api_key` + `api_secret` only
