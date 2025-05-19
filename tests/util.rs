use crypto_scanner_agent::util::{cpu_core_count, max_parallel_threads};

#[test]
fn core_count_nonzero() {
    assert!(cpu_core_count() >= 1);
}

#[test]
fn parallel_threads_nonzero() {
    assert!(max_parallel_threads() >= 1);
}
