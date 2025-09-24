CREATE TABLE IF NOT EXISTS wallet_balances (
    id TEXT PRIMARY KEY,
    exchange_connection_id TEXT NOT NULL,
    wallet_type TEXT NOT NULL,
    asset_symbol TEXT NOT NULL,
    free_balance TEXT NOT NULL,
    locked_balance TEXT NOT NULL,
    total_balance TEXT NOT NULL,
    usd_value TEXT,
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (exchange_connection_id) REFERENCES exchange_connections (id) ON DELETE CASCADE
);
