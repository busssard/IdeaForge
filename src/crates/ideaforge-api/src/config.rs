use serde::Deserialize;

/// Application configuration, loaded from environment or config file.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub blockchain: BlockchainConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct BlockchainConfig {
    pub network: String,
    pub blockfrost_url: String,
    pub blockfrost_project_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "postgres://ideaforge:ideaforge@localhost:5432/ideaforge".to_string(),
                max_connections: 10,
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production".to_string(),
                access_token_ttl_minutes: 15,
                refresh_token_ttl_days: 7,
            },
            blockchain: BlockchainConfig {
                network: "preview".to_string(),
                blockfrost_url: "https://cardano-preview.blockfrost.io/api/v0".to_string(),
                blockfrost_project_id: "".to_string(),
            },
        }
    }
}
