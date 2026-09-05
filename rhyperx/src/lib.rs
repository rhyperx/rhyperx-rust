//! # rhyperx
//!
//! Umbrella/facade crate for the [`rhyperx`] hypergraph library.
//!
//! This crate is the single entry point for downstream users: it re-exports
//! the public API of the whole workspace, so that only `rhyperx` needs to be
//! added as a dependency.
//!
//! * **Data structures** (`rhyperx-core`): the `graph`, `hypergraph`, `motif`,
//!   `collections`, `types`, `error`, `misc` and `util` modules are re-exported
//!   at the crate root.
//! * **Algorithms** (`rhyperx-algo`): exposed under the [`algo`] module.
//! * **Dataset loaders** (`rhyperx-io`): exposed under the [`io`] module.
//!
//! # Feature flags
//!
//! | Feature     | Default | Description                                             |
//! |-------------|---------|---------------------------------------------------------|
//! | `algo`      | yes     | Re-exports [`rhyperx-algo`] under the [`algo`] module.  |
//! | `io`        | no      | Re-exports [`rhyperx-io`] under the [`io`] module.      |
//! | `serialize` | no      | Enables rkyv-based serialization in `rhyperx-core`.     |

pub use rhyperx_core::{CompactMotif, compact_motif, iter_hyperedges};
pub use rhyperx_core::{collections, error, graph, hypergraph, misc, motif, types, util};

/// Algorithms implemented on top of the core data structures.
#[cfg(feature = "algo")]
pub mod algo {
    pub use rhyperx_algo::*;
}

/// Dataset loaders and I/O utilities.
#[cfg(feature = "io")]
pub mod io {
    pub use rhyperx_io::*;
}
