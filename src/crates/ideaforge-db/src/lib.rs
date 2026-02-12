pub mod entities;
pub mod migrations;
pub mod repositories;
pub mod seed;

use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

pub use migrations::Migrator;

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

    /// Run all pending migrations.
    pub async fn run_migrations(&self) -> Result<(), sea_orm::DbErr> {
        Migrator::up(&self.conn, None).await?;
        Ok(())
    }
}
