use crate::AppState;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteArguments;
use sqlx::{Arguments, Row};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Image {
    pub(crate) id: i64,
    pub tags: String,
    pub thumbnail: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct ImageFilter {
    pub id: Option<i64>,
    pub tags: Option<String>,
    pub thumbnail: Option<bool>,
}

#[derive(Debug)]
pub enum ImageResult {
    Single(Image),
    Multiple(Vec<Image>),
}

#[async_trait]
pub trait ImageRepository: Send + Sync + 'static {
    async fn count(&self) -> String;
    async fn insert(&self, tags: &str) -> Result<i64>;
    async fn delete(&self, id: i64) -> Result<()>;
    async fn update(&self, image: Image) -> Result<()>;
    async fn filter(&self, filter: ImageFilter) -> Result<ImageResult>;
}

type Thumbnail = fn(i64) -> Result<()>;

#[async_trait]
impl ImageRepository for AppState {
    async fn count(&self) -> String {
        let result = sqlx::query("SELECT COUNT(id) FROM images")
            .fetch_one(&self.db_pool)
            .await
            .unwrap();
        let count = result.get::<i64, _>(0);
        format!("{count} images in the database")
    }

    async fn insert(&self, tags: &str) -> Result<i64> {
        let row = sqlx::query("INSERT INTO images (tags) VALUES (?) RETURNING id")
            .bind(tags)
            .fetch_one(&self.db_pool)
            .await?;
        Ok(row.get(0))
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM images WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn update(&self, image: Image) -> Result<()> {
        println!("update");
        sqlx::query("UPDATE images SET thumbnail = ?, tags = ? WHERE id = ?")
            .bind(image.thumbnail)
            .bind(image.tags.clone())
            .bind(image.id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    async fn filter(&self, filters: ImageFilter) -> Result<ImageResult> {
        let mut query = "SELECT * FROM images WHERE 1 = 1".to_string();
        let mut args = SqliteArguments::default();

        if let Some(ref tags) = filters.tags {
            query += " AND tags LIKE ?";
            args.add(format!("%{}%", tags));
        }

        if let Some(thumbnail) = filters.thumbnail {
            query += " AND thumbnail = ?";
            args.add(thumbnail as i64);
        }

        if let Some(id) = filters.id {
            query += " AND id = ?";
            args.add(id);
        }

        let images = sqlx::query_as_with::<_, Image, _>(&query, args)
            .fetch_all(&self.db_pool)
            .await
            .context("failed to fetch")?;

        match filters.id {
            None => Ok(ImageResult::Multiple(images)),
            Some(_) if images.len() == 1 => Ok(ImageResult::Single(images[0].clone())),
            Some(_) => Ok(ImageResult::Multiple(images)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
