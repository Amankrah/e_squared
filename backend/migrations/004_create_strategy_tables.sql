-- Create DCA Strategy Tables
CREATE TABLE IF NOT EXISTS dca_strategies (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_symbol TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    config_json TEXT NOT NULL,
    total_invested REAL NOT NULL DEFAULT 0.0,
    total_purchased REAL NOT NULL DEFAULT 0.0,
    average_buy_price REAL,
    last_execution_at TEXT,
    next_execution_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dca_executions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    exchange_connection_id TEXT NOT NULL,
    execution_type TEXT NOT NULL,
    trigger_reason TEXT NOT NULL,
    amount_usd REAL NOT NULL,
    amount_asset REAL,
    price_at_execution REAL,
    fear_greed_index INTEGER,
    market_volatility REAL,
    order_id TEXT,
    order_status TEXT NOT NULL,
    execution_timestamp TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES dca_strategies (id) ON DELETE CASCADE,
    FOREIGN KEY (exchange_connection_id) REFERENCES exchange_connections (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS market_data (
    id TEXT PRIMARY KEY,
    asset_symbol TEXT NOT NULL,
    price REAL NOT NULL,
    volume_24h REAL,
    market_cap REAL,
    fear_greed_index INTEGER,
    volatility_7d REAL,
    volatility_30d REAL,
    rsi_14 REAL,
    ema_20 REAL,
    ema_50 REAL,
    ema_200 REAL,
    support_level REAL,
    resistance_level REAL,
    trend_direction TEXT,
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Create Grid Trading Strategy Tables
CREATE TABLE IF NOT EXISTS grid_trading_strategies (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_symbol TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    config_json TEXT NOT NULL,
    total_invested REAL NOT NULL DEFAULT 0.0,
    total_purchased REAL NOT NULL DEFAULT 0.0,
    average_buy_price REAL,
    current_inventory REAL NOT NULL DEFAULT 0.0,
    grid_levels_count INTEGER NOT NULL DEFAULT 0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    realized_pnl REAL NOT NULL DEFAULT 0.0,
    unrealized_pnl REAL,
    max_drawdown REAL,
    grid_center_price REAL,
    grid_upper_bound REAL,
    grid_lower_bound REAL,
    last_rebalance_at TEXT,
    total_grid_profit REAL NOT NULL DEFAULT 0.0,
    active_buy_orders INTEGER NOT NULL DEFAULT 0,
    active_sell_orders INTEGER NOT NULL DEFAULT 0,
    last_execution_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS grid_trading_executions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    exchange_connection_id TEXT NOT NULL,
    execution_type TEXT NOT NULL,
    trigger_reason TEXT NOT NULL,
    amount_usd REAL NOT NULL,
    amount_asset REAL NOT NULL,
    price_at_execution REAL NOT NULL,
    grid_level_index INTEGER NOT NULL,
    grid_level_price REAL NOT NULL,
    inventory_before REAL NOT NULL,
    inventory_after REAL NOT NULL,
    grid_profit REAL,
    order_id TEXT,
    order_status TEXT NOT NULL,
    execution_timestamp TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES grid_trading_strategies (id) ON DELETE CASCADE,
    FOREIGN KEY (exchange_connection_id) REFERENCES exchange_connections (id) ON DELETE CASCADE
);

-- Create SMA Crossover Strategy Tables
CREATE TABLE IF NOT EXISTS sma_crossover_strategies (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_symbol TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    config_json TEXT NOT NULL,
    total_invested REAL NOT NULL DEFAULT 0.0,
    total_purchased REAL NOT NULL DEFAULT 0.0,
    average_buy_price REAL,
    current_position INTEGER NOT NULL DEFAULT 0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    realized_pnl REAL NOT NULL DEFAULT 0.0,
    unrealized_pnl REAL,
    current_streak INTEGER NOT NULL DEFAULT 0,
    max_drawdown REAL,
    last_fast_sma REAL,
    last_slow_sma REAL,
    last_signal_type TEXT,
    last_signal_time TEXT,
    last_execution_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sma_crossover_executions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    exchange_connection_id TEXT NOT NULL,
    execution_type TEXT NOT NULL,
    trigger_reason TEXT NOT NULL,
    amount_usd REAL NOT NULL,
    amount_asset REAL,
    price_at_execution REAL NOT NULL,
    fast_sma_value REAL NOT NULL,
    slow_sma_value REAL NOT NULL,
    sma_spread REAL NOT NULL,
    signal_strength TEXT NOT NULL,
    crossover_type TEXT,
    position_before INTEGER NOT NULL,
    position_after INTEGER NOT NULL,
    realized_pnl REAL,
    order_id TEXT,
    order_status TEXT NOT NULL,
    execution_timestamp TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES sma_crossover_strategies (id) ON DELETE CASCADE,
    FOREIGN KEY (exchange_connection_id) REFERENCES exchange_connections (id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_dca_strategies_user_id ON dca_strategies(user_id);
CREATE INDEX IF NOT EXISTS idx_dca_strategies_status ON dca_strategies(status);
CREATE INDEX IF NOT EXISTS idx_dca_executions_strategy_id ON dca_executions(strategy_id);
CREATE INDEX IF NOT EXISTS idx_market_data_symbol ON market_data(asset_symbol);
CREATE INDEX IF NOT EXISTS idx_market_data_timestamp ON market_data(timestamp);

CREATE INDEX IF NOT EXISTS idx_grid_trading_strategies_user_id ON grid_trading_strategies(user_id);
CREATE INDEX IF NOT EXISTS idx_grid_trading_strategies_status ON grid_trading_strategies(status);
CREATE INDEX IF NOT EXISTS idx_grid_trading_executions_strategy_id ON grid_trading_executions(strategy_id);

CREATE INDEX IF NOT EXISTS idx_sma_crossover_strategies_user_id ON sma_crossover_strategies(user_id);
CREATE INDEX IF NOT EXISTS idx_sma_crossover_strategies_status ON sma_crossover_strategies(status);
CREATE INDEX IF NOT EXISTS idx_sma_crossover_executions_strategy_id ON sma_crossover_executions(strategy_id);
