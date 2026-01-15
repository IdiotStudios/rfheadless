# RFEngine specifics (feature: `rfengine`)

`RFEngine` is the pure-Rust backend included under the `rfengine` feature. Key characteristics:

- Does not require Chrome or an external browser process.
- Supports stylesheet prefetching with configurable concurrency and optional preconnect HEAD warmups.
- Supports running JavaScript via a worker harness; supports process-backed workers for stronger abort semantics.

## Script execution config

Fields in `EngineConfig` that affect script execution:

- `script_timeout_ms` — how long to wait for script evaluation before timing out (ms).
- `script_loop_iteration_limit` — maximum loop iterations for the engine's JS runtime.
- `script_recursion_limit` — maximum recursion depth.
- `use_process_worker` — when true, RFEngine spawns a subprocess to run JS; abort semantics kill the process and recreate it.

## Stylesheet fetching

- `stylesheet_fetch_concurrency` — number of concurrent stylesheet fetches.
- `enable_preconnect` — perform lightweight HEAD requests to warm connections.
- `wait_for_stylesheets_on_load` — when `true`, `load_url` waits for stylesheets to finish; set to `false` for asynchronous fetches.

## Notes and tips

- For low-latency experiments, enable the persistent runtime (`enable_persistent_runtime: true`) so async work shares a global `tokio` runtime.
- Use `evaluate_script` for isolated evaluation and `evaluate_script_in_page` (when implemented) if you need direct page-context access.
