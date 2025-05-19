# Contributor Guidelines

This file provides instructions for anyone contributing to this repository. Please follow these conventions when working on the codebase.

## 1. Code Style
- Format all Rust code with `cargo fmt` before committing.
- Keep functions small and well-commented where necessary.

## 2. Testing
- Always run `cargo check` to ensure the code compiles.
- Run `cargo test` before opening a pull request.

## 3. Parallelising DeepSeek Calls
This section is inspired by `agent_parallelization.rs`. When making multiple requests to DeepSeek, parallelise them using `futures::stream::iter` and `buffer_unordered`:

```rust
use futures::{stream, StreamExt};

let results = stream::iter(tasks)
    .map(|t| async move { deepseek_call(t).await })
    .buffer_unordered(8) // adjust concurrency level
    .collect::<Vec<_>>()
    .await;
```

Cache successful responses to avoid hitting the network repeatedly. Update existing sections in your documentation accordingly.
