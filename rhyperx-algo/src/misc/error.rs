use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] rkyv::rancor::Error),

    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
