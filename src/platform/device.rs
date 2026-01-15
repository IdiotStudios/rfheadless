/// Device emulation primitives for deterministic tests

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceMetrics {
    pub width: u32,
    pub height: u32,
    pub dpr: f32,
    pub touch: bool,
}

pub trait DeviceEmulation: Send + Sync {
    fn set_metrics(&self, m: DeviceMetrics);
    fn metrics(&self) -> DeviceMetrics;
}

/// Noop implementation that stores metrics in a Mutex
pub struct NoopDeviceEmulation {
    metrics: std::sync::Mutex<DeviceMetrics>,
}

impl NoopDeviceEmulation {
    pub fn new() -> Self {
        NoopDeviceEmulation {
            metrics: std::sync::Mutex::new(DeviceMetrics {
                width: 1280,
                height: 720,
                dpr: 1.0,
                touch: false,
            }),
        }
    }
}

impl Default for NoopDeviceEmulation {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceEmulation for NoopDeviceEmulation {
    fn set_metrics(&self, m: DeviceMetrics) {
        let mut g = self.metrics.lock().unwrap();
        *g = m;
    }

    fn metrics(&self) -> DeviceMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_device_metrics_can_be_updated() {
        let d = NoopDeviceEmulation::new();
        assert_eq!(d.metrics().width, 1280);
        d.set_metrics(DeviceMetrics {
            width: 800,
            height: 600,
            dpr: 2.0,
            touch: true,
        });
        let m = d.metrics();
        assert_eq!(m.width, 800);
        assert_eq!(m.dpr, 2.0);
        assert!(m.touch);
    }
}
