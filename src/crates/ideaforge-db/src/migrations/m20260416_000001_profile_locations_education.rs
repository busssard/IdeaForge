use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Locations: up to 3 free-text strings (city, country, region — user's
        // choice). Stored as JSONB array for indexability and ergonomic updates.
        // Education level: free-text label (e.g. "MSc", "Self-taught", "PhD");
        // we don't enumerate because the space is culturally diverse.
        conn.execute_unprepared(
            "ALTER TABLE users
             ADD COLUMN IF NOT EXISTS locations JSONB NOT NULL DEFAULT '[]'::jsonb,
             ADD COLUMN IF NOT EXISTS education_level VARCHAR(100)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE users
             DROP COLUMN IF EXISTS locations,
             DROP COLUMN IF EXISTS education_level",
        )
        .await?;
        Ok(())
    }
}
