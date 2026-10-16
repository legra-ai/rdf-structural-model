# Agent instructions

- Read the README and preserve the RDF 1.2 position and identity contracts.
- Use Rust 1.99.1 with the separately pinned nightly formatter (`scripts/fmt.sh`).
- Test fixes first. Use `cargo test`; never use `cargo check`.
- Shared semantic types support Serde in JSON and non-self-describing formats.
- Preserve exact lexical forms, scoped blank-node identity and typed extensions.
- No JSON-specific model types, untagged Serde enums or whole-graph containers.
- No filesystem I/O or global mutable state in this model crate.
- Commit focused validated changes promptly; never bypass hooks.
- Use scoped Conventional Commits. Keep Cargo.lock tracked.
- Release only through the shared main CI pipeline; never push directly to main.
