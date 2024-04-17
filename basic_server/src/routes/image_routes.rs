use crate::repository::image_reposiotry::ImageRepository;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

pub fn image_routes<T: ImageRepository>(repository: Arc<T>) -> Router {
    Router::new()
        .route("/images/count", get(count_images))
        .with_state(repository)
}

async fn count_images<T: ImageRepository>(State(repo): State<Arc<T>>) -> String {
    repo.count_images().await
}
