# rhyperx-rust

Rust hypergraph library. Workspace with 6 crates, edition 2024, resolver "3".

## Workspace members

| Crate | Responsibility |
|---|---|
| `rhyperx` | Public umbrella/facade crate |
| `rhyperx-core` | Core data structures, types, collections, utilities, motifs |
| `rhyperx-algo` | Graph, hypergraph, triangle, and motif algorithms |
| `rhyperx-macros` | Procedural macros used by the workspace |
| `rhyperx-io` | Dataset loading, serialization, and caching |
| `rhyperx-tests` | Integration tests and benchmarks on real datasets |

## Conventions

* **Safety and API usability:** The library should be as fool-proof as reasonably possible. Prefer APIs and implementations that make incorrect or unexpectedly expensive usage difficult for the end user. If two approaches have comparable raw performance, prefer the one that provides stronger safety guarantees and makes misuse harder.

* **Performance first:** This is a performance-critical library. Do not knowingly introduce sub-optimal algorithms, unnecessary allocations, avoidable copies, or other performance regressions. When choosing between implementations with meaningfully different performance characteristics, prefer the faster one. If you are uncertain about the algorithmic complexity or performance of an approach, **ask before proceeding** rather than guessing.

* **Performance-oriented or hacky code:** Achieving the required performance may sometimes require code that is less idiomatic, less elegant, or harder to maintain. **Do not introduce such code without asking first.** Before resorting to a hack, consider whether the same performance can be achieved cleanly using existing Rust features, declarative macros, procedural macros, or other compile-time techniques. If a macro-based solution could make performance-oriented code substantially cleaner, propose it and ask before proceeding.

* **Modularity and crate boundaries:** Keep the code modular, well-organized, and consistent with the workspace architecture. Every implementation should live in the crate that is responsible for that functionality. Avoid introducing dependencies between crates when the functionality can reasonably remain isolated.

* **Respect task scope:** When the user specifies a crate or scope, restrict investigation, modifications, and validation to that scope whenever possible. Do not inspect or modify the entire Cargo workspace by default. Inspect other crates only when necessary to understand or validate a dependency of the code being changed.

## Tool Routing Strategy

Prefer specialized tools over generic shell commands when they provide the required functionality.

* **LSP**: Use for workspace semantic code operations:
  * Local definitions, references, type hints, and compiler diagnostics.

* **rust-mcp-server**: Use for local Cargo workflows:
  * Running `check`, `test`, `clippy`, `fmt`, or `bench`.

* **rust-docs-mcp**: Use for inspecting external crates & dependencies:

* **rust-docs-mcp**: Use for API docs & source queries:
  * **Caching:** Cache new dependencies via `cache_crate` if not already indexed.
  * **Docs & API:** Search symbols (`search_items_fuzzy`), read signatures (`get_item_details`), or view module trees (`structure`).
  * **Source Inspection:** Read precise function implementations (`get_item_source`) instead of reading raw files.
  * Fallback: If `rust-docs-mcp` tools fail, lack detail, or return incomplete code for internal logic, read raw `.rs` source files directly from:
    1. `~/.rust-docs-mcp/cache/<source_type>/<crate-version>/source/src/`
    2. `~/.cargo/registry/src/index.crates.io-*/<crate-version>/src/`

* **Bash**: Use exclusively for shell-native tasks (`git`, `rg`, file manipulation) or when specialized tools fail.
