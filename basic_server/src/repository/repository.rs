use crate::AppState;
use sqlx::{Row};

pub trait ImageRepository: Send + Sync + 'static {
    async fn count_images(&self) -> String;
}

impl ImageRepository for AppState {
    async fn count_images(&self) -> String {
        let result = sqlx::query("SELECT COUNT(id) FROM images")
            .fetch_one(&self.db_pool)
            .await
            .unwrap();
        let count = result.get::<i64, _>(0);
        format!("{count} images in the database")
    }
}
