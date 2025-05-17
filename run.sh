#!/usr/bin/env bash
clear
RUST_LOG=debug shuttle run --secrets Secrets.toml
