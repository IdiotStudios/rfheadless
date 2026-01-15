//! Platform API surface: service workers, media hooks, accessibility, device emulation
//!
//! This module contains the public types and traits used by engine backends to
//! expose deterministic, testable platform primitives needed for parity tests.

pub mod accessibility;
pub mod device;
pub mod media;
pub mod service_worker;

pub use accessibility::{AccessibilityNode, AccessibilityProvider, AccessibilityTree};
pub use device::{DeviceEmulation, DeviceMetrics};
pub use media::{MediaHooks, MediaState};
pub use service_worker::{FetchEvent, ServiceWorkerManager, ServiceWorkerRegistration};

/// A small composite trait that engine implementations can offer to allow
/// consumers to access platform primitives in a typed way.
///
/// Engines that don't yet provide certain surfaces may implement a noop
/// provider that returns reasonable defaults for tests.
pub trait PlatformApi: Send + Sync {
    fn service_worker_manager(&self) -> Box<dyn ServiceWorkerManager>;
    fn media_hooks(&self) -> Box<dyn MediaHooks>;
    fn accessibility_provider(&self) -> Box<dyn AccessibilityProvider>;
    fn device_emulation(&self) -> Box<dyn DeviceEmulation>;
}

/// A small noop Platform implementation used in unit tests and as a safe
/// default for backends that haven't implemented the full surface yet.
pub struct NoopPlatform;

impl NoopPlatform {
    pub fn new() -> Self {
        NoopPlatform
    }
}

impl Default for NoopPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformApi for NoopPlatform {
    fn service_worker_manager(&self) -> Box<dyn ServiceWorkerManager> {
        Box::new(service_worker::NoopServiceWorkerManager::new())
    }

    fn media_hooks(&self) -> Box<dyn MediaHooks> {
        Box::new(media::NoopMediaHooks::new())
    }

    fn accessibility_provider(&self) -> Box<dyn AccessibilityProvider> {
        Box::new(accessibility::NoopAccessibility::new())
    }

    fn device_emulation(&self) -> Box<dyn DeviceEmulation> {
        Box::new(device::NoopDeviceEmulation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_platform_provides_noop_surfaces() {
        let p = NoopPlatform::new();
        let sw = p.service_worker_manager();
        assert!(sw.list_registrations().is_empty());

        let media = p.media_hooks();
        assert_eq!(media.state(), MediaState::Paused);

        let acc = p.accessibility_provider();
        let tree = acc.export_tree();
        assert!(tree.nodes.is_empty());

        let dev = p.device_emulation();
        let m = dev.metrics();
        assert_eq!(m.width, 1280);
    }
}
