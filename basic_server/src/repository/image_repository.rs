use crate::AppState;
use async_trait::async_trait;
use sqlx::Row;
#[async_trait]
pub trait ImageRepository: Send + Sync + 'static {
    async fn count_images(&self) -> String;
}

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
}
