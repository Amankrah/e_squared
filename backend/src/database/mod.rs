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

    info!("✓ Database migrations completed");
    Ok(())
}

async fn migrate_2fa_columns(db: &DatabaseConnection) -> Result<()> {
    let migrations = vec![
        "ALTER TABLE users ADD COLUMN totp_secret TEXT;",
        "ALTER TABLE users ADD COLUMN totp_enabled BOOLEAN DEFAULT 0;",
        "UPDATE users SET totp_enabled = 0 WHERE totp_enabled IS NULL;",
    ];

    for migration in migrations {
        if let Err(e) = db.execute_unprepared(migration).await {
            // Log but don't fail - these columns might already exist
            warn!("Migration warning (expected if already applied): {}", e);
        }
    }

    Ok(())
}

async fn migrate_passphrase_columns(db: &DatabaseConnection) -> Result<()> {
    let migrations = vec![
        "ALTER TABLE exchange_connections ADD COLUMN encrypted_passphrase TEXT;",
        "ALTER TABLE exchange_connections ADD COLUMN passphrase_nonce TEXT;",
        "ALTER TABLE exchange_connections ADD COLUMN passphrase_salt TEXT;",
    ];

    for migration in migrations {
        if let Err(e) = db.execute_unprepared(migration).await {
            // Log but don't fail - these columns might already exist
            warn!("Migration warning (expected if already applied): {}", e);
        }
    }

    Ok(())
}
