use dotenv;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    println!("{db_url}");
    // let pool = sqlx::SqlitePool(&db_url).await?;

    Ok(())
}
