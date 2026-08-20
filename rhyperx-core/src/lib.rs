pub mod error;
pub mod misc;

#[cfg(test)]
pub mod tests;
// pub mod graph;
pub mod hypergraph;
pub mod types;

// pub use graph::*;
pub use hypergraph::*;

// pub type NodeId = u32;
// pub type EdgeId = u32;
// pub type NodeWeight = f32;
