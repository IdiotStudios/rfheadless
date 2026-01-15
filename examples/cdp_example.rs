//! CDP example (deprecated in headless-only plan)
//!
//! The project is focused on the headless engine and the CDP-related examples
//! are intentionally kept as a feature-gated placeholder. Enable `--features cdp`
//! and adapt this example if/when you want to run Chrome comparisons.

#[cfg(feature = "cdp")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CDP example placeholder - enable `cdp` feature and adapt for your environment.");
    Ok(())
}

#[cfg(not(feature = "cdp"))]
fn main() {
    eprintln!(
        "This example requires the 'cdp' feature: cargo run --example cdp_example --features cdp"
    );
}
