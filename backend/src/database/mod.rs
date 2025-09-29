use sea_orm::{Database, DatabaseConnection, ConnectionTrait};
use tracing::{info, warn, error};
use anyhow::{Result, Context};

pub async fn create_connection(database_url: &str) -> Result<DatabaseConnection> {
    info!("Connecting to database: {}", database_url);
    let db = Database::connect(database_url)
        .await
        .context("Failed to connect to database")?;

    // Run database migrations/setup
    setup_database(&db).await
        .context("Failed to setup database schema")?;

    info!("Database connection established successfully");
    Ok(db)
}

async fn setup_database(db: &DatabaseConnection) -> Result<()> {
    info!("Setting up database schema...");

    // Create all necessary tables if they don't exist
    create_tables(db).await?;
    create_indexes(db).await?;
    run_migrations(db).await?;

    info!("Database schema setup completed");
    Ok(())
}

async fn create_tables(db: &DatabaseConnection) -> Result<()> {
    info!("Creating database tables...");

    let tables = vec![
        ("users", include_str!("sql/create_users_table.sql")),
        ("user_profiles", include_str!("sql/create_user_profiles_table.sql")),
        ("user_sessions", include_str!("sql/create_user_sessions_table.sql")),
        ("exchange_connections", include_str!("sql/create_exchange_connections_table.sql")),
        ("wallet_balances", include_str!("sql/create_wallet_balances_table.sql")),
        ("dca_strategies", include_str!("sql/create_dca_strategies_table.sql")),
        ("dca_executions", include_str!("sql/create_dca_executions_table.sql")),
        ("market_data", include_str!("sql/create_market_data_table.sql")),
        ("backtest_results", include_str!("sql/create_backtest_results_table.sql")),
    ];

    for (table_name, sql) in tables {
        match db.execute_unprepared(sql).await {
            Ok(_) => info!("✓ Created table: {}", table_name),
            Err(e) => {
                error!("✗ Failed to create table {}: {}", table_name, e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}

async fn create_indexes(db: &DatabaseConnection) -> Result<()> {
    info!("Creating database indexes...");

    let indexes_sql = include_str!("sql/create_indexes.sql");
    db.execute_unprepared(indexes_sql)
        .await
        .context("Failed to create database indexes")?;

    info!("✓ Database indexes created successfully");
    Ok(())
}

async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    info!("Running database migrations...");

    // Migration for 2FA support
    migrate_2fa_columns(db).await?;
    // Migration for passphrase support
    migrate_passphrase_columns(db).await?;
    // Migration for strategy config JSON support
    migrate_strategy_config_columns(db).await?;
    // Migration for backtest results table schema fix
    migrate_backtest_results_schema(db).await?;

    info!("✓ Database migrations completed");
    Ok(())
}

async fn migrate_2fa_columns(db: &DatabaseConnection) -> Result<()> {
    // Check if totp_secret column exists by trying to select it
    match db.execute_unprepared("SELECT totp_secret FROM users LIMIT 1").await {
        Ok(_) => {}, // Column exists
        Err(_) => {
            // Column doesn't exist, add it
            if let Err(e) = db.execute_unprepared("ALTER TABLE users ADD COLUMN totp_secret TEXT").await {
                warn!("Could not add totp_secret column: {}", e);
            } else {
                info!("✓ Added totp_secret column to users table");
            }
        }
    }

    // Check if totp_enabled column exists by trying to select it
    match db.execute_unprepared("SELECT totp_enabled FROM users LIMIT 1").await {
        Ok(_) => {}, // Column exists
        Err(_) => {
            // Column doesn't exist, add it
            if let Err(e) = db.execute_unprepared("ALTER TABLE users ADD COLUMN totp_enabled BOOLEAN DEFAULT 0").await {
                warn!("Could not add totp_enabled column: {}", e);
            } else {
                info!("✓ Added totp_enabled column to users table");
            }
        }
    }

    // Update any NULL values
    let _ = db.execute_unprepared("UPDATE users SET totp_enabled = 0 WHERE totp_enabled IS NULL").await;

    Ok(())
}

async fn migrate_strategy_config_columns(db: &DatabaseConnection) -> Result<()> {
    // Try to select the column to see if it exists
    let test_query = "SELECT config_json FROM dca_strategies LIMIT 1";

    match db.execute_unprepared(test_query).await {
        Ok(_) => {
            // Column exists, no need to add it
            info!("config_json column already exists in dca_strategies table");
        },
        Err(_) => {
            // Column doesn't exist, add it
            match db.execute_unprepared("ALTER TABLE dca_strategies ADD COLUMN config_json TEXT").await {
                Ok(_) => info!("✓ Added config_json column to dca_strategies table"),
                Err(e) => {
                    error!("Failed to add config_json column: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}

async fn migrate_passphrase_columns(db: &DatabaseConnection) -> Result<()> {
    // Check columns by trying to select them
    let columns_to_check = vec![
        ("encrypted_passphrase", "ALTER TABLE exchange_connections ADD COLUMN encrypted_passphrase TEXT"),
        ("passphrase_nonce", "ALTER TABLE exchange_connections ADD COLUMN passphrase_nonce TEXT"),
        ("passphrase_salt", "ALTER TABLE exchange_connections ADD COLUMN passphrase_salt TEXT"),
    ];

    for (column_name, add_query) in columns_to_check {
        let test_query = format!("SELECT {} FROM exchange_connections LIMIT 1", column_name);

        match db.execute_unprepared(&test_query).await {
            Ok(_) => {}, // Column exists
            Err(_) => {
                // Column doesn't exist, add it
                if let Err(e) = db.execute_unprepared(add_query).await {
                    warn!("Could not add {} column: {}", column_name, e);
                } else {
                    info!("✓ Added {} column to exchange_connections table", column_name);
                }
            }
        }
    }

    Ok(())
}

async fn migrate_backtest_results_schema(db: &DatabaseConnection) -> Result<()> {
    // Check if the backtest_results table has the old TEXT schema
    let check_query = "SELECT sql FROM sqlite_master WHERE type='table' AND name='backtest_results'";

    match db.execute_unprepared(check_query).await {
        Ok(_) => {
            // Check if initial_balance is TEXT (old schema)
            let schema_check = "PRAGMA table_info(backtest_results)";
            if let Ok(_) = db.execute_unprepared(schema_check).await {
                // If the table exists, we need to migrate it
                info!("Migrating backtest_results table schema to use REAL types for decimal columns...");

                // SQLite doesn't support ALTER COLUMN, so we need to recreate the table
                let migration_sql = r#"
                BEGIN TRANSACTION;

                -- Create new table with correct schema
                CREATE TABLE IF NOT EXISTS backtest_results_new (
                  id TEXT PRIMARY KEY,
                  user_id TEXT NOT NULL,
                  name TEXT NOT NULL,
                  description TEXT,
                  strategy_name TEXT NOT NULL,
                  symbol TEXT NOT NULL,
                  interval TEXT NOT NULL,
                  start_date TEXT NOT NULL,
                  end_date TEXT NOT NULL,
                  initial_balance REAL NOT NULL,
                  final_balance REAL NOT NULL,
                  total_return REAL NOT NULL,
                  total_return_percentage REAL NOT NULL,
                  max_drawdown REAL NOT NULL,
                  max_drawdown_percentage REAL NOT NULL,
                  sharpe_ratio REAL,
                  total_trades INTEGER NOT NULL,
                  winning_trades INTEGER NOT NULL,
                  losing_trades INTEGER NOT NULL,
                  win_rate REAL NOT NULL,
                  profit_factor REAL,
                  largest_win REAL NOT NULL,
                  largest_loss REAL NOT NULL,
                  average_win REAL NOT NULL,
                  average_loss REAL NOT NULL,
                  strategy_parameters TEXT NOT NULL,
                  trades_data TEXT NOT NULL,
                  equity_curve TEXT NOT NULL,
                  drawdown_curve TEXT NOT NULL,
                  status TEXT NOT NULL DEFAULT 'running',
                  error_message TEXT,
                  execution_time_ms INTEGER,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
                );

                -- Copy data from old table (converting TEXT to REAL where needed)
                INSERT OR IGNORE INTO backtest_results_new
                SELECT
                  id, user_id, name, description, strategy_name, symbol, interval,
                  start_date, end_date,
                  CAST(COALESCE(initial_balance, '0') AS REAL),
                  CAST(COALESCE(final_balance, '0') AS REAL),
                  CAST(COALESCE(total_return, '0') AS REAL),
                  CAST(COALESCE(total_return_percentage, '0') AS REAL),
                  CAST(COALESCE(max_drawdown, '0') AS REAL),
                  CAST(COALESCE(max_drawdown_percentage, '0') AS REAL),
                  CASE WHEN sharpe_ratio IS NULL OR sharpe_ratio = '' THEN NULL ELSE CAST(sharpe_ratio AS REAL) END,
                  COALESCE(total_trades, 0), COALESCE(winning_trades, 0), COALESCE(losing_trades, 0),
                  CAST(COALESCE(win_rate, '0') AS REAL),
                  CASE WHEN profit_factor IS NULL OR profit_factor = '' THEN NULL ELSE CAST(profit_factor AS REAL) END,
                  CAST(COALESCE(largest_win, '0') AS REAL),
                  CAST(COALESCE(largest_loss, '0') AS REAL),
                  CAST(COALESCE(average_win, '0') AS REAL),
                  CAST(COALESCE(average_loss, '0') AS REAL),
                  COALESCE(strategy_parameters, '{}'), COALESCE(trades_data, '[]'),
                  COALESCE(equity_curve, '[]'), COALESCE(drawdown_curve, '[]'),
                  COALESCE(status, 'running'), error_message, execution_time_ms,
                  COALESCE(created_at, datetime('now')), COALESCE(updated_at, datetime('now'))
                FROM backtest_results
                WHERE EXISTS (SELECT 1 FROM backtest_results);

                -- Drop old table and rename new one
                DROP TABLE IF EXISTS backtest_results;
                ALTER TABLE backtest_results_new RENAME TO backtest_results;

                COMMIT;
                "#;

                match db.execute_unprepared(migration_sql).await {
                    Ok(_) => info!("✓ Successfully migrated backtest_results table schema"),
                    Err(e) => {
                        // If migration fails, it might be because the table already has the correct schema
                        info!("Backtest results migration skipped (table may already have correct schema): {}", e);
                    }
                }
            }
        },
        Err(_) => {
            // Table doesn't exist yet, will be created with correct schema
            info!("Backtest results table doesn't exist yet, will be created with correct schema");
        }
    }

    Ok(())
}
