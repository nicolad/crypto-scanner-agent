#[cfg(feature = "nautilus")]
use nautilus_trader as nt;

#[cfg(feature = "nautilus")]
fn main() {
    // Placeholder demonstrating where Nautilus Trader logic would run.
    println!("Nautilus Trader integration enabled: {}", nt::VERSION);
}

#[cfg(not(feature = "nautilus"))]
fn main() {
    println!("Nautilus Trader feature not enabled.\n\
             Rebuild with `--features nautilus` to run this example.");
}
