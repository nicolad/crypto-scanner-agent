use crypto_scanner_agent::util::{cpu_core_count, max_parallel_threads};

fn main() {
    let cores = cpu_core_count();
    let threads = max_parallel_threads();

    println!("CPU cores detected: {}", cores);
    println!("Maximum parallel threads: {}", threads);
}
