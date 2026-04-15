use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Lifecycle is orthogonal to maturity: maturity = quality of the idea
        // (spark → in_work), lifecycle = execution state (not_started →
        // ongoing → finished).
        conn.execute_unprepared(
            "CREATE TYPE idea_lifecycle AS ENUM ('not_started', 'ongoing', 'finished')",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE ideas ADD COLUMN IF NOT EXISTS lifecycle idea_lifecycle NOT NULL DEFAULT 'not_started'",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_ideas_lifecycle ON ideas (lifecycle)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("DROP INDEX IF EXISTS idx_ideas_lifecycle")
            .await?;

        conn.execute_unprepared("ALTER TABLE ideas DROP COLUMN IF EXISTS lifecycle")
            .await?;

        conn.execute_unprepared("DROP TYPE IF EXISTS idea_lifecycle")
            .await?;

        Ok(())
    }
}
