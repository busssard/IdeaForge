use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let db = ideaforge_db::Database::connect(&database_url).await?;

    println!("Running migrations...");
    db.run_migrations().await?;

    println!("Seeding database...");
    ideaforge_db::seed::seed_database(db.connection()).await?;

    println!("Seed complete.");
    Ok(())
}
