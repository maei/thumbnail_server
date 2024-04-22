// Imports necessary libraries for file handling, I/O operations, and formatting.
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Defines a custom enum for thumbnail-related errors with two variants to handle
// different types of errors: file not found and errors during processing.
#[derive(Debug)]
pub enum ThumbnailError {
    NotFound(String),
    Processing(String),
}

// Implements Display trait for ThumbnailError to enable user-friendly error messages.
impl Display for ThumbnailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbnailError::NotFound(file_name) => write!(f, "file not found: {}", file_name),
            ThumbnailError::Processing(text) => write!(f, "error while processing: {}", text),
        }
    }
}

// Implements the standard Error trait for ThumbnailError to support error handling in Rust.
impl std::error::Error for ThumbnailError {}

// Defines the Thumbnail struct. Currently, this struct does not encapsulate any data
// and serves as a namespace for the thumbnail creation functionality.
pub struct Thumbnail {}

// Implements functionality to create a thumbnail from a specified image file.
impl Thumbnail {
    /// Creates a thumbnail of an image file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A generic parameter P that must satisfy the AsRef<Path> trait, represents the path to the source image file.
    /// * `thumbnail_path` - Same as `file_path`, used to specify the path where the thumbnail will be saved.
    ///
    /// # Returns
    ///
    /// If successful, returns Ok(()). On failure, returns an error encapsulated in anyhow::Result, detailing the nature of the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let source_path = PathBuf::from("path/to/source/image.jpg");
    ///     let thumbnail_path = PathBuf::from("path/to/save/thumbnail.jpg");
    ///     super::Thumbnail::make_thumbnail(&source_path, &thumbnail_path)?;
    ///     Ok(())
    /// }
    /// ```
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

// Unit tests for the library functionality.
#[cfg(test)]
mod tests {
    // Imports all necessary components from the outer module.
    //use super::*;

    #[test]
    fn it_works() {
        // Example test function. Should be expanded with actual tests.
    }
}
