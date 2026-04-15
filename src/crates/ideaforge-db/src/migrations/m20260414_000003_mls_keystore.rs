use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Password-wrapped MLS keystore (one row per user). The server never sees
/// the PIN or the wrapped plaintext — it stores only a verifier hash (for
/// gating access) and the opaque wrapped blob. Rate-limit bookkeeping lives
/// on the same row so unlock attempts can be atomically checked and updated
/// in a single query.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE mls_keystore (
                user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
                salt BYTEA NOT NULL,
                verifier BYTEA NOT NULL,
                wrapped_blob BYTEA NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                failed_attempts INT NOT NULL DEFAULT 0,
                first_failed_at TIMESTAMPTZ,
                locked_until TIMESTAMPTZ
            )",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS mls_keystore")
            .await?;
        Ok(())
    }
}
