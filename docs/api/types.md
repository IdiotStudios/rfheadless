# Key types

This page summarizes important public types returned and accepted by the API.

## `EngineConfig`

Configuration used when creating an engine. Notable fields:

- `user_agent: String` — user agent string (default includes `RFOX` tag).
- `viewport: Viewport` — `width` / `height` (default 1280×720).
- `timeout_ms: u64` — page load timeout in milliseconds (default 30000).
- `enable_javascript: bool` — global JS toggle (default `true`).
- `enable_js_isolation: bool` — isolate JS in a sandboxed context (default `true`).
- `enable_preconnect: bool` — preconnect HEAD requests for stylesheet hosts.
- `wait_for_stylesheets_on_load: bool` — whether `load_url` waits for stylesheet fetches to complete.
- `stylesheet_fetch_concurrency: usize` — concurrency limit for stylesheet fetches.

Defaults are available via `EngineConfig::default()`.

## `Viewport`

Simple `width` and `height` pair for viewport sizing.

## `TextSnapshot`

Returned by `render_text_snapshot`:

- `title: String` — page title
- `text: String` — extracted textual content
- `url: String` — final page URL after redirects

## `ScriptResult`

- `value: String` — serialized result of evaluation
- `is_error: bool` — whether the evaluation thrown an exception

## `ConsoleMessage`, `RequestInfo`, `Cookie`, etc.
