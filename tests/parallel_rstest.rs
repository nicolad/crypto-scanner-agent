use futures::{stream, StreamExt};
use std::time::{Duration, Instant};

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

#[tokio::test]
async fn parallelism_behavior_parallel() {
    let delays = vec![100, 100, 100];
    let start = Instant::now();
    let result = run_tasks(delays.clone(), 3).await;
    let elapsed = start.elapsed();
    assert_eq!(result.len(), delays.len());
    assert!(elapsed < Duration::from_millis(200));
}

#[tokio::test]
async fn parallelism_behavior_sequential() {
    let delays = vec![100, 100, 100];
    let start = Instant::now();
    let result = run_tasks(delays.clone(), 1).await;
    let elapsed = start.elapsed();
    assert_eq!(result.len(), delays.len());
    assert!(elapsed >= Duration::from_millis(300));
}
