# Engine API

This page documents the public `Engine` trait and high-level usage patterns.

## Creating an engine

Use `rfheadless::new_engine` with an `EngineConfig` to create the default implementation for your build configuration.

```rust
use rfheadless::{new_engine, EngineConfig};

let cfg = EngineConfig::default();
let mut engine = rfheadless::new_engine(cfg)?;
```

> Note: when the `rfengine` feature is enabled the crate will use the pure-Rust `RFEngine` implementation. When the `cdp` feature is enabled the CDP backend may be used (depending on build configuration).

## Core `Engine` trait overview

The `Engine` trait defines the core operations:

- `fn new(config: EngineConfig) -> Result<Self>` — create an engine instance.
- `fn load_url(&mut self, url: &str) -> Result<()>` — load and wait for page readiness.
- `fn render_text_snapshot(&self) -> Result<TextSnapshot>` — extract a text snapshot.
- `fn render_png(&self) -> Result<Vec<u8>>` — render the page as PNG bytes.
- `fn evaluate_script(&mut self, script: &str) -> Result<ScriptResult>` — evaluate JS in the page context.

There are additional helpers and lifecycle hooks:

- `on_load`, `on_console`, `on_request` — register callbacks for load events, console messages, and outgoing requests.
- Cookie helpers: `get_cookies`, `set_cookies`, `delete_cookie`, `clear_cookies` and convenience helpers like `set_cookie_simple`.
- `close(self)` — clean up resources and shut down the engine.

## Common workflow

1. Create engine with desired `EngineConfig`.
2. Call `load_url` with a URL to load the page.
3. Optionally evaluate scripts or wait for events.
4. Render snapshots or PNGs.
5. Close the engine when done.

Example:

```rust
let cfg = EngineConfig { timeout_ms: 30_000, ..Default::default() };
let mut engine = rfheadless::new_engine(cfg)?;
engine.load_url("https://example.com")?;
let snap = engine.render_text_snapshot()?;
println!("{}", snap.title);
engine.close()?;
```
