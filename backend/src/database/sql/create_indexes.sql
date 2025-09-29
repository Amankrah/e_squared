-- User-related indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_user_profiles_user_id ON user_profiles(user_id);

-- Session-related indexes
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions(expires_at);

-- Exchange connection indexes
CREATE INDEX IF NOT EXISTS idx_exchange_connections_user_id ON exchange_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_exchange_connections_exchange_name ON exchange_connections(exchange_name);

-- Wallet balance indexes
CREATE INDEX IF NOT EXISTS idx_wallet_balances_exchange_connection_id ON wallet_balances(exchange_connection_id);
CREATE INDEX IF NOT EXISTS idx_wallet_balances_wallet_type ON wallet_balances(wallet_type);
CREATE INDEX IF NOT EXISTS idx_wallet_balances_asset_symbol ON wallet_balances(asset_symbol);

-- DCA strategy indexes
CREATE INDEX IF NOT EXISTS idx_dca_strategies_user_id ON dca_strategies(user_id);
CREATE INDEX IF NOT EXISTS idx_dca_strategies_status ON dca_strategies(status);
CREATE INDEX IF NOT EXISTS idx_dca_strategies_asset_symbol ON dca_strategies(asset_symbol);
CREATE INDEX IF NOT EXISTS idx_dca_strategies_next_execution ON dca_strategies(next_execution_at);

-- DCA execution indexes
CREATE INDEX IF NOT EXISTS idx_dca_executions_strategy_id ON dca_executions(strategy_id);
CREATE INDEX IF NOT EXISTS idx_dca_executions_timestamp ON dca_executions(execution_timestamp);

-- Market data indexes
CREATE INDEX IF NOT EXISTS idx_market_data_symbol ON market_data(asset_symbol);
CREATE INDEX IF NOT EXISTS idx_market_data_timestamp ON market_data(timestamp);

-- Backtest result indexes
CREATE INDEX IF NOT EXISTS idx_backtest_results_user_id ON backtest_results(user_id);
CREATE INDEX IF NOT EXISTS idx_backtest_results_strategy ON backtest_results(strategy_name);
CREATE INDEX IF NOT EXISTS idx_backtest_results_symbol ON backtest_results(symbol);
CREATE INDEX IF NOT EXISTS idx_backtest_results_status ON backtest_results(status);
CREATE INDEX IF NOT EXISTS idx_backtest_results_created_at ON backtest_results(created_at);
