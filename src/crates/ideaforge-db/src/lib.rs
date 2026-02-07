//! IdeaForge Database - SeaORM entities, migrations, and repository layer.

pub mod entities;
pub mod migrations;
pub mod repositories;

use sea_orm::DatabaseConnection;

/// Shared database state, wrapping the SeaORM connection pool.
#[derive(Debug, Clone)]
pub struct Database {
    pub conn: DatabaseConnection,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sea_orm::DbErr> {
        let conn = sea_orm::Database::connect(database_url).await?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}
