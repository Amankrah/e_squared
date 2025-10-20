-- Create wallet_connections table for storing encrypted wallet credentials
CREATE TABLE IF NOT EXISTS wallet_connections (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    blockchain_network TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    display_name TEXT NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    private_key_nonce TEXT NOT NULL,
    private_key_salt TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_used TEXT,
    connection_status TEXT NOT NULL DEFAULT 'pending',
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_wallet_connections_user_id ON wallet_connections(user_id);

-- Create index on blockchain_network for filtering
CREATE INDEX IF NOT EXISTS idx_wallet_connections_network ON wallet_connections(blockchain_network);

-- Create unique index on user_id + blockchain_network + wallet_address to prevent duplicates
CREATE UNIQUE INDEX IF NOT EXISTS idx_wallet_connections_unique
ON wallet_connections(user_id, blockchain_network, wallet_address);
