use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum ThumbnailError {
    NotFound(String),
    Processing(String),
}

impl Display for ThumbnailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbnailError::NotFound(file_name) => write!(f, "file not found: {}", file_name),
            ThumbnailError::Processing(text) => write!(f, "error while processing: {}", text),
        }
    }
}

impl std::error::Error for ThumbnailError {}

pub struct Thumbnail {}

impl Thumbnail {
    pub fn make_thumbnail<P: AsRef<Path>>(file_path: P, thumbnail_path: P) -> anyhow::Result<()> {
        let mut file = File::open(file_path.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ThumbnailError::NotFound(file_path.as_ref().to_string_lossy().into_owned())
            } else {
                ThumbnailError::Processing("Error processing file".to_string())
            }
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let image = if let Ok(format) = image::guess_format(&buffer) {
            image::load_from_memory_with_format(&buffer, format)?
        } else {
            image::load_from_memory(&buffer)?
        };

        let thumbnail = image.thumbnail(100, 100);
        thumbnail.save(thumbnail_path.as_ref())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
