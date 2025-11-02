pub mod backtesting;

use actix_web::{web, HttpResponse};
use serde_json::json;
use crate::middleware::auth::AuthMiddleware;

use crate::handlers::{
    auth, user_profile, two_factor, session_management, exchange_management, wallet_management,
    dca_strategy_management, sma_crossover_strategy_management,
    grid_trading_strategy_management, strategy_summary, market_data, stock_data,
};

/// Configure all application routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(configure_auth_routes)
            .configure(configure_profile_routes)
            .configure(configure_2fa_routes)
            .configure(configure_session_routes)
            .configure(configure_exchange_routes)
            .configure(configure_wallet_routes)
            .configure(configure_dca_routes)
            .configure(configure_sma_crossover_routes)
            .configure(configure_grid_trading_routes)
            .configure(configure_exchange_connector_routes)
            .configure(configure_backtesting_routes)
            .configure(configure_market_data_routes)
            .configure(configure_stock_data_routes)
            .configure(configure_public_routes)
    )
    .route("/health", web::get().to(health_check));
}

/// Health check endpoint with API information
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "platform": "E² Algorithmic Trading Platform",
        "version": "1.0.0",
        "description": "Professional algorithmic trading platform for CEX & DEX with backtesting and automated execution",
        "features": {
            "strategies": [
                "DCA (Dollar Cost Averaging)",
                "Grid Trading",
                "SMA Crossover"
            ],
            "capabilities": [
                "Automated strategy execution",
                "Historical backtesting",
                "CEX & DEX support",
                "Non-custodial trading",
                "Real-time market indicators",
                "Performance analytics"
            ],
            "platforms": {
                "cex": [
                    "Binance",
                    "Coinbase (coming soon)",
                    "Kraken (coming soon)"
                ],
                "dex": [
                    "Uniswap (coming soon)",
                    "PancakeSwap (coming soon)",
                    "Wallet Connect integration"
                ]
            }
        },
        "market_data": {
            "indicators": [
                "DXY (US Dollar Index)",
                "Bitcoin Price",
                "Bitcoin Dominance",
                "M2 Money Supply"
            ]
        },
        "api": {
            "version": "v1",
            "endpoints": {
                "auth": "/api/v1/auth",
                "strategies": "/api/v1/strategies",
                "backtesting": "/api/v1/backtesting",
                "exchanges": "/api/v1/exchanges",
                "market_data": "/api/v1/market-data",
                "profile": "/api/v1/profile"
            }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Configure authentication routes
fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/signup", web::post().to(auth::signup))
            .route("/login", web::post().to(auth::login))
            .route("/logout", web::post().to(auth::logout))
            .route("/csrf-token", web::get().to(auth::get_csrf_token))
            .route("/me", web::get().to(auth::get_current_user_optional))
            .route("/strategy-summary", web::get().to(strategy_summary::get_user_strategy_summary))
            .service(
                web::scope("")
                    .route("/profile", web::get().to(auth::get_current_user))
                    .route("/change-password", web::post().to(auth::change_password))
            )
    );
}

/// Configure user profile routes
fn configure_profile_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/profile")
            .route("", web::post().to(user_profile::create_profile))
            .route("", web::get().to(user_profile::get_profile))
            .route("", web::put().to(user_profile::update_profile))
            .route("", web::delete().to(user_profile::delete_profile))
    );
}

/// Configure public routes (no authentication required)
fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/public")
            .route("/profile/{id}", web::get().to(user_profile::get_profile_by_id))
    );
}

/// Configure two-factor authentication routes
fn configure_2fa_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/2fa")
            .route("/setup", web::post().to(two_factor::setup_2fa))
            .route("/verify-setup", web::post().to(two_factor::verify_2fa_setup))
            .route("/verify", web::post().to(two_factor::verify_2fa))
            .route("/disable", web::post().to(two_factor::disable_2fa))
            .route("/status", web::get().to(two_factor::get_2fa_status))
    );
}

/// Configure session management routes
fn configure_session_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .route("", web::get().to(session_management::get_active_sessions))
            .route("/{session_id}", web::delete().to(session_management::revoke_session))
            .route("/revoke-all", web::post().to(session_management::revoke_all_sessions))
    );
}

/// Configure exchange connection routes
fn configure_exchange_routes(cfg: &mut web::ServiceConfig) {
    tracing::info!("Configuring exchange routes...");
    cfg.service(
        web::scope("/exchanges")
            .route("/connections", web::post().to(exchange_management::create_exchange_connection))
            .route("/connections", web::get().to(exchange_management::get_exchange_connections))
            .route("/connections/{connection_id}", web::put().to(exchange_management::update_exchange_connection))
            .route("/connections/{connection_id}", web::delete().to(exchange_management::delete_exchange_connection))
            .route("/connections/{connection_id}/sync", web::post().to(exchange_management::sync_exchange_balances))
            .route("/connections/{connection_id}/live-balances", web::post().to(exchange_management::get_live_wallet_balances))
            .route("/live-balances", web::post().to(exchange_management::get_all_live_user_balances))
    );
    tracing::info!("Exchange routes configured");
}

/// Configure DCA strategy routes
fn configure_dca_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dca")
            .route("/strategies", web::post().to(dca_strategy_management::create_dca_strategy))
            .route("/strategies", web::get().to(dca_strategy_management::get_dca_strategies))
            .route("/strategies/from-preset", web::post().to(dca_strategy_management::create_dca_strategy_from_preset))
            .route("/strategies/{strategy_id}", web::get().to(dca_strategy_management::get_dca_strategy))
            .route("/strategies/{strategy_id}", web::put().to(dca_strategy_management::update_dca_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(dca_strategy_management::delete_dca_strategy))
            .route("/strategies/{strategy_id}/execute", web::post().to(dca_strategy_management::execute_dca_strategy))
            .route("/execution-stats", web::get().to(dca_strategy_management::get_execution_stats))
            .route("/presets", web::get().to(dca_strategy_management::get_dca_presets))
    );
}

/// Configure exchange connector routes
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

/// Configure SMA Crossover strategy routes
fn configure_sma_crossover_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sma-crossover")
            .route("/strategies", web::post().to(sma_crossover_strategy_management::create_sma_crossover_strategy))
            .route("/strategies", web::get().to(sma_crossover_strategy_management::get_user_sma_crossover_strategies))
            .route("/strategies/{strategy_id}", web::get().to(sma_crossover_strategy_management::get_sma_crossover_strategy))
            .route("/strategies/{strategy_id}", web::put().to(sma_crossover_strategy_management::update_sma_crossover_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(sma_crossover_strategy_management::delete_sma_crossover_strategy))
            .route("/strategies/{strategy_id}/pause", web::post().to(sma_crossover_strategy_management::pause_sma_crossover_strategy))
            .route("/strategies/{strategy_id}/resume", web::post().to(sma_crossover_strategy_management::resume_sma_crossover_strategy))
    );
}

/// Configure Grid Trading strategy routes
fn configure_grid_trading_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/grid-trading")
            .route("/strategies", web::post().to(grid_trading_strategy_management::create_grid_trading_strategy))
            .route("/strategies", web::get().to(grid_trading_strategy_management::get_grid_trading_strategies))
            .route("/strategies/{strategy_id}", web::get().to(grid_trading_strategy_management::get_grid_trading_strategy))
            .route("/strategies/{strategy_id}", web::put().to(grid_trading_strategy_management::update_grid_trading_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(grid_trading_strategy_management::delete_grid_trading_strategy))
            .route("/execution-stats", web::get().to(grid_trading_strategy_management::get_grid_trading_execution_stats))
    );
}

/// Configure backtesting routes
fn configure_backtesting_routes(cfg: &mut web::ServiceConfig) {
    // Use the new backtesting module
    backtesting::configure(cfg);
}

/// Configure market data routes
fn configure_market_data_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/market-data")
            .route("/{symbol}/current", web::get().to(market_data::get_current_price))
            .route("/dxy", web::get().to(market_data::get_dxy))
            .route("/btc-dominance", web::get().to(market_data::get_btc_dominance))
            .route("/m2", web::get().to(market_data::get_m2))
            .route("/btc-price", web::get().to(market_data::get_btc_price))
            .route("/fear-greed", web::get().to(market_data::get_fear_greed_index))
    );
}

/// Configure stock data routes
fn configure_stock_data_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stocks")
            .route("/{symbol}/price", web::get().to(stock_data::get_stock_price))
            .route("/{symbol}/historical", web::get().to(stock_data::get_stock_historical))
    );
}

/// Configure wallet connection routes
fn configure_wallet_routes(cfg: &mut web::ServiceConfig) {
    tracing::info!("Configuring wallet routes...");
    cfg.service(
        web::scope("/wallets")
            .route("/connections", web::post().to(wallet_management::create_wallet_connection))
            .route("/connections", web::get().to(wallet_management::get_wallet_connections))
            .route("/connections/{connection_id}", web::get().to(wallet_management::get_wallet_connection))
            .route("/connections/{connection_id}", web::put().to(wallet_management::update_wallet_connection))
            .route("/connections/{connection_id}", web::delete().to(wallet_management::delete_wallet_connection))
            .route("/connections/{connection_id}/balance", web::post().to(wallet_management::get_wallet_balance))
    );
    tracing::info!("Wallet routes configured");
}
