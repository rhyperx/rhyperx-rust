# rhyperx-rust

Rust hypergraph library. Workspace with 6 crates, edition 2024, resolver "3".

## Workspace members

| Crate | Purpose |
|---|---|
| `rhyperx` | Umbrella facade crate. Currently empty (`pub fn main(){}`). |
| `rhyperx-core` | Core data structures: `graph/`, `hypergraph/`, `types`, `error`. Entrypoint. |
| `rhyperx-algo` | Algorithms: `bin_store/` (compact motif, bin store, node set), `util/` (const ops, permutations, sorting networks, misc). The `motifs/` module is commented out in lib.rs — do **not** add `pub mod motifs` without checking whether its code compiles (it imports `crate::motifs::` from within `motifs/` which is circular). |
| `rhyperx-macros` | Proc-macro crate. 6 attributes: `#[repeat]`, `#[hoist_mod]`, `#[inherent]`, `#[ct_map]`, `#[ct_map_accessor]`, `#[loaders]`, `#[remove_attr]`. |
| `rhyperx-io` | I/O and serialization. Currently a placeholder. |
| `rhyperx-tests` | Benchmarks & integration tests on real datasets (publish=false). |

## Conventions

* **Safety and API usability:** The library should be as fool-proof as reasonably possible. Prefer APIs and implementations that make incorrect or unexpectedly expensive usage difficult for the end user. If two approaches have comparable raw performance, prefer the one that provides stronger safety guarantees and makes misuse harder.

* **Performance first:** This is a performance-critical library. Do not knowingly introduce sub-optimal algorithms, unnecessary allocations, avoidable copies, or other performance regressions. When choosing between implementations with meaningfully different performance characteristics, prefer the faster one. If you are uncertain about the algorithmic complexity or performance of an approach, **ask before proceeding** rather than guessing.

* **Performance-oriented or hacky code:** Achieving the required performance may sometimes require code that is less idiomatic, less elegant, or harder to maintain. **Do not introduce such code without asking first.** Before resorting to a hack, consider whether the same performance can be achieved cleanly using existing Rust features, declarative macros, procedural macros, or other compile-time techniques. If a macro-based solution could make performance-oriented code substantially cleaner, propose it and ask before proceeding.

* **Modularity and crate boundaries:** Keep the code modular, well-organized, and consistent with the workspace architecture. Every implementation should live in the crate that is responsible for that functionality. Avoid introducing dependencies between crates when the functionality can reasonably remain isolated.

* **Respect task scope:** When the user specifies a crate or scope, restrict investigation, modifications, and validation to that scope whenever possible. Do not inspect or modify the entire Cargo workspace by default. Inspect other crates only when necessary to understand or validate a dependency of the code being changed.

## Tool Usage

Prefer specialized tools over generic shell commands when they provide the required functionality.

* Use **LSP** for semantic Rust operations such as:

  * finding definitions,
  * finding references,
  * inspecting types,
  * obtaining Rust diagnostics,
  * navigating symbols.

* Use **Rust MCP tools** for Cargo/project operations when an appropriate tool is available, such as:

  * checking,
  * testing,
  * linting,
  * formatting,
  * benchmarking.

* Use **Bash** for operations that are inherently shell-based or are not provided by a specialized tool, such as:

  * `git` operations,
  * `rg`/text search,
  * inspecting files,
  * interacting with other development tools.

Do not use Bash to reproduce functionality already available through a specialized LSP or MCP tool unless there is a specific reason to do so.
When a specialized tool fails, is unavailable, or does not provide sufficient information, fall back to Bash when appropriate.
