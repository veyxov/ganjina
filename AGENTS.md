# Ganjina

Self-hosted personal digital vault. MVP scope: photo/video ingest, dedup, browse.

## Project structure

Rust workspace, dependency direction strictly inward:

- `core/` — content-addressed blob store, SQLite metadata, job runner,
  watched-inbox importer, `Module` trait. Depends on nothing else in the workspace.
- `photos/` — implements `Module` for photos/videos. Depends on `core`.
- `web/` — Axum + Askama + htmx server. Depends on `core` and `photos`.

### Code style

- Never key anything by filesystem path — content hash (BLAKE3) and stable asset
  UUID only.
- New data type = new crate implementing `Module`, not new columns bolted onto
  existing tables.
- No external services (Redis, Postgres) unless actually needed at scale.
- Run `cargo fmt` and `cargo clippy` before committing.
