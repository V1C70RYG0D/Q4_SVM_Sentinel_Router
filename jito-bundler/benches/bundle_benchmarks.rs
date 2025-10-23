//! Benchmarks for Jito Bundle operations
//!
//! Performance targets:
//! - Bundle creation: <100μs
//! - Protection marker addition: <10μs
//! - Bundle serialization: <50μs
//!
//! Note: These benchmarks are currently disabled due to missing criterion dependency.
//! To enable, add criterion to [dev-dependencies] in Cargo.toml:
//! ```toml
//! [dev-dependencies]
//! criterion = { version = "0.5", features = ["html_reports"] }
//! ```

fn main() {
    println!("Benchmarks are currently disabled. Add criterion to dev-dependencies to enable.");
}
