CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_token TEXT UNIQUE NOT NULL,
    device_info TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    location TEXT,
    user_agent TEXT NOT NULL,
    is_current BOOLEAN NOT NULL DEFAULT 0,
    last_activity TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
