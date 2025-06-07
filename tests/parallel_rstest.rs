use futures::{stream, StreamExt};
use std::time::{Duration, Instant};
use rstest::rstest;

async fn run_tasks(delays: Vec<u64>, concurrency: usize) -> Vec<u64> {
    stream::iter(delays)
        .map(|d| async move {
            tokio::time::sleep(Duration::from_millis(d)).await;
            d
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await
}

#[rstest(concurrency, is_parallel,
    case(3, true),
    case(1, false)
)]
#[tokio::test]
async fn parallelism_behavior(concurrency: usize, is_parallel: bool) {
    let delays = vec![100, 100, 100];
    let start = Instant::now();
    let result = run_tasks(delays.clone(), concurrency).await;
    let elapsed = start.elapsed();
    assert_eq!(result.len(), delays.len());
    if is_parallel {
        assert!(elapsed < Duration::from_millis(200));
    } else {
        assert!(elapsed >= Duration::from_millis(300));
    }
}
