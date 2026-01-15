---
title: Contributing
---

# Contributing

Thanks for wanting to contribute! A few guidelines to keep the project tidy:

- Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings` before opening PRs.
- Add unit tests for behavioral changes and use the `tests/goldens` directory for rendering fixtures. Use `UPDATE_GOLDENS=1` to update expected golden files when changing deterministic outputs.
- Docs: add or update files in `docs/` (Markdown) for user-visible changes.
- CI: consider adding a job that fails on warnings so regressions are caught early.

If you add new public API, include a short doc page in `docs/api/` describing the rationale and examples.
