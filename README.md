# RFox Headless Engine

A headless browsing engine API for Rust providing a compact, testable, and deterministic environment for loading pages, running JavaScript, and extracting rendered values without always requiring Chrome.

## Features

- **RFEngine (default)**: pure-Rust engine using **Boa** for JS execution and `scraper` for HTML/CSS extraction.
- Deterministic console forwarding with rich metadata (source, line, column, stack).
- Minimal DOM helpers exposed to JS (`querySelector`, `dataset`, `classList`, `innerHTML`, etc.).
- Basic CSSOM parsing and `getComputedStyle` with value normalization (colors, simple units).
- Optional CDP backend (feature-gated) for Chrome/Chromium comparisons if you enable `--features cdp`.

---

## Quick start (FYI I have not tested this yet)

```rust
use rfheadless::{Engine, EngineConfig};

let mut engine = rfheadless::new_engine(EngineConfig::default())?;
engine.load_url("https://example.com")?;
let snapshot = engine.render_text_snapshot()?;
println!("Title: {}", snapshot.title);
```

See `examples/` for runnable demonstrations.

---

## Roadmap & Milestones

High-level goals (subject to change):

- **M1 — JS runtime safety & scheduling**
  - Script timeouts, runtime limits (done: timeouts + loop/recursion limits)
  - Microtask/job queue & timer APIs (progress: microtask queue, timers (`setTimeout`/`setInterval`, cancellation), and helpers added; basic Promise polyfill when native Promise is not available)
  - Context reuse & isolation improvements (TODO)

- **M2 — CSSOM & computed-value parity** 
  - Full property normalization (em/rem/percent conversions, hsl/hsla/hwb)
  - Pseudo-elements, advanced selectors, cascade edge-cases
  - Expand golden fixtures and cross-engine comparisons

- **M3 — Layout & rendering prototype**
  - Simplified layout engine (flow & box model)
  - Layout computations (reflow, sizes, line wrapping)
  - Paint pipeline and screenshot API (basic rasterization)

- **M4 — Network & tooling**
  - Request interception, fulfill, and mock responses
  - Network emulation (latency/bandwidth/offline)
  - Optional Chrome comparison CI job (gated)

- **M5 — Polishing & ops**
  - Performance optimizations, benchmarks, docs, security/fuzzing, release automation

---

## Scope & Boundaries

**(current focus — prioritized):**
- **Rendering (priority: high)** — Achieve full pixel-accurate rendering parity with Chromium. Phase 1 focuses on a simplified layout & paint pipeline (flow layout, box model, reflow, line wrapping, and basic rasterization). Deliverables: deterministic screenshot API, image golden fixtures, and cross-engine comparison tooling.
- **Platform APIs (priority: high)** — Implement the Web platform surface required for parity tests: service workers, media playback hooks, accessibility tree basics, and device emulation. Each surface will be delivered incrementally and gated by deterministic tests and golden fixtures.
- **CDP surface (priority: medium)** — Provide an optional, feature-gated CDP server translation façade for external tooling and Chrome-compatibility checks. Aim for a minimal surface sufficient for comparisons and CI integration, expandable over time.

**(long-term):**
- Expand and stabilize the rendering and platform stacks, add performance optimizations, implement advanced layout features (flex/grid, table layout), and harden via fuzzing and benchmarks.

Decisions favor a pragmatic, test-first approach: prioritize an incremental path to platform parity by implementing the rendering stack and platform APIs as driven by deterministic tests and parity goals.

---

## License

Licensed under either of:

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

You may choose either license for your contribution.

---