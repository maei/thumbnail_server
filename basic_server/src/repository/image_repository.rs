use crate::AppState;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Row, Sqlite, Transaction};
use tokio::task::spawn_blocking;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct Image {
    id: i64,
    tags: String,
    thumbnail: bool, // Use Rust's bool type for clarity in your code.
}
#[async_trait]
pub trait ImageRepository: Send + Sync + 'static {
    async fn count_images(&self) -> String;
    async fn insert_image(&self, tags: &str) -> Result<i64>;
    async fn create_thumbnails(&self, thumbnail: Thumbnail) -> Result<Vec<Image>>;
    async fn update_images(&self, images: Vec<Image>) -> Result<()>;
}

type Thumbnail = fn(i64) -> Result<()>;

#[async_trait]
impl ImageRepository for AppState {
    async fn count_images(&self) -> String {
        let result = sqlx::query("SELECT COUNT(id) FROM images")
            .fetch_one(&self.db_pool)
            .await
            .unwrap();
        let count = result.get::<i64, _>(0);
        format!("{count} images in the database")
    }

    async fn insert_image(&self, tags: &str) -> Result<i64> {
        let row = sqlx::query("INSERT INTO images (tags) VALUES (?) RETURNING id")
            .bind(tags)
            .fetch_one(&self.db_pool)
            .await?;
        Ok(row.get(0))
    }

    async fn create_thumbnails(&self, thumbnail: Thumbnail) -> Result<Vec<Image>> {
        println!("Creating thumbnails.");
        let images: Vec<Image> = sqlx::query_as("SELECT * FROM images WHERE thumbnail = 0")
            .fetch_all(&self.db_pool)
            .await?;

        if images.is_empty() {
            println!("No images need thumbnails.");
            return Ok(Vec::new());
        }

        let mut handles = Vec::with_capacity(images.len());

        for image in &images {
            let id = image.id;
            let handle = spawn_blocking(move || thumbnail(id));
            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }

        Ok(images)
    }

    async fn update_images(&self, images: Vec<Image>) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        for image in &images {
            sqlx::query("UPDATE images SET thumbnail = ?, tags = ? WHERE id = ?")
                .bind(image.thumbnail)
                .bind(image.tags.clone())
                .bind(image.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
