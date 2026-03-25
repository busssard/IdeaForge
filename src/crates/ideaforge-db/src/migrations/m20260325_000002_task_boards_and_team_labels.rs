use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // --- 1. Add role_label column to team_members ---
        conn.execute_unprepared(
            "ALTER TABLE team_members ADD COLUMN IF NOT EXISTS role_label VARCHAR(100)"
        ).await?;

        // --- 2. Create task_status enum ---
        conn.execute_unprepared(
            "CREATE TYPE task_status AS ENUM ('open', 'assigned', 'in_review', 'done')"
        ).await?;

        // --- 3. Create task_priority enum ---
        conn.execute_unprepared(
            "CREATE TYPE task_priority AS ENUM ('low', 'normal', 'high', 'urgent')"
        ).await?;

        // --- 4. Create board_tasks table ---
        manager
            .create_table(
                Table::create()
                    .table(BoardTasks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(BoardTasks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(BoardTasks::IdeaId).uuid().not_null())
                    .col(
                        ColumnDef::new(BoardTasks::Title)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(BoardTasks::Description).text())
                    .col(
                        ColumnDef::new(BoardTasks::Status)
                            .custom(Alias::new("task_status"))
                            .not_null()
                            .default("open"),
                    )
                    .col(
                        ColumnDef::new(BoardTasks::Priority)
                            .custom(Alias::new("task_priority"))
                            .not_null()
                            .default("normal"),
                    )
                    .col(ColumnDef::new(BoardTasks::AssigneeId).uuid())
                    .col(ColumnDef::new(BoardTasks::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(BoardTasks::SkillTags)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .col(ColumnDef::new(BoardTasks::DueDate).date())
                    .col(
                        ColumnDef::new(BoardTasks::Position)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BoardTasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BoardTasks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(BoardTasks::CompletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_tasks_idea")
                            .from(BoardTasks::Table, BoardTasks::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_tasks_assignee")
                            .from(BoardTasks::Table, BoardTasks::AssigneeId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_tasks_creator")
                            .from(BoardTasks::Table, BoardTasks::CreatedBy)
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // --- 5. Indexes ---
        manager
            .create_index(
                Index::create()
                    .name("idx_board_tasks_idea")
                    .table(BoardTasks::Table)
                    .col(BoardTasks::IdeaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_board_tasks_assignee")
                    .table(BoardTasks::Table)
                    .col(BoardTasks::AssigneeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_board_tasks_status")
                    .table(BoardTasks::Table)
                    .col(BoardTasks::IdeaId)
                    .col(BoardTasks::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop in reverse order
        manager
            .drop_table(Table::drop().table(BoardTasks::Table).if_exists().to_owned())
            .await?;

        conn.execute_unprepared("DROP TYPE IF EXISTS task_priority").await?;
        conn.execute_unprepared("DROP TYPE IF EXISTS task_status").await?;

        conn.execute_unprepared(
            "ALTER TABLE team_members DROP COLUMN IF EXISTS role_label"
        ).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BoardTasks {
    Table,
    Id,
    IdeaId,
    Title,
    Description,
    Status,
    Priority,
    AssigneeId,
    CreatedBy,
    SkillTags,
    DueDate,
    Position,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
}
