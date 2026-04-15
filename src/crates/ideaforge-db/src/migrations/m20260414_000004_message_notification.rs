use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Add the `message` value to the notification_kind PG enum so we can fire
/// bell notifications when an MLS message or Welcome arrives.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TYPE notification_kind ADD VALUE IF NOT EXISTS 'message'",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Postgres doesn't support dropping enum values without recreating the
        // type. This migration is one-way.
        Ok(())
    }
}
