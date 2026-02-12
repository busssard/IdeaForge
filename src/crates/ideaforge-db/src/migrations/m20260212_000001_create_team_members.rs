use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE team_member_role AS ENUM ('lead', 'builder', 'advisor')",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TeamMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamMembers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamMembers::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(TeamMembers::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(TeamMembers::Role)
                            .custom(Alias::new("team_member_role"))
                            .not_null()
                            .default("builder"),
                    )
                    .col(
                        ColumnDef::new(TeamMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_team_members_idea")
                            .from(TeamMembers::Table, TeamMembers::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_team_members_user")
                            .from(TeamMembers::Table, TeamMembers::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // UNIQUE constraint on (idea_id, user_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_idea_user_unique")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::IdeaId)
                    .col(TeamMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on idea_id
        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_idea")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::IdeaId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TeamMembers::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS team_member_role")
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TeamMembers {
    Table,
    Id,
    IdeaId,
    UserId,
    Role,
    JoinedAt,
}
