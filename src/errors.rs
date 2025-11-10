use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Prop2YamlError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid property line: {0}")]
    InvalidPropertyLine(String),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("yamlfmt failed with exit code {0}")]
    YamlfmtFailed(i32),

    #[error("yamlfmt error: {0}")]
    YamlfmtNotFound(String),

    #[error("Unsupported YAML value type: {0}")]
    UnsupportedYamlValueType(String),
}
