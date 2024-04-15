# Basic Server
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
````rust

````