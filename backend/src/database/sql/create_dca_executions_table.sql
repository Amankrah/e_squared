CREATE TABLE IF NOT EXISTS dca_executions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    exchange_connection_id TEXT NOT NULL,
    execution_type TEXT NOT NULL,
    trigger_reason TEXT NOT NULL,
    amount_usd TEXT NOT NULL,
    amount_asset TEXT,
    price_at_execution TEXT,
    fear_greed_index INTEGER,
    market_volatility TEXT,
    order_id TEXT,
    order_status TEXT NOT NULL DEFAULT 'pending',
    execution_timestamp TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES dca_strategies (id) ON DELETE CASCADE,
    FOREIGN KEY (exchange_connection_id) REFERENCES exchange_connections (id) ON DELETE CASCADE
);
