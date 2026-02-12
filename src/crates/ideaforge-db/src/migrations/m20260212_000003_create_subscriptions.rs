use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subscriptions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subscriptions::UserId).uuid().not_null())
                    .col(ColumnDef::new(Subscriptions::IdeaId).uuid().not_null())
                    .col(
                        ColumnDef::new(Subscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriptions_user")
                            .from(Subscriptions::Table, Subscriptions::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriptions_idea")
                            .from(Subscriptions::Table, Subscriptions::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // UNIQUE constraint on (user_id, idea_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_user_idea_unique")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::UserId)
                    .col(Subscriptions::IdeaId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_user")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::UserId)
                    .to_owned(),
            )
            .await?;

        // Index on idea_id
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_idea")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::IdeaId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
    UserId,
    IdeaId,
    CreatedAt,
}
