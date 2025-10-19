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
