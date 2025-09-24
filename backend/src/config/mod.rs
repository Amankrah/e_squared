use serde::Deserialize;
use std::env;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origin: String,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .or_else(|_| env::var("database_url"))
            .unwrap_or_else(|_| "sqlite:./e_squared.db".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .or_else(|_| env::var("jwt_secret"))
            .context("JWT_SECRET environment variable is required")?;

        let server_host = env::var("SERVER_HOST")
            .or_else(|_| env::var("server_host"))
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .or_else(|_| env::var("server_port"))
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("SERVER_PORT must be a valid port number")?;

        let cors_origin = env::var("CORS_ORIGIN")
            .or_else(|_| env::var("cors_origin"))
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        Ok(Config {
            database_url,
            jwt_secret,
            server_host,
            server_port,
            cors_origin,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters long");
        }

        if self.server_port == 0 {
            anyhow::bail!("SERVER_PORT must be a valid port number");
        }

        // Validate CORS origin format
        if !self.cors_origin.starts_with("http://") && !self.cors_origin.starts_with("https://") {
            anyhow::bail!("CORS_ORIGIN must start with http:// or https://");
        }

        Ok(())
    }
}