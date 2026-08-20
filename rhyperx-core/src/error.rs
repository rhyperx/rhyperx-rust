use thiserror::Error;

#[derive(Error, Debug)]
pub enum HypergraphError {
    #[error("Hyperedges cannot have duplicate nodes: {0}")]
    DuplicateNodes(usize),

    #[error("Unexpected hyperedge size: expected {expected}, got {got}")]
    InvalidHyperedgeSize { expected: usize, got: usize },

    #[error("Generic error: {0}")]
    Unknown(String),
}
