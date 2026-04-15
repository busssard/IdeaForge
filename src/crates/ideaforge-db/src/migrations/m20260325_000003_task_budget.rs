use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Add budget_cents column (amount in cents, e.g. 50000 = $500.00)
        conn.execute_unprepared(
            "ALTER TABLE board_tasks ADD COLUMN IF NOT EXISTS budget_cents BIGINT NOT NULL DEFAULT 0"
        ).await?;

        // Add currency column (ISO 4217: USD, EUR, ADA, etc.)
        conn.execute_unprepared(
            "ALTER TABLE board_tasks ADD COLUMN IF NOT EXISTS currency VARCHAR(3) NOT NULL DEFAULT 'USD'"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("ALTER TABLE board_tasks DROP COLUMN IF EXISTS currency")
            .await?;

        conn.execute_unprepared("ALTER TABLE board_tasks DROP COLUMN IF EXISTS budget_cents")
            .await?;

        Ok(())
    }
}
