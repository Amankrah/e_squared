pub mod backtesting;

use actix_web::web;
use std::env;

use crate::handlers::{
    auth, user_profile, two_factor, session_management, exchange_management,
    dca_strategy_management,
    AuthService
};
use crate::middleware::AuthMiddleware;

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
            .configure(configure_strategy_template_routes)
            .configure(configure_exchange_connector_routes)
            .configure(configure_backtesting_routes)
            .configure(configure_public_routes)
    )
    .route("/health", web::get().to(health_check));
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Configure authentication routes
fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/auth")
            .route("/signup", web::post().to(auth::signup))
            .route("/login", web::post().to(auth::login))
            .route("/logout", web::post().to(auth::logout))
            .route("/csrf-token", web::get().to(auth::get_csrf_token))
            .route("/me", web::get().to(auth::get_current_user_optional))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
                    .route("/profile", web::get().to(auth::get_current_user))
                    .route("/change-password", web::post().to(auth::change_password))
            )
    );
}

/// Configure user profile routes
fn configure_profile_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/profile")
            .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
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
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/2fa")
            .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
            .route("/setup", web::post().to(two_factor::setup_2fa))
            .route("/verify-setup", web::post().to(two_factor::verify_2fa_setup))
            .route("/verify", web::post().to(two_factor::verify_2fa))
            .route("/disable", web::post().to(two_factor::disable_2fa))
            .route("/status", web::get().to(two_factor::get_2fa_status))
    );
}

/// Configure session management routes
fn configure_session_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/sessions")
            .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
            .route("", web::get().to(session_management::get_active_sessions))
            .route("/{session_id}", web::delete().to(session_management::revoke_session))
            .route("/revoke-all", web::post().to(session_management::revoke_all_sessions))
    );
}

/// Configure exchange connection routes
fn configure_exchange_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/exchanges")
            .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
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
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    cfg.service(
        web::scope("/dca")
            .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
            .route("/strategies", web::post().to(dca_strategy_management::create_dca_strategy))
            .route("/strategies", web::get().to(dca_strategy_management::get_dca_strategies))
            .route("/strategies/{strategy_id}", web::get().to(dca_strategy_management::get_dca_strategy))
            .route("/strategies/{strategy_id}", web::put().to(dca_strategy_management::update_dca_strategy))
            .route("/strategies/{strategy_id}", web::delete().to(dca_strategy_management::delete_dca_strategy))
            .route("/strategies/{strategy_id}/execute", web::post().to(dca_strategy_management::execute_dca_strategy))
            .route("/execution-stats", web::get().to(dca_strategy_management::get_execution_stats))
    );
}

/// Configure strategy template routes
fn configure_strategy_template_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable is required");

    // Legacy strategy templates routes removed - using new modular strategy system
    // cfg.service(
    //     web::scope("/strategy-templates")
    //         .wrap(AuthMiddleware::new(AuthService::new(jwt_secret)))
    //         .route("", web::get().to(strategy_templates_handler::get_strategy_templates))
    //         .route("/{template_id}/backtest", web::post().to(strategy_templates_handler::run_template_backtest))
    //         .route("/{template_id}", web::get().to(strategy_templates_handler::get_strategy_template))
    // );
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

/// Configure backtesting routes
fn configure_backtesting_routes(cfg: &mut web::ServiceConfig) {
    // Use the new backtesting module
    backtesting::configure(cfg);
}
