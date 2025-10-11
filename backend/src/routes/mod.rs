pub mod backtesting;

use actix_web::{web, HttpResponse};
use serde_json::json;
use crate::middleware::auth::AuthMiddleware;

use crate::handlers::{
    auth, user_profile, two_factor, session_management, exchange_management,
    dca_strategy_management, rsi_strategy_management, macd_strategy_management,
    sma_crossover_strategy_management, grid_trading_strategy_management, strategy_summary,
    market_data,
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
            .configure(configure_dca_routes)
            .configure(configure_rsi_routes)
            .configure(configure_macd_routes)
            .configure(configure_sma_crossover_routes)
            .configure(configure_grid_trading_routes)
            .configure(configure_exchange_connector_routes)
            .configure(configure_backtesting_routes)
            .configure(configure_market_data_routes)
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
        "description": "Professional algorithmic trading platform with backtesting and live execution",
        "features": {
            "strategies": [
                "DCA (Dollar Cost Averaging)",
                "SMA Crossover",
                "Grid Trading",
                "RSI (Relative Strength Index)",
                "MACD (Moving Average Convergence Divergence)"
            ],
            "capabilities": [
                "Real-time strategy execution",
                "Historical backtesting",
                "Multi-exchange support",
                "Risk management",
                "Performance analytics"
            ],
            "exchanges": [
                "Binance",
                "More exchanges coming soon"
            ]
        },
        "api": {
            "version": "v1",
            "endpoints": {
                "auth": "/api/auth",
                "strategies": "/api/strategies",
                "backtesting": "/api/backtesting",
                "exchanges": "/api/exchanges",
                "profiles": "/api/profile"
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

/// Configure RSI strategy routes
fn configure_rsi_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/rsi")
            .route("/strategies", web::post().to(rsi_strategy_management::create_rsi_strategy))
            .route("/strategies", web::get().to(rsi_strategy_management::get_rsi_strategies))
            .route("/strategies/{strategy_id}", web::get().to(rsi_strategy_management::get_rsi_strategy))
            .route("/strategies/{strategy_id}", web::put().to(rsi_strategy_management::update_rsi_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(rsi_strategy_management::delete_rsi_strategy))
            .route("/execution-stats", web::get().to(rsi_strategy_management::get_rsi_execution_stats))
    );
}

/// Configure MACD strategy routes
fn configure_macd_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/macd")
            .route("/strategies", web::post().to(macd_strategy_management::create_macd_strategy))
            .route("/strategies", web::get().to(macd_strategy_management::get_macd_strategies))
            .route("/strategies/{strategy_id}", web::get().to(macd_strategy_management::get_macd_strategy))
            .route("/strategies/{strategy_id}", web::put().to(macd_strategy_management::update_macd_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(macd_strategy_management::delete_macd_strategy))
            .route("/execution-stats", web::get().to(macd_strategy_management::get_macd_execution_stats))
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
    );
}
