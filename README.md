# RFox Headless Engine

A headless browsing engine API for Rust providing a compact, testable, and deterministic environment for loading pages, running JavaScript, and extracting rendered values without always requiring Chrome.

Crates: https://crates.io/crates/rfheadless
Docs: https://idiotstudios.github.io/rfheadless/

---

## Features

- **RFEngine (default)**: pure-Rust engine using **Boa** for JS execution and `scraper` for HTML/CSS extraction.
- Deterministic console forwarding with rich metadata (source, line, column, stack).
- Minimal DOM helpers exposed to JS (`querySelector`, `dataset`, `classList`, `innerHTML`, etc.).
- Basic CSSOM parsing and `getComputedStyle` with value normalization (colors, simple units).
- Optional CDP backend (feature-gated) for Chrome/Chromium comparisons if you enable `--features cdp`.

---

## Benchmarks

Results recorded on an i7 4770K CPU with 16GB DDR3 RAM  
Test on your system with:
``` bash
cargo bench --features rfengine
```

### Results:

[latency_percentiles] samples=[7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 9, 11, 12, 12, 16, 18, 19]
[latency_percentiles] p50=8ms p95=18ms p99=19ms (threshold=200ms)
[ perf record: Woken up 129 times to write data ]
[ perf record: Captured and wrote 35.258 MB perf.data (572 samples) ]

---

## Quick start

```rust
use rfheadless::{Engine, EngineConfig};

let mut engine = rfheadless::new_engine(EngineConfig::default())?;
engine.load_url("https://example.com")?;
let snapshot = engine.render_text_snapshot()?;
println!("Title: {}", snapshot.title);
```

See `examples/` for runnable demonstrations.

---

## Desktop releases (Windows, Linux & macOS)

Each release includes:
- A packed binary (Linux: .tar.gz, Windows: .zip, macOS: .tar.gz for x86_64 and aarch64)
- A corresponding SHA256 checksum file for each artifact

Verifying downloaded artifacts:
- On Linux: `sha256sum -c rfheadless-VERSION-x86_64-unknown-linux-gnu.tar.gz.sha256`
- On macOS: `shasum -a 256 -c rfheadless-VERSION-x86_64-apple-darwin.tar.gz.sha256`
- On Windows (PowerShell): `Get-FileHash .\\rfheadless-VERSION-x86_64-pc-windows-msvc.zip -Algorithm SHA256`

Notes:
- The release workflow lives at `.github/workflows/release.yml` and is triggered on push tags `v*` and via manual dispatch.
- Pushing an annotated tag (e.g. `git tag -a v0.2.0 -m "v0.2.0" && git push origin v0.2.0`) will start the release pipeline.

---

## Contributing

If you do wanna conrtibute (apprciated) then feel free to create issues or pull requests! I love the support. We mainly focusing on getting the engine to work the best in the shortest amunt of time but other help in the engine is good to!

---

## License

Licensed under either of:

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

You may choose either license for your contribution.

---