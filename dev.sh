#!/usr/bin/env bash
# Runs backend + frontend together with live reload:
#   - cargo-watch rebuilds and restarts the Rust server on any core/photos/web change
#   - vite dev server gives instant frontend HMR, proxying /api and /blobs to the backend
#
# Open http://localhost:5173 — that's the frontend dev server, not :3000 directly.
set -euo pipefail
cd "$(dirname "$0")"

trap 'kill 0' EXIT

cargo watch -w core -w photos -w web -x 'run -p web' &
(cd frontend && npm run dev) &

wait
