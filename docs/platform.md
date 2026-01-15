---
title: Platform API
---

# Platform API (service workers, media, accessibility, device)

This repository exposes simple trait-based primitives for deterministic tests and backends.

## `PlatformApi`

Backends can implement `PlatformApi` to provide platform surfaces:

- `service_worker_manager() -> Box<dyn ServiceWorkerManager>`
- `media_hooks() -> Box<dyn MediaHooks>`
- `accessibility_provider() -> Box<dyn AccessibilityProvider>`
- `device_emulation() -> Box<dyn DeviceEmulation>`

A `NoopPlatform` implementation returns noop providers suitable for unit tests.

## Service Worker shim

`ServiceWorkerManager` provides:

- `register`, `unregister`, `list_registrations`
- `dispatch_fetch` — for test-only synthetic fetch responses

`NoopServiceWorkerManager` returns `Err` on register and a `noop` response body on fetch.

## Media hooks

`MediaHooks` supports `play`, `pause`, `seek`, and `state()` returning `MediaState`.
`NoopMediaHooks` stores state in-memory for tests.

## Accessibility

`AccessibilityProvider::export_tree` returns a reproducible `AccessibilityTree` for golden tests.
`NoopAccessibility` returns an empty tree.

## Device emulation

`DeviceEmulation` supports getting and setting `DeviceMetrics` (width/height/dpr/touch).
`NoopDeviceEmulation` stores metrics in a mutex for deterministic tests.
