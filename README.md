# Basic Server
[Ardan Labs Git](https://github.com/thebracket/ArdanUltimateRust-5Days/blob/main/03-Async/ThumbnailServer.md)

## Dependencies

```rust
cargo add tokio -F full
cargo add serde -F derive
cargo add axum -F multipart
cargo add sqlx -F runtime-tokio-native-tls -F sqlite
cargo add anyhow
cargo add dotenv
cargo add futures
cargo add tokio_util -F io
cargo add image
```

## Create Database

1. add .env with DATABASE_URL -> ``DATABASE_URL="sqlite:images.db"``
2. Database [sqlx-cli](https://crates.io/crates/sqlx-cli) \
 2.1 create database `sqlx database create` \
 2.2 migrate database ``sqlx migrate add initial``\
 2.3 add sql 

```sql
-- Create images table
CREATE TABLE IF NOT EXISTS images
(
id          INTEGER PRIMARY KEY NOT NULL,
tags        TEXT                NOT NULL
);
```

3. Build migration to rust directly
```rust
    sqlx::migrate!("./migrations").run(&pool).await?;
```

4. Use Axum Dependency Injection
- state is recommended [https://docs.rs/axum/latest/axum/#using-the-state-extractor](https://docs.rs/axum/latest/axum/#using-the-state-extractor)

4.1. DJ on whole app 
````rust
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use dotenv;
use sqlx::{Pool, Row, Sqlite};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new().route("/", get(test)).with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn test(State(pool): State<Pool<Sqlite>>) -> String {
    let result = sqlx::query("SELECT COUNT(id) FROM images")
        .fetch_one(&pool)
        .await
        .unwrap();
    let count = result.get::<i64, _>(0);
    format!("{count} images in the database")
}
````
4.2. DJ only on specific route
````rust
    let app = Router::new().route("/", get(test).with_state(pool));
````

5. Build HTML for index 
````rust
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use dotenv;
use sqlx::{Pool, Row, Sqlite};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::SqlitePool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/", get(index_page))
        .route("/test", get(test))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index_page() -> Html<String> {
    let path = std::path::Path::new("./src/templates/index.html");
    let content = tokio::fs::read_to_string(&path).await.unwrap();
    Html(content)
}

async fn test(State(pool): State<Pool<Sqlite>>) -> String {
    let result = sqlx::query("SELECT COUNT(id) FROM images")
        .fetch_one(&pool)
        .await
        .unwrap();
    let count = result.get::<i64, _>(0);
    format!("{count} images in the database")
}
````



