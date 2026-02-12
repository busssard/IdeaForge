use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let db = ideaforge_db::Database::connect(&database_url).await?;

    println!("Running migrations...");
    ideaforge_db::Migrator::up(db.connection(), None).await?;

    println!("Migrations applied successfully.");
    Ok(())
}
