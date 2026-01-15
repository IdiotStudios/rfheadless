---
title: Rendering primitives
---

# Rendering primitives

The crate contains a Phase 1 rendering prototype with simple primitives used by tests and goldens.

## `Screenshot`

A small container with `width`, `height`, and `png_data: Vec<u8>`.

- `Screenshot::empty(width, height)` — returns an empty PNG placeholder (used by the dummy rasterizer).

## Deterministic raster

- `rasterize_with_seed(width, height, seed)` — produces deterministic bytes derived from a SHA-256 digest of the provided `seed` bytes. Useful for golden tests where outputs need to be content-addressable.

## Layout primitives

- `Rect { x, y, width, height }` — simple rectangle
- `BoxModel { margin, border, padding }` — CSS box model
- `LayoutBox::content_width()` — returns `width` minus box model using saturating subtraction to avoid underflow.

These primitives are intentionally small and focused on testability for Phase 1 of the renderer.
