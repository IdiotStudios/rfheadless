use rfheadless::platform::NoopPlatform;
use rfheadless::platform::{DeviceMetrics, FetchEvent, PlatformApi};

#[test]
fn platform_noop_smoke() {
    let p = NoopPlatform::new();

    // device
    let d = p.device_emulation();
    assert_eq!(d.metrics().width, 1280);
    d.set_metrics(DeviceMetrics {
        width: 360,
        height: 640,
        dpr: 3.0,
        touch: true,
    });
    assert_eq!(d.metrics().width, 360);

    // service worker fetch
    let sw = p.service_worker_manager();
    let ev = FetchEvent {
        request_url: "https://example/".into(),
        method: "GET".into(),
        headers: Default::default(),
    };
    let res = sw.dispatch_fetch(&ev).unwrap();
    assert_eq!(res, b"noop".to_vec());
}
