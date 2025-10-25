use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Prop2YamlError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] io::Error),
}
