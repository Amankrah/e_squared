# Backtesting API Documentation

## Overview

The backtesting module provides a robust, high-performance system for testing trading strategies on historical data from Binance. It features intelligent caching to minimize API calls and prevent rate limiting.

## Key Features

### Smart Caching System
- **In-memory cache with TTL**: Caches frequently accessed data for 5-15 minutes
- **Hot data detection**: Frequently accessed data gets extended TTL (15 min vs 5 min)
- **Automatic eviction**: LRU eviction when cache exceeds 500MB
- **Rate limiting protection**: Tracks API weight and automatically throttles requests
- **Multi-user optimization**: Shared cache across users to minimize Binance API calls

### API Endpoints

#### 1. Run Backtest
```
POST /api/backtesting/run
```

Request body:
```json
{
  "symbol": "BTCUSDT",
  "interval": "1h",
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": "2024-01-31T23:59:59Z",
  "initial_balance": 10000,
  "strategy_name": "dca",
  "strategy_parameters": {
    "investment_amount": 100,
    "frequency": "daily"
  },
  "stop_loss_percentage": 5,
  "take_profit_percentage": 10
}
```

#### 2. Fetch Historical Data
```
GET /api/backtesting/historical?symbol=BTCUSDT&interval=1h&start_date=2024-01-01T00:00:00Z&end_date=2024-01-31T23:59:59Z
```

#### 3. List Available Strategies
```
GET /api/backtesting/strategies
```

#### 4. Get Strategy Details
```
GET /api/backtesting/strategies/{name}
```

#### 5. Get Available Symbols
```
GET /api/backtesting/symbols
```

#### 6. Get Available Intervals
```
GET /api/backtesting/intervals
```

#### 7. Validate Backtest Parameters
```
POST /api/backtesting/validate
```

#### 8. Cache Statistics (Admin)
```
GET /api/backtesting/cache/stats
```

#### 9. Clear Cache (Admin)
```
POST /api/backtesting/cache/clear
```

## Performance Optimization

### Caching Strategy
1. **First request**: Fetches from Binance API, stores in cache
2. **Subsequent requests**: Served from cache if within TTL
3. **Popular data**: Automatically promoted to "hot" status with extended TTL
4. **Rate limiting**: Automatic throttling when approaching Binance limits

### Rate Limiting Protection
- Tracks API weight usage (max 1200/minute for Binance)
- Automatic delays when approaching limits
- Queues requests when rate limited
- Returns cached data when possible to avoid API calls

## Architecture

### Components

1. **BacktestEngine**: Core engine for running simulations
2. **BinanceFetcher**: Handles Binance API communication with rate limiting
3. **DataCache**: Smart in-memory cache with TTL and LRU eviction
4. **StrategyFactory**: Creates and manages trading strategies

### Data Flow

```
User Request → API Endpoint → BacktestEngine
                                    ↓
                            Check DataCache
                                    ↓
                    [Cache Hit] ←─────→ [Cache Miss]
                         ↓                    ↓
                    Return Data         BinanceFetcher
                                             ↓
                                        Binance API
                                             ↓
                                        Store in Cache
                                             ↓
                                        Return Data
```

## Usage Examples

### Running a DCA Backtest

```bash
curl -X POST http://localhost:8080/api/backtesting/run \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "interval": "1d",
    "start_date": "2024-01-01T00:00:00Z",
    "end_date": "2024-12-31T23:59:59Z",
    "initial_balance": 10000,
    "strategy_name": "dca",
    "strategy_parameters": {
      "investment_amount": 100,
      "frequency": "weekly"
    }
  }'
```

### Response Structure

```json
{
  "config": { ... },
  "trades": [
    {
      "timestamp": "2024-01-01T12:00:00Z",
      "trade_type": "Buy",
      "price": 42000,
      "quantity": 0.00238,
      "total_value": 100,
      "pnl": null,
      "pnl_percentage": null
    }
  ],
  "metrics": {
    "total_return": 2500,
    "total_return_percentage": 25,
    "annualized_return": 28.5,
    "sharpe_ratio": 1.2,
    "max_drawdown": 15,
    "total_trades": 52,
    "win_rate": 65
  },
  "performance_chart": [ ... ],
  "execution_time_ms": 245
}
```

## Best Practices

1. **Use reasonable time ranges**: Limit to 1 year maximum
2. **Leverage caching**: Multiple tests on same data benefit from cache
3. **Monitor rate limits**: Check cache stats to understand usage
4. **Batch similar tests**: Run related tests close together to benefit from hot cache

## Security Considerations

- All endpoints require authentication
- No permanent storage of user data
- Cache is in-memory only and expires automatically
- API keys are never stored or logged