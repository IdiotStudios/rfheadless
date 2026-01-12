# RFox Headless Engine

[![Crates.io](https://img.shields.io/crates/v/rfheadless.svg)](https://crates.io/crates/rfheadless)
[![Documentation](https://docs.rs/rfheadless/badge.svg)](https://docs.rs/rfheadless)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A headless browsing engine API for Rust that provides a high-level interface for loading pages, running JavaScript, and producing rendered outputs (text snapshots and screenshots).

## Features

- **RFEngine** (default): A more full-featured pure-Rust engine tailored for RFox. It extracts inline & linked CSS and supports basic JS evaluation (Boa-based) exposing a minimal `document` and `console`. RFEngine is the recommended default and does not require Chrome.
- **CDP Backend** (opt-in): Uses Chrome DevTools Protocol via headless Chrome for near-complete web compatibility — enable with `--features cdp`.
- **Modular Design**: Adapter-based architecture allows swappable backend implementations
- **Safe Defaults**: Sandboxing and restrictive defaults with explicit opt-ins for risky features
- **Ergonomic API**: Simple, trait-based interface for common browsing tasks
- **Simple HTTP Engine** (opt-in): Lightweight engine using `reqwest` + `scraper` for HTML-only extraction (no JS or screenshots). **Note:** `SimpleEngine` now acts as a compatibility shim; the preferred pure-Rust backend is `RFEngine`.
- **Text & Image Rendering**: Extract text content and capture PNG screenshots
- **JavaScript Execution**: Evaluate scripts in the page context (runs by default inside a sandboxed iframe for improved isolation)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rfheadless = "0.1"
```

The CDP backend is enabled by default. To use without any backend (for custom implementations):

```toml
[dependencies]
rfheadless = { version = "0.1", default-features = false }
```

## Requirements

The CDP backend requires Chrome or Chromium to be installed on your system:

- **Linux**: Install `chromium` or `google-chrome`
- **macOS**: Install Chrome or Chromium via Homebrew
- **Windows**: Install Chrome or Chromium

## Quick Start

```rust
use rfheadless::{Engine, EngineConfig, Viewport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the engine
    let config = EngineConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/115.0 RFOX/0.3".to_string(),
        ..Default::default()
    };

    // Create and use the engine
    let mut engine = rfheadless::new_engine(config)?;
    
    // Load a page
    engine.load_url("https://example.com")?;
    
    // Get text content
    let snapshot = engine.render_text_snapshot()?;
    println!("Title: {}", snapshot.title);
    println!("Text: {}", snapshot.text);
    
    // Take a screenshot
    let png_data = engine.render_png()?;
    std::fs::write("screenshot.png", png_data)?;
    
    // Execute JavaScript
    let result = engine.evaluate_script("document.title")?;
    println!("Script result: {}", result.value);
    
    // Clean up
    engine.close()?;
    
    Ok(())
}
```

## Examples

The `examples/` directory contains several demonstration programs:

- **async_example.rs**: Async-friendly usage via `rfheadless::Browser`
- **cdp_example.rs**: Basic usage of the CDP engine
- **text_snapshot.rs**: Extracting text content from multiple pages
- **rfengine_example.rs** (opt-in `--features rfengine`): Demonstrates `RFEngine` page load, CSS extraction and JS evaluation
- **rfengine_async_example.rs** (opt-in `--features rfengine`): Async wrapper example using `tokio::spawn_blocking` to run `RFEngine` in an async task
Run an example with:

```bash
cargo run --example async_example
```

## Documentation

Additional guides and API documentation are available in the `docs/` directory:

- `docs/getting_started.md` — Quick start and running examples
- `docs/api.md` — High-level API overview
- `docs/security.md` — Security considerations and JS isolation
- `docs/examples.md` — Example descriptions and how to run them
- `docs/cookies.md` — Cookie management guide and examples

Read the docs locally by opening the files in `docs/` or by running `cargo doc` and visiting `target/doc`.

## API Overview

### Core Types

- **`EngineConfig`**: Configuration for the engine (user agent, viewport, timeout, headers, etc.)
- **`Viewport`**: Viewport dimensions (width, height)
- **`TextSnapshot`**: Extracted page content (title, text, URL)
- **`ScriptResult`**: Result of JavaScript execution

## Architecture

The crate is designed with modularity in mind:

```
rfheadless/
├── src/
│   ├── lib.rs          # Core API and types
│   ├── error.rs        # Error types
│   └── cdp.rs          # CDP adapter implementation
├── examples/           # Example programs
└── tests/              # Integration tests
```

Backend implementations are feature-gated, allowing future expansion to pure-Rust engines or other protocols.

## Roadmap

- [x] CDP backend implementation (MVP)
- [x] Text snapshot extraction
- [x] Screenshot capture
- [x] JavaScript evaluation
- [x] JS sandboxed evaluation (iframe isolation)
- [x] Event callbacks (on_load, on_console, on_request)
- [x] Cookie management API
- [ ] Pure-Rust engine adapter (long-term)
- [ ] Async API support
- [ ] Multi-tab/context support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built on top of [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome)
- Inspired by the Puppeteer and Playwright projects

---

## Chrome comparisons & golden fixtures ⚖️

If you have Chrome and the `cdp` feature available, you can run the optional Chrome comparison test with:

```bash
RUN_CHROMIUM_COMPARISONS=1 cargo test --features cdp -- --ignored
```

To allow meaningful testing without Chrome installed, we maintain a set of **golden fixtures** (in `tests/computed_style_golden.json`) that describe representative HTML/CSS inputs and expected normalized computed values (colors are canonicalized as `#rrggbb`, font sizes normalized to `px`, etc.).

When you later run the Chrome comparison test and observe differences, update the fixtures or file a bug so we can correct RFEngine's normalization logic. The CDP test is gated and ignored by default to keep CI stable for environments without Chrome.

---

## Selector operators & computed-style normalization examples 🔍

- Attribute operators: `[attr]`, `[attr=value]`, `[attr~=value]` (word), `[attr^=val]` (starts-with), `[attr$=val]` (ends-with), `[attr*=val]` (contains), `[attr|=val]` (dash-separated)
- Pseudo-classes supported: `:first-child`, `:last-child`
- Computed style normalization: colors are canonicalized (e.g., `hsl(0,100%,50%)` -> `#ff0000`), numeric lengths without units are normalized to `px`, and common unit values are standardized.
- Script execution timeouts and runtime limits: configure `script_timeout_ms`, `script_loop_iteration_limit`, and `script_recursion_limit` via `EngineConfig` to guard runaway scripts.

Example usage (RFEngine JS evaluation):

```js
// attribute operator
querySelector('[data-test~="two"]').textContent();
// pseudo-class
querySelector('#p span:first-child').textContent();
// normalized computed style
getComputedStyle(document.querySelector('#hsl')).getPropertyValue('color'); // "#ff0000"
```

These examples are covered by the golden fixtures and unit tests in `tests/` to ensure deterministic behavior without Chrome.
