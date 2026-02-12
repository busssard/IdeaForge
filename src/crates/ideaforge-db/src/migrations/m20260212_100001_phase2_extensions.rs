use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // --- 1. Extend idea_openness enum with 'private' ---
        conn.execute_unprepared("ALTER TYPE idea_openness ADD VALUE IF NOT EXISTS 'private'")
            .await?;

        // --- 2. Extend user_role enum with 'admin' ---
        conn.execute_unprepared("ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'admin'")
            .await?;

        // --- 3. Add bot fields to users table ---
        conn.execute_unprepared(
            "ALTER TABLE users
             ADD COLUMN IF NOT EXISTS is_bot BOOLEAN NOT NULL DEFAULT false,
             ADD COLUMN IF NOT EXISTS bot_operator VARCHAR(255),
             ADD COLUMN IF NOT EXISTS bot_description TEXT,
             ADD COLUMN IF NOT EXISTS bot_api_key_hash VARCHAR(255),
             ADD COLUMN IF NOT EXISTS skills JSONB NOT NULL DEFAULT '[]'::jsonb,
             ADD COLUMN IF NOT EXISTS looking_for TEXT,
             ADD COLUMN IF NOT EXISTS availability VARCHAR(100)"
        ).await?;

        // --- 4. Add looking_for_skills to ideas ---
        conn.execute_unprepared(
            "ALTER TABLE ideas
             ADD COLUMN IF NOT EXISTS looking_for_skills JSONB NOT NULL DEFAULT '[]'::jsonb"
        ).await?;

        // --- 5. Create invite_links table ---
        conn.execute_unprepared(
            "CREATE TYPE invite_permission AS ENUM ('view', 'comment')"
        ).await?;

        manager
            .create_table(
                Table::create()
                    .table(InviteLinks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(InviteLinks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(InviteLinks::IdeaId).uuid().not_null())
                    .col(
                        ColumnDef::new(InviteLinks::Token)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(InviteLinks::Permission)
                            .custom(Alias::new("invite_permission"))
                            .not_null()
                            .default("view"),
                    )
                    .col(ColumnDef::new(InviteLinks::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(InviteLinks::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(InviteLinks::RevokedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(InviteLinks::AccessCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InviteLinks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_links_idea")
                            .from(InviteLinks::Table, InviteLinks::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_links_creator")
                            .from(InviteLinks::Table, InviteLinks::CreatedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- 6. Create flags table ---
        conn.execute_unprepared(
            "CREATE TYPE flag_target_type AS ENUM ('idea', 'comment', 'user')"
        ).await?;

        conn.execute_unprepared(
            "CREATE TYPE flag_status AS ENUM ('pending', 'reviewed', 'dismissed')"
        ).await?;

        manager
            .create_table(
                Table::create()
                    .table(Flags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Flags::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Flags::FlaggerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Flags::TargetType)
                            .custom(Alias::new("flag_target_type"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Flags::TargetId).uuid().not_null())
                    .col(ColumnDef::new(Flags::Reason).text().not_null())
                    .col(
                        ColumnDef::new(Flags::Status)
                            .custom(Alias::new("flag_status"))
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Flags::ReviewedBy).uuid())
                    .col(
                        ColumnDef::new(Flags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flags_flagger")
                            .from(Flags::Table, Flags::FlaggerId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- 7. Create bot_endorsements table ---
        manager
            .create_table(
                Table::create()
                    .table(BotEndorsements::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(BotEndorsements::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(BotEndorsements::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(BotEndorsements::BotId).uuid().not_null())
                    .col(ColumnDef::new(BotEndorsements::Reason).text().not_null())
                    .col(
                        ColumnDef::new(BotEndorsements::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bot_endorsements_idea")
                            .from(BotEndorsements::Table, BotEndorsements::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bot_endorsements_bot")
                            .from(BotEndorsements::Table, BotEndorsements::BotId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_bot_endorsements_idea")
                    .table(BotEndorsements::Table)
                    .col(BotEndorsements::IdeaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_bot_endorsements_unique")
                    .table(BotEndorsements::Table)
                    .col(BotEndorsements::IdeaId)
                    .col(BotEndorsements::BotId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // --- 8. Create notifications table ---
        conn.execute_unprepared(
            "CREATE TYPE notification_kind AS ENUM ('stoke', 'comment', 'suggestion', 'team_application', 'team_accepted', 'team_rejected', 'milestone', 'bot_analysis', 'mention')"
        ).await?;

        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Notifications::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Notifications::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Notifications::Kind)
                            .custom(Alias::new("notification_kind"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notifications::Title).string_len(255).not_null())
                    .col(ColumnDef::new(Notifications::Message).text().not_null())
                    .col(ColumnDef::new(Notifications::LinkUrl).string_len(500))
                    .col(
                        ColumnDef::new(Notifications::ReadAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(Notifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notifications_user")
                            .from(Notifications::Table, Notifications::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_notifications_user_unread")
                    .table(Notifications::Table)
                    .col(Notifications::UserId)
                    .col(Notifications::ReadAt)
                    .to_owned(),
            )
            .await?;

        // --- 9. Add is_bot flag to contributions ---
        conn.execute_unprepared(
            "ALTER TABLE contributions
             ADD COLUMN IF NOT EXISTS is_bot BOOLEAN NOT NULL DEFAULT false"
        ).await?;

        // --- 10. Index for user skills search ---
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_users_skills ON users USING GIN (skills)"
        ).await?;

        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_users_is_bot ON users (is_bot)"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop in reverse order
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_users_is_bot").await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_users_skills").await?;
        conn.execute_unprepared("ALTER TABLE contributions DROP COLUMN IF EXISTS is_bot").await?;

        manager
            .drop_table(Table::drop().table(Notifications::Table).if_exists().to_owned())
            .await?;
        conn.execute_unprepared("DROP TYPE IF EXISTS notification_kind").await?;

        manager
            .drop_table(Table::drop().table(BotEndorsements::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Flags::Table).if_exists().to_owned())
            .await?;
        conn.execute_unprepared("DROP TYPE IF EXISTS flag_status").await?;
        conn.execute_unprepared("DROP TYPE IF EXISTS flag_target_type").await?;

        manager
            .drop_table(Table::drop().table(InviteLinks::Table).if_exists().to_owned())
            .await?;
        conn.execute_unprepared("DROP TYPE IF EXISTS invite_permission").await?;

        conn.execute_unprepared("ALTER TABLE ideas DROP COLUMN IF EXISTS looking_for_skills").await?;

        conn.execute_unprepared(
            "ALTER TABLE users
             DROP COLUMN IF EXISTS is_bot,
             DROP COLUMN IF EXISTS bot_operator,
             DROP COLUMN IF EXISTS bot_description,
             DROP COLUMN IF EXISTS bot_api_key_hash,
             DROP COLUMN IF EXISTS skills,
             DROP COLUMN IF EXISTS looking_for,
             DROP COLUMN IF EXISTS availability"
        ).await?;

        // Note: Cannot remove enum values in PostgreSQL, only drop+recreate
        // For development, this is acceptable

        Ok(())
    }
}

#[derive(DeriveIden)]
enum InviteLinks {
    Table,
    Id,
    IdeaId,
    Token,
    Permission,
    CreatedBy,
    ExpiresAt,
    RevokedAt,
    AccessCount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Flags {
    Table,
    Id,
    FlaggerId,
    TargetType,
    TargetId,
    Reason,
    Status,
    ReviewedBy,
    CreatedAt,
}

#[derive(DeriveIden)]
enum BotEndorsements {
    Table,
    Id,
    IdeaId,
    BotId,
    Reason,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    Id,
    UserId,
    Kind,
    Title,
    Message,
    LinkUrl,
    ReadAt,
    CreatedAt,
}
