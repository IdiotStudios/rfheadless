# Development

- Run tests: `cargo test`
- Run benches: `cargo bench --features rfengine`
- Generate a flamegraph for a bench (requires `cargo-flamegraph`):

  cargo flamegraph --bench latency_runner --features rfengine

- Build docs: `cargo doc --no-deps --open`

- Formatting and linting: `cargo fmt` and `cargo clippy`

- If you see warnings: run `cargo build` and address any `unused` or other warnings.
