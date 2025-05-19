pub mod util {
    /// Returns the number of logical CPU cores available on the system.
    pub fn cpu_core_count() -> usize {
        num_cpus::get()
    }

    /// Returns the maximum number of threads that can run in parallel.
    pub fn max_parallel_threads() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}
