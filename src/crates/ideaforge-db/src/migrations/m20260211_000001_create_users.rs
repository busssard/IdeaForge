use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE user_role AS ENUM ('entrepreneur', 'maker', 'curious')",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::PasswordHash)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::DisplayName)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::Bio).text().not_null().default(""),
                    )
                    .col(ColumnDef::new(Users::AvatarUrl).string_len(500))
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(Alias::new("user_role"))
                            .not_null()
                            .default("curious"),
                    )
                    .col(
                        ColumnDef::new(Users::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS user_role")
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    DisplayName,
    Bio,
    AvatarUrl,
    Role,
    EmailVerified,
    CreatedAt,
    UpdatedAt,
}
