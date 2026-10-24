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

## Zero-user pre-production versions

This repository has zero users. Fixes use patch releases; features and breaking
changes use minor releases. Preserve accurate breaking-change markers and use
`breaking-change-bump: minor` for both planning and preparation. A major increase
still requires the user's specific explicit approval. Do not add migrations or
compatibility paths for hypothetical consumers.

Every major version increase requires explicit user approval of that version and
release scope. General implementation, merge, or release approval does not suffice.
Never approve a major-release environment with owner credentials without that
specific authorization.
