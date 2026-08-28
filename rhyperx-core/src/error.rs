use thiserror::Error;

/// Custom error type for hypergraph operations.
#[derive(Error, Debug)]
pub enum HypergraphError {
    #[error("Hyperedges cannot have duplicate nodes: {0}")]
    DuplicateNodes(usize),

    #[error("Unexpected hyperedge size: expected {expected}, got {got}")]
    InvalidHyperedgeSize { expected: usize, got: usize },

    #[error("Generic error: {0}")]
    Unknown(String),
}

/// Custom error type for types that can be serialized on disk through rkyv.
#[cfg(feature = "serialize")]
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] rkyv::rancor::Error),

    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
