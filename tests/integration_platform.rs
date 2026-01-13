use rfheadless::platform::service_worker::NoopServiceWorkerManager;
use rfheadless::platform::device::NoopDeviceEmulation;
use rfheadless::platform::media::NoopMediaHooks;
use rfheadless::platform::accessibility::NoopAccessibility;
use rfheadless::platform::{ServiceWorkerManager, DeviceEmulation, MediaHooks, AccessibilityProvider};

#[test]
fn service_worker_noop_dispatch() {
    let m = NoopServiceWorkerManager::new();
    let ev = rfheadless::platform::FetchEvent { request_url: "https://example.com/".into(), method: "GET".into(), headers: Default::default() };
    let res = m.dispatch_fetch(&ev).expect("dispatch should succeed");
    assert_eq!(res, b"noop".to_vec());
}

#[test]
fn device_emulation_metrics_roundtrip() {
    let d = NoopDeviceEmulation::new();
    let before = d.metrics();
    assert_eq!(before.width, 1280);
    d.set_metrics(rfheadless::platform::DeviceMetrics { width: 360, height: 640, dpr: 3.0, touch: true });
    let after = d.metrics();
    assert_eq!(after.width, 360);
    assert!(after.touch);
}

#[test]
fn media_hooks_state_transitions() {
    let m = NoopMediaHooks::new();
    assert_eq!(m.state(), rfheadless::platform::MediaState::Paused);
    m.play();
    assert_eq!(m.state(), rfheadless::platform::MediaState::Playing);
    m.pause();
    assert_eq!(m.state(), rfheadless::platform::MediaState::Paused);
}

#[test]
fn accessibility_noop_exports_empty() {
    let a = NoopAccessibility::new();
    let t = a.export_tree();
    assert!(t.nodes.is_empty());
    assert!(t.root_id.is_none());
}
