mod config;
mod handlers;
mod middleware;
mod models;
mod utils;
mod services;
mod exchange_connectors;
mod backtesting;
mod strategies;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger, cookie::Key};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::env;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use config::Config;
use handlers::{auth, user_profile, two_factor, session_management, exchange_management, dca_strategy_management, strategy_templates_handler, AuthService};
use services::{MarketDataService, DCAExecutionEngine, StrategyTemplateService};
use utils::encryption::EncryptionService;
use middleware::{AuthMiddleware, SessionTrackingMiddleware};

async fn create_database_connection(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db = Database::connect(database_url).await?;

    // Create tables if they don't exist
    let create_users_table = r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            is_verified BOOLEAN NOT NULL DEFAULT 0,
            totp_secret TEXT,
            totp_enabled BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
    "#;

    let create_user_profiles_table = r#"
        CREATE TABLE IF NOT EXISTS user_profiles (
            id TEXT PRIMARY KEY,
            user_id TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            phone TEXT,
            location TEXT,
            bio TEXT,
            join_date TEXT NOT NULL,
            avatar_url TEXT,
            is_verified BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        );
    "#;

    let create_user_sessions_table = r#"
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
    "#;

    let create_exchange_connections_table = r#"
        CREATE TABLE IF NOT EXISTS exchange_connections (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            exchange_name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            encrypted_api_key TEXT NOT NULL,
            encrypted_api_secret TEXT NOT NULL,
            encrypted_passphrase TEXT,
            api_key_nonce TEXT NOT NULL,
            api_secret_nonce TEXT NOT NULL,
            passphrase_nonce TEXT,
            api_key_salt TEXT NOT NULL,
            api_secret_salt TEXT NOT NULL,
            passphrase_salt TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            last_sync TEXT,
            connection_status TEXT NOT NULL DEFAULT 'pending',
            last_error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        );
    "#;

    let create_wallet_balances_table = r#"
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
    "#;

    let create_dca_strategies_table = r#"
        CREATE TABLE IF NOT EXISTS dca_strategies (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            asset_symbol TEXT NOT NULL,
            total_allocation TEXT NOT NULL,
            base_tranche_size TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            strategy_type TEXT NOT NULL DEFAULT 'adaptive_zone',
            sentiment_multiplier BOOLEAN NOT NULL DEFAULT 0,
            volatility_adjustment BOOLEAN NOT NULL DEFAULT 0,
            fear_greed_threshold_buy INTEGER NOT NULL DEFAULT 25,
            fear_greed_threshold_sell INTEGER NOT NULL DEFAULT 75,
            max_tranche_percentage TEXT NOT NULL DEFAULT '50.0',
            min_tranche_percentage TEXT NOT NULL DEFAULT '10.0',
            dca_interval_hours INTEGER NOT NULL DEFAULT 24,
            target_zones TEXT,
            stop_loss_percentage TEXT,
            take_profit_percentage TEXT,
            total_invested TEXT NOT NULL DEFAULT '0.0',
            total_purchased TEXT NOT NULL DEFAULT '0.0',
            average_buy_price TEXT,
            last_execution_at TEXT,
            next_execution_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        );
    "#;

    let create_dca_executions_table = r#"
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
    "#;

    let create_market_data_table = r#"
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
    "#;

    let create_indexes = r#"
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        CREATE INDEX IF NOT EXISTS idx_user_profiles_user_id ON user_profiles(user_id);
        CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions(session_token);
        CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions(expires_at);
        CREATE INDEX IF NOT EXISTS idx_exchange_connections_user_id ON exchange_connections(user_id);
        CREATE INDEX IF NOT EXISTS idx_exchange_connections_exchange_name ON exchange_connections(exchange_name);
        CREATE INDEX IF NOT EXISTS idx_wallet_balances_exchange_connection_id ON wallet_balances(exchange_connection_id);
        CREATE INDEX IF NOT EXISTS idx_wallet_balances_wallet_type ON wallet_balances(wallet_type);
        CREATE INDEX IF NOT EXISTS idx_wallet_balances_asset_symbol ON wallet_balances(asset_symbol);
        CREATE INDEX IF NOT EXISTS idx_dca_strategies_user_id ON dca_strategies(user_id);
        CREATE INDEX IF NOT EXISTS idx_dca_strategies_status ON dca_strategies(status);
        CREATE INDEX IF NOT EXISTS idx_dca_strategies_asset_symbol ON dca_strategies(asset_symbol);
        CREATE INDEX IF NOT EXISTS idx_dca_strategies_next_execution ON dca_strategies(next_execution_at);
        CREATE INDEX IF NOT EXISTS idx_dca_executions_strategy_id ON dca_executions(strategy_id);
        CREATE INDEX IF NOT EXISTS idx_dca_executions_timestamp ON dca_executions(execution_timestamp);
        CREATE INDEX IF NOT EXISTS idx_market_data_symbol ON market_data(asset_symbol);
        CREATE INDEX IF NOT EXISTS idx_market_data_timestamp ON market_data(timestamp);
    "#;

    // Execute table creation
    use sea_orm::ConnectionTrait;
    db.execute_unprepared(create_users_table).await?;
    db.execute_unprepared(create_user_profiles_table).await?;
    db.execute_unprepared(create_user_sessions_table).await?;
    db.execute_unprepared(create_exchange_connections_table).await?;
    db.execute_unprepared(create_wallet_balances_table).await?;
    db.execute_unprepared(create_dca_strategies_table).await?;
    db.execute_unprepared(create_dca_executions_table).await?;
    db.execute_unprepared(create_market_data_table).await?;
    db.execute_unprepared(create_indexes).await?;

    // Add the new 2FA columns if they don't exist (for existing databases)
    // First add the nullable column
    let _ = db.execute_unprepared("ALTER TABLE users ADD COLUMN totp_secret TEXT;").await;

    // Add the boolean column with default
    let _ = db.execute_unprepared("ALTER TABLE users ADD COLUMN totp_enabled BOOLEAN DEFAULT 0;").await;

    // Update any existing rows that might have NULL values
    let _ = db.execute_unprepared("UPDATE users SET totp_enabled = 0 WHERE totp_enabled IS NULL;").await;

    // Add passphrase columns for exchange connections if they don't exist
    let _ = db.execute_unprepared("ALTER TABLE exchange_connections ADD COLUMN encrypted_passphrase TEXT;").await;
    let _ = db.execute_unprepared("ALTER TABLE exchange_connections ADD COLUMN passphrase_nonce TEXT;").await;
    let _ = db.execute_unprepared("ALTER TABLE exchange_connections ADD COLUMN passphrase_salt TEXT;").await;

    Ok(db)
}

fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/signup", web::post().to(auth::signup))
            .route("/login", web::post().to(auth::login))
            .route("/logout", web::post().to(auth::logout))
            .route("/csrf-token", web::get().to(auth::get_csrf_token))
            .route("/me", web::get().to(auth::get_current_user_optional))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(AuthService::new(
                        env::var("JWT_SECRET").expect("JWT_SECRET must be set")
                    )))
                    .route("/profile", web::get().to(auth::get_current_user))
                    .route("/change-password", web::post().to(auth::change_password))
            )
    );
}

fn configure_profile_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/profile")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("", web::post().to(user_profile::create_profile))
            .route("", web::get().to(user_profile::get_profile))
            .route("", web::put().to(user_profile::update_profile))
            .route("", web::delete().to(user_profile::delete_profile))
    );
}

fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/public")
            .route("/profile/{id}", web::get().to(user_profile::get_profile_by_id))
    );
}

fn configure_2fa_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/2fa")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("/setup", web::post().to(two_factor::setup_2fa))
            .route("/verify-setup", web::post().to(two_factor::verify_2fa_setup))
            .route("/verify", web::post().to(two_factor::verify_2fa))
            .route("/disable", web::post().to(two_factor::disable_2fa))
            .route("/status", web::get().to(two_factor::get_2fa_status))
    );
}

fn configure_session_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("", web::get().to(session_management::get_active_sessions))
            .route("/{session_id}", web::delete().to(session_management::revoke_session))
            .route("/revoke-all", web::post().to(session_management::revoke_all_sessions))
    );
}

fn configure_exchange_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/exchanges")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("/connections", web::post().to(exchange_management::create_exchange_connection))
            .route("/connections", web::get().to(exchange_management::get_exchange_connections))
            .route("/connections/{connection_id}", web::put().to(exchange_management::update_exchange_connection))
            .route("/connections/{connection_id}", web::delete().to(exchange_management::delete_exchange_connection))
            .route("/connections/{connection_id}/sync", web::post().to(exchange_management::sync_exchange_balances))
            .route("/connections/{connection_id}/live-balances", web::post().to(exchange_management::get_live_wallet_balances))
            .route("/live-balances", web::post().to(exchange_management::get_all_live_user_balances))
    );
}

fn configure_dca_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dca")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("/strategies", web::post().to(dca_strategy_management::create_dca_strategy))
            .route("/strategies", web::get().to(dca_strategy_management::get_dca_strategies))
            .route("/strategies/{strategy_id}", web::get().to(dca_strategy_management::get_dca_strategy))
            .route("/strategies/{strategy_id}", web::put().to(dca_strategy_management::update_dca_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(dca_strategy_management::delete_dca_strategy))
            .route("/strategies/{strategy_id}/execute", web::post().to(dca_strategy_management::execute_dca_strategy))
            .route("/execution-stats", web::get().to(dca_strategy_management::get_execution_stats))
    );
}

fn configure_strategy_template_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/strategy-templates")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("", web::get().to(strategy_templates_handler::get_strategy_templates))
            .route("/{template_id}/backtest", web::post().to(strategy_templates_handler::run_template_backtest))
            .route("/{template_id}", web::get().to(strategy_templates_handler::get_strategy_template))
    );
}

fn configure_exchange_connector_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/exchange-accounts")
            .route("/all", web::get().to(exchange_management::get_account_by_type))
            .route("/spot", web::get().to(exchange_management::get_account_by_type))
            .route("/margin", web::get().to(exchange_management::get_account_by_type))
            .route("/futures", web::get().to(exchange_management::get_account_by_type))
            .route("/test", web::get().to(exchange_management::test_connection_status))
    );
}

fn configure_backtesting_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backtesting")
            .wrap(AuthMiddleware::new(AuthService::new(
                env::var("JWT_SECRET").expect("JWT_SECRET must be set")
            )))
            .route("/strategies", web::get().to(handlers::backtesting_handler::get_strategies))
            .route("/strategies/{strategy_name}", web::get().to(handlers::backtesting_handler::get_strategy_info))
            .route("/strategies/{strategy_name}/validate", web::post().to(handlers::backtesting_handler::validate_strategy_parameters))
            .route("/run", web::post().to(handlers::backtesting_handler::run_backtest))
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = Config::from_env().expect("Failed to load configuration");

    let db = create_database_connection(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let auth_service = AuthService::new(config.jwt_secret.clone());

    // Initialize services
    let market_service = MarketDataService::new();
    let encryption_service = EncryptionService::new();
    let strategy_template_service = StrategyTemplateService::new();

    // Initialize DCA execution engine
    let db_arc = std::sync::Arc::new(db.clone());
    let execution_engine = DCAExecutionEngine::new(
        db_arc.clone(),
        market_service.clone(),
        encryption_service.clone(),
    );

    // Start the execution engine in the background
    let engine_clone = execution_engine.clone();
    tokio::spawn(async move {
        engine_clone.start_engine().await;
    });


    info!("Starting server at {}:{}", config.server_host, config.server_port);
    info!("DCA Execution Engine started");

    // Use a consistent key for session encryption (derive from JWT secret for consistency)
    use sha2::{Sha256, Digest};

    // Generate 64 bytes by hashing the JWT secret twice
    let mut hasher1 = Sha256::new();
    hasher1.update(config.jwt_secret.as_bytes());
    hasher1.update(b"session_key_part1");
    let key_part1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(config.jwt_secret.as_bytes());
    hasher2.update(b"session_key_part2");
    let key_part2 = hasher2.finalize();

    // Combine to create 64-byte key
    let mut key_array = [0u8; 64];
    key_array[..32].copy_from_slice(&key_part1);
    key_array[32..].copy_from_slice(&key_part2);
    let secret_key = Key::from(&key_array);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&config.cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                "Authorization",
                "Content-Type",
                "Accept",
                "X-CSRF-Token",
                "X-Requested-With",
                "Origin",
                "Access-Control-Request-Method",
                "Access-Control-Request-Headers"
            ])
            .expose_headers(vec!["Set-Cookie"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(market_service.clone()))
            .app_data(web::Data::new(execution_engine.clone()))
            .app_data(web::Data::new(strategy_template_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("session".to_string())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_http_only(true)
                    .build()
            )
            .service(
                web::scope("/api/v1")
                    .wrap(SessionTrackingMiddleware)
                    .configure(configure_auth_routes)
                    .configure(configure_profile_routes)
                    .configure(configure_2fa_routes)
                    .configure(configure_session_routes)
                    .configure(configure_exchange_routes)
                    .configure(configure_dca_routes)
                    .configure(configure_strategy_template_routes)
                    .configure(configure_exchange_connector_routes)
                    .configure(configure_backtesting_routes)
                    .configure(configure_public_routes)
            )
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind(format!("{}:{}", config.server_host, config.server_port))?
    .run()
    .await
}
