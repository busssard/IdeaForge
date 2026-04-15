use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// MLS (RFC 9420) delivery-service tables. The server stores only ciphertext
/// and group membership — it never sees plaintext. See
/// `docs/architecture/simplex_messaging_spike.md` §13–15 for the design.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // KeyPackages: short-lived handshake primitives that let other users
        // add this user to a group. Consuming a KeyPackage is a destructive
        // read — each one is single-use.
        conn.execute_unprepared(
            "CREATE TABLE mls_keypackages (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                key_package BYTEA NOT NULL,
                consumed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                expires_at TIMESTAMPTZ NOT NULL
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_mls_keypackages_available
             ON mls_keypackages (user_id, consumed_at)
             WHERE consumed_at IS NULL",
        )
        .await?;

        // Groups. `mls_group_id` is the MLS protocol-level GroupID (opaque
        // bytes). The platform-level id is our own UUID for routing.
        conn.execute_unprepared(
            "CREATE TABLE mls_groups (
                id UUID PRIMARY KEY,
                mls_group_id BYTEA NOT NULL UNIQUE,
                name TEXT,
                created_by UUID NOT NULL REFERENCES users(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        // Group membership. The DS knows who's in a group (to authorise
        // reads/posts) but not what they say.
        conn.execute_unprepared(
            "CREATE TABLE mls_group_members (
                group_id UUID NOT NULL REFERENCES mls_groups(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (group_id, user_id)
            )",
        )
        .await?;

        // Ciphertext fan-out. Both Application messages and handshake Commits
        // live here as opaque bytes. The DS does not parse them.
        conn.execute_unprepared(
            "CREATE TABLE mls_messages (
                id BIGSERIAL PRIMARY KEY,
                group_id UUID NOT NULL REFERENCES mls_groups(id) ON DELETE CASCADE,
                sender_user_id UUID NOT NULL REFERENCES users(id),
                ciphertext BYTEA NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_mls_messages_group_seq ON mls_messages (group_id, id)",
        )
        .await?;

        // Welcome messages: the out-of-band handshake primitive that adds a
        // new member to a group. Arrive on the recipient's personal queue,
        // not on the group's stream.
        conn.execute_unprepared(
            "CREATE TABLE mls_welcomes (
                id UUID PRIMARY KEY,
                recipient_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                ciphertext BYTEA NOT NULL,
                delivered_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_mls_welcomes_pending
             ON mls_welcomes (recipient_user_id, created_at)
             WHERE delivered_at IS NULL",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Order matters for FKs.
        conn.execute_unprepared("DROP TABLE IF EXISTS mls_welcomes")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS mls_messages")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS mls_group_members")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS mls_groups")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS mls_keypackages")
            .await?;

        Ok(())
    }
}
