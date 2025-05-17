#!/usr/bin/env bash
cd frontend
pnpm build
cd ..
RUST_LOG=debug shuttle run --secrets backend/Secrets.toml
