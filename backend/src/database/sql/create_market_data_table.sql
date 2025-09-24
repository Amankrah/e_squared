CREATE TABLE IF NOT EXISTS market_data (
    id TEXT PRIMARY KEY,
    asset_symbol TEXT NOT NULL,
    price TEXT NOT NULL,
    volume_24h TEXT,
    market_cap TEXT,
    fear_greed_index INTEGER,
    volatility_7d TEXT,
    volatility_30d TEXT,
    rsi_14 TEXT,
    ema_20 TEXT,
    ema_50 TEXT,
    ema_200 TEXT,
    support_level TEXT,
    resistance_level TEXT,
    trend_direction TEXT,
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL
);
