use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::Path as Path2;
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::{
    extract::{multipart::Field, Multipart, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use thumbnail::{Thumbnail, ThumbnailError};
use tokio::fs::{read_to_string, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::task::spawn_blocking;
use tokio_util::io::ReaderStream;

use crate::repository::image_repository::{ImageFilter, ImageRepository, ImageResult};

pub fn image_routes<T: ImageRepository>(repository: Arc<T>) -> Router {
    Router::new()
        .route("/images/count", get(count_images))
        .route("/images/upload", post(upload_handler))
        .route("/images/:id", get(get_image))
        .with_state(repository)
}

async fn count_images<T: ImageRepository>(State(repo): State<Arc<T>>) -> String {
    repo.count().await
}

async fn insert_image_into_db<T: ImageRepository>(repo: Arc<T>, tags: &str) -> Result<i64> {
    repo.insert(tags).await
}

async fn store_image(image_id: i64, data: &[u8]) -> Result<()> {
    let file_path = Path::new("../images/").join(format!("{image_id}.jpg"));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_path)
        .await
        .context("Failed to open file for writing")?;
    file.write_all(data)
        .await
        .context("Failed to write data to file")
}

async fn get_image(Path2(id): Path2<i64>) -> impl IntoResponse {
    let filename = format!("../images/{id}.jpg");
    let attachment = format!("filename={filename}");

    match File::open(&filename).await {
        Ok(file) => {
            let reader = ReaderStream::new(file);
            let stream_body = Body::from_stream(reader);
            Response::builder()
                .header(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("image/jpeg"),
                )
                .header(
                    header::CONTENT_DISPOSITION,
                    header::HeaderValue::from_str(&attachment).unwrap(),
                )
                .body(stream_body)
                .unwrap_or_else(|_| Response::default())
        }
        Err(_) => not_found().await,
    }
}

async fn not_found() -> Response<Body> {
    let path_error = Path::new("./src/templates/file_not_found.html");
    match read_to_string(&path_error).await {
        Ok(content) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(content))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from("Error page not found."))
            .unwrap(),
    }
}

pub async fn fill_missing_thumbnails<T: ImageRepository>(repo: Arc<T>) -> Result<()> {


    let image_filter = ImageFilter {
        id: None,
        tags: None,
        thumbnail: Some(false),
    };

    let images = match repo.filter(image_filter).await? {
        ImageResult::Multiple(images) => images,
        _ => Vec::new(),
    };

    if images.is_empty() {
        println!("nothing to update");
        return Ok(());
    }

    let mut handles = Vec::with_capacity(images.len());
    let mut to_delete: Vec<i64> = Vec::new();

    for image in &images {
        let id = image.id;
        let handle = spawn_blocking(move || {
            let file_path = Path::new("../images/").join(format!("{id}.jpg"));
            let thumbnail_path = Path::new("../images/").join(format!("{id}_thumbnail.jpg"));
            match Thumbnail::make_thumbnail(file_path, thumbnail_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });
        handles.push((id, handle));
    }

    for (id, handle) in handles {
        match handle.await {
            Ok(_) => println!("Thumbnail created successfully for ID {}", id),
            Err(e) => {
                if let Some(thumbnail_error) = e.downcast_ref::<ThumbnailError>() {
                    match thumbnail_error {
                        ThumbnailError::NotFound(text) => {
                            println!("File not found: {}, delete it from db...", text);
                            to_delete.push(id);
                        }
                        ThumbnailError::Processing(text) => {
                            println!("Processing error: {}", text)
                        }
                    }
                } else {
                    println!("Unhandled error type");
                }
            }
        }
    }

    // TODO if postgres use BULK operations
    for id in &to_delete {
        match repo.delete(*id).await {
            Ok(_) => {
                println!("Image {id} deleted successfully.");
            }
            Err(e) => {
                eprintln!("Failed to delete image {id}: {e}. Will attempt to delete later.");
                // push the id back to a retry queue or log it to a file/database for later retry.
            }
        }
    }

    Ok(())
}

async fn upload_handler<T: ImageRepository>(
    State(repo): State<Arc<T>>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut tags = None;
    let mut image_data = None;
    //let mut file_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("tags") => {
                let bytes = field.bytes().await.expect("Failed to read bytes for tags");
                tags = Some(
                    String::from_utf8(bytes.to_vec()).expect("Failed to decode tags from UTF-8"),
                );
            }
            Some("file") => {
                //file_name = field.file_name().map(|s| s.to_string());
                let bytes = field.bytes().await.expect("Failed to read bytes for files");
                image_data = Some(bytes.to_vec());
            }
            _ => eprintln!("Unsupported field received"),
        }
    }

    if let (Some(tags), Some(image)) = (tags, image_data) {
        let image_id = insert_image_into_db(repo, &tags).await.unwrap();
        println!("id is {}", image_id);

        store_image(image_id, &image)
            .await
            .expect("error while storing file");
    }

    let path_success = Path::new("./src/templates/upload.html");
    let path_error = Path::new("./src/templates/upload_error.html");

    match read_to_string(&path_success).await {
        Ok(content) => Html(content),
        Err(_) => {
            let content = read_to_string(&path_error).await.unwrap();
            Html(content)
        }
    }
}

async fn uploader_chunks(mut multipart: Multipart) -> impl IntoResponse {
    let mut tags = None;
    let mut image = None;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "tags" => {
                println!("got form field tags: {}", name);
                let data = match field.bytes().await {
                    Ok(data) => data,
                    Err(_) => return Html("Error reading tags data".to_string()),
                };
                tags = Some(String::from_utf8(data.to_vec()).unwrap_or_default());
            }
            "file" => {
                println!("got form field file: {}", name);
                if let Some(file_name) = field.file_name().map(|n| n.to_string()) {
                    let file_path = Path::new("../images/").join(&file_name);
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&file_path)
                        .await
                        .expect("sadad");

                    let mut total_bytes = 0usize;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        file.write_all(&chunk)
                            .await
                            .expect("Failed to write to file");
                        let chunk_size = chunk.len();
                        total_bytes += chunk_size;
                        println!("Received {} bytes (total: {})", chunk_size, total_bytes);
                    }
                    println!("Finished file: {} ({} bytes)", file_name, total_bytes);
                    image = Some(true);
                } else {
                    println!("shit happens, over and over again")
                }
            }
            _ => return Html(format!("Unknown field: {}", name)),
        }
    }
    println!("Exited the loop");

    match (tags, image) {
        (Some(tags), Some(_)) => {
            println!("Received tags: {} and image", tags);
            Html("Ok".to_string())
        }
        _ => Html("Missing field".to_string()),
    }
}

#[derive(Debug, PartialEq)]
enum ContentType {
    ImagePng,
    ImageJpg,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::ImagePng => write!(f, "content type PNG"),
            ContentType::ImageJpg => write!(f, "content type JPG"),
        }
    }
}

fn get_content_type(field: &dyn FieldBehavior) -> Option<ContentType> {
    field
        .content_type()
        .map(|mime| mime.as_ref())
        .and_then(|mime_str| match mime_str {
            "image/png" => Some(ContentType::ImagePng),
            "image/jpg" => Some(ContentType::ImageJpg),
            _ => None,
        })
}

trait FieldBehavior {
    fn content_type(&self) -> Option<&str>;
}

impl FieldBehavior for Field<'_> {
    fn content_type(&self) -> Option<&str> {
        self.content_type().map(|mime| mime.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockField {
        content_type: Option<String>,
    }

    impl FieldBehavior for MockField {
        fn content_type(&self) -> Option<&str> {
            self.content_type.as_deref()
        }
    }

    #[test]
    fn test_get_content_type_png() {
        let field = MockField {
            content_type: Some("image/png".to_string()),
        };
        assert_eq!(get_content_type(&field), Some(ContentType::ImagePng));
    }

    #[test]
    fn test_get_content_type_jpg() {
        let field = MockField {
            content_type: Some("image/jpg".to_string()),
        };
        assert_eq!(get_content_type(&field), Some(ContentType::ImageJpg));
    }

    #[test]
    fn test_get_content_type_unsupported() {
        let field = MockField {
            content_type: Some("text/html".to_string()),
        };
        assert_eq!(get_content_type(&field), None);
    }
}
