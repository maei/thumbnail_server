mod repository;
mod routes;
mod service;

use std::sync::Arc;

use axum::{response::Html, routing::get, Router};
use dotenv;
use sqlx::{Pool, Sqlite};

use crate::routes::image_routes::{fill_missing_thumbnails, image_routes};

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Sqlite>,
}

impl AppState {
    fn new(db_pool: Pool<Sqlite>) -> Arc<Self> {
        Arc::new(Self { db_pool })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = AppState::new(pool);
    let app = Router::new()
        .route("/", get(index_page))
        .merge(image_routes(app_state.clone()));

    fill_missing_thumbnails(app_state.clone()).await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index_page() -> Html<String> {
    let path = std::path::Path::new("./src/templates/index.html");
    let content = tokio::fs::read_to_string(&path).await.unwrap();
    Html(content)
}
