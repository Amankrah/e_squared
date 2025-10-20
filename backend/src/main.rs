mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
mod exchange_connectors;
mod dex_connectors;
mod backtesting;
mod strategies;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger, cookie::Key, dev::Service as _};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use anyhow::{Result, Context};
use std::sync::Arc;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use config::Config;
use handlers::AuthService;
use middleware::{SessionTrackingMiddleware, auth::AuthMiddleware};
use routes::configure_routes;
use services::{MarketDataService, DCAExecutionEngine, DxyService, MarketIndicatorsService};
use utils::encryption::EncryptionService;

/// Initialize application services
struct AppServices {
    database: Arc<sea_orm::DatabaseConnection>,
    auth_service: AuthService,
    market_service: MarketDataService,
    execution_engine: DCAExecutionEngine,
    dxy_service: DxyService,
    market_indicators: MarketIndicatorsService,
}

impl AppServices {
    async fn new(config: &Config) -> Result<Self> {
        // Initialize database connection
        let database = Arc::new(
            database::create_connection(&config.database_url)
                .await
                .context("Failed to initialize database")?
        );

        // Initialize services
        let auth_service = AuthService::new(config.jwt_secret.clone());
        let market_service = MarketDataService::new();
        let encryption_service = EncryptionService::new();

        // Initialize trading strategies
        if let Err(e) = strategies::init_all_strategies() {
            return Err(anyhow::anyhow!("Failed to initialize trading strategies: {:?}", e));
        }

        // Initialize DCA execution engine
        let execution_engine = DCAExecutionEngine::new(
            database.clone(),
            market_service.clone(),
            encryption_service,
        );

        // Initialize DXY service
        let dxy_service = DxyService::default();

        // Initialize Market Indicators service
        let market_indicators = MarketIndicatorsService::default();

        Ok(Self {
            database,
            auth_service,
            market_service,
            execution_engine,
            dxy_service,
            market_indicators,
        })
    }

    /// Start background services
    async fn start_background_services(&self) {
        info!("Starting strategy execution engines...");
        let engine_clone = self.execution_engine.clone();
        tokio::spawn(async move {
            engine_clone.start_engine().await;
        });
    }
}

/// Initialize logging system
fn init_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    info!("Logging system initialized");
    Ok(())
}

/// Create session key from JWT secret for consistent session encryption
fn create_session_key(jwt_secret: &str) -> Key {
    use sha2::{Sha256, Digest};

    // Generate 64 bytes by hashing the JWT secret twice
    let mut hasher1 = Sha256::new();
    hasher1.update(jwt_secret.as_bytes());
    hasher1.update(b"session_key_part1");
    let key_part1 = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(jwt_secret.as_bytes());
    hasher2.update(b"session_key_part2");
    let key_part2 = hasher2.finalize();

    // Combine to create 64-byte key
    let mut key_array = [0u8; 64];
    key_array[..32].copy_from_slice(&key_part1);
    key_array[32..].copy_from_slice(&key_part2);
    
    Key::from(&key_array)
}

/// Create and configure the HTTP server
fn create_server(
    config: &Config,
    services: &AppServices,
) -> Result<actix_web::dev::Server> {
    let secret_key = create_session_key(&config.jwt_secret);
    let bind_address = format!("{}:{}", config.server_host, config.server_port);
    
    let server = HttpServer::new({
        let database = services.database.clone();
        let auth_service = services.auth_service.clone();
        let market_service = services.market_service.clone();
        let execution_engine = services.execution_engine.clone();
        let dxy_service = services.dxy_service.clone();
        let market_indicators = services.market_indicators.clone();
        // Legacy strategy_template_service removed
        let secret_key = secret_key.clone();
        let cors_origin = config.cors_origin.clone();
        
        move || {
            let cors = Cors::default()
                .allowed_origin(&cors_origin)
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
                .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(market_service.clone()))
            .app_data(web::Data::new(execution_engine.clone()))
            .app_data(web::Data::new(dxy_service.clone()))
            .app_data(web::Data::new(market_indicators.clone()))
            // Custom JSON error handler for better error logging
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        tracing::error!("JSON deserialization error: {}", err_msg);
                        actix_web::error::InternalError::from_response(
                            err,
                            actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                                "error": "Invalid JSON",
                                "message": format!("Failed to parse request body: {}", err_msg)
                            }))
                        ).into()
                    })
            )
            // Legacy strategy_template_service removed
            .wrap(cors)
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                let path = req.path().to_string();
                let method = req.method().to_string();
                tracing::info!("Incoming {} request to {}", method, path);
                let fut = srv.call(req);
                async move {
                    let res = fut.await?;
                    let status = res.status();
                    if status.is_server_error() {
                        tracing::error!("Server error {} for {} {}", status, method, path);
                    }
                    Ok(res)
                }
            })
            .wrap(
                    SessionMiddleware::builder(
                        CookieSessionStore::default(), 
                        secret_key.clone()
                    )
                    .cookie_name("session".to_string())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_http_only(true)
                    .build()
            )
                    .wrap(SessionTrackingMiddleware)
                .configure(configure_routes)
        }
    })
    .bind(&bind_address)
    .context(format!("Failed to bind server to {}", bind_address))?
    .run();
    
    info!("Server configured to run on {}", bind_address);
    Ok(server)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC OCCURRED: {:?}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("Panic at {}:{}:{}", location.file(), location.line(), location.column());
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Panic message: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Panic message: {}", s);
        }
    }));

    // Load environment variables from .env in current directory
    dotenv::dotenv().ok();

    // Initialize logging
    init_logging()
        .context("Failed to initialize logging system")?;
    
    info!("🚀 Starting E² Algorithmic Trading Platform");
    
    // Load and validate configuration
    let config = Config::from_env()
        .context("Failed to load configuration")?;
    
    config.validate()
        .context("Configuration validation failed")?;
    
    info!("✓ Configuration loaded successfully");
    info!("  - Database: {}", config.database_url);
    info!("  - Server: {}:{}", config.server_host, config.server_port);
    info!("  - CORS Origin: {}", config.cors_origin);
    
    // Initialize application services
    let services = AppServices::new(&config)
    .await
        .context("Failed to initialize application services")?;
    
    info!("✓ Application services initialized");
    
    // Start background services
    services.start_background_services().await;
    info!("✓ Background services started");
    
    // Create and start the HTTP server
    let server = create_server(&config, &services)
        .context("Failed to create HTTP server")?;
    
    info!("🌐 Server starting on {}:{}", config.server_host, config.server_port);
    info!("📚 E² Trading Platform API - Visit http://{}:{}/api/v1/docs for documentation", config.server_host, config.server_port);
    info!("🔧 Available Strategies: DCA, SMA Crossover, Grid Trading, RSI, MACD");
    info!("📊 Backtesting Engine: Active | 🤖 Execution Engine: Active");
    
    // Run the server
    server.await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
