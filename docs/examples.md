# Examples & benches

This repo contains small examples and benches to demonstrate usage and measure performance.

## Examples

- `examples/simple_headless.rs` — Minimal example that starts an engine, loads a tiny local server page, and prints a snapshot. Run with:

  cargo run --example simple_headless --features rfengine

- `examples/bench_latency.rs` — Quick p50/p95/p99 latency runner for local checks.

## Benchmarks

- `benches/latency_runner.rs` — A non-harness bench target that prints percentiles directly; configured in `Cargo.toml`.

Run all benches (RFEngine feature):

  cargo bench --features rfengine

To generate a flamegraph for the latency bench:

  cargo flamegraph --bench latency_runner --features rfengine

Notes:
- Bench code uses tiny local servers to provide deterministic HTML and CSS responses and measures elapsed times across `load_url` calls.
