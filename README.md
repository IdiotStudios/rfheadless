# RFox Headless Engine

A headless browsing engine API for Rust providing a compact, testable, and deterministic environment for loading pages, running JavaScript, and extracting rendered values without always requiring Chrome.

## Features

- **RFEngine (default)**: pure-Rust engine using **Boa** for JS execution and `scraper` for HTML/CSS extraction.
- Deterministic console forwarding with rich metadata (source, line, column, stack).
- Minimal DOM helpers exposed to JS (`querySelector`, `dataset`, `classList`, `innerHTML`, etc.).
- Basic CSSOM parsing and `getComputedStyle` with value normalization (colors, simple units).
- Optional CDP backend (feature-gated) for Chrome/Chromium comparisons if you enable `--features cdp`.

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

## Roadmap & Milestones

High-level goals and rough estimates (subject to change):

- **M1 — JS runtime safety & scheduling** (≈ 100–140h)
  - Script timeouts, runtime limits (done: timeouts + loop/recursion limits)
  - Microtask/job queue & timer APIs (progress: microtask queue, timers (`setTimeout`/`setInterval`, cancellation), and helpers added; basic Promise polyfill when native Promise is not available)
  - Context reuse & isolation improvements (TODO — next work item)

- **M2 — CSSOM & computed-value parity** (≈ 100–140h)
  - Full property normalization (em/rem/percent conversions, hsl/hsla/hwb)
  - Pseudo-elements, advanced selectors, cascade edge-cases
  - Expand golden fixtures and cross-engine comparisons

- **M3 — Layout & rendering prototype** (≈ 200–260h)
  - Simplified layout engine (flow & box model)
  - Layout computations (reflow, sizes, line wrapping)
  - Paint pipeline and screenshot API (basic rasterization)

- **M4 — Network & tooling** (≈ 60–80h)
  - Request interception, fulfill, and mock responses
  - Network emulation (latency/bandwidth/offline)
  - Optional Chrome comparison CI job (gated)

- **M5 — Polishing & ops** (≈ 40–60h)
  - Performance optimizations, benchmarks, docs, security/fuzzing, release automation

*Total estimate: 600+ hours (including buffer/contingency). If we exceed this, that’s fine — we can prioritize accordingly.*

---

## Scope & Boundaries

**(current focus):**
- Deterministic JS eval with a small, safe DOM shim and console forwarding
- CSS parsing for computed values needed by tests (normalized colors, basic units)
- Test-driven development with golden fixtures to allow Chrome-free verification
- Optional, gated Chrome/Chromium comparisons when available

**(long-term):**
- Full, pixel-accurate rendering equivalent to Chromium (requires a layout/paint stack)
- Complete Web platform APIs (service workers, media, accessibility tree, full device emulation)
- Serving a CDP server surface for external tooling (we may optionally add a translation façade later)

Decisions favor a pragmatic, test-first approach: implement what we need for deterministic tests and expand toward parity iteratively.

---

## Contributing & Local workflow

- Work is done on a local `wip/rfengine` branch until we reach the agreed milestone (e.g., 100h), then we will discuss pushing upstream.
- Use `WORKING.md` for a local log of time & notes (not pushed remotely by default).
- Optional test utility scripts live in `scripts/` (e.g., `scripts/log_session.sh` for daily session logging).

---

## License

Licensed under either of:

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

You may choose either license for your contribution.

---

If you want a different structure or more details in the roadmap (breakdowns, Gantt-style milestones, or CI gating rules), tell me how verbose you want it and I’ll expand it into `docs/roadmap.md`.
