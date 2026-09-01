use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Io error occurred while accessing path: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Malformed dataset file: {0}")]
    MlformedDataset(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
