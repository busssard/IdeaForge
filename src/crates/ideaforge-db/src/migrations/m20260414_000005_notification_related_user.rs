use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Adds `related_user_id` to notifications so we can coalesce per-sender.
/// e.g. "6 new messages from Bob" instead of 6 rows of "New message from Bob".
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "ALTER TABLE notifications ADD COLUMN IF NOT EXISTS related_user_id UUID",
        )
        .await?;

        // Counter field — when coalesced, the title becomes "N new messages
        // from X" where N is count. Default of 1 so existing rows read
        // naturally.
        conn.execute_unprepared(
            "ALTER TABLE notifications ADD COLUMN IF NOT EXISTS count INTEGER NOT NULL DEFAULT 1",
        )
        .await?;

        // Index covering the coalescing query: find unread notifications of a
        // given kind for a given recipient from a given related user.
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_notifications_coalesce \
             ON notifications (user_id, kind, related_user_id, read_at)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_notifications_coalesce")
            .await?;
        conn.execute_unprepared("ALTER TABLE notifications DROP COLUMN IF EXISTS count")
            .await?;
        conn.execute_unprepared(
            "ALTER TABLE notifications DROP COLUMN IF EXISTS related_user_id",
        )
        .await?;
        Ok(())
    }
}
