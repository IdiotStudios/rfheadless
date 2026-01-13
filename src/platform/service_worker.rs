use std::collections::HashMap;

/// Minimal service worker registration metadata used for tests
#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    pub scope: String,
    pub script_url: String,
    pub id: String,
}

/// A fetch event that the engine can dispatch to registered workers
#[derive(Debug, Clone)]
pub struct FetchEvent {
    pub request_url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
}

/// Manager trait for service worker lifecycle and fetch interception
pub trait ServiceWorkerManager: Send + Sync {
    /// Register a worker for a given scope
    fn register(&self, scope: &str, script_url: &str) -> Result<ServiceWorkerRegistration, String>;

    /// Unregister a worker
    fn unregister(&self, scope: &str) -> Result<(), String>;

    /// List current registrations
    fn list_registrations(&self) -> Vec<ServiceWorkerRegistration>;

    /// Dispatch a fetch event to the worker(s) and return a response bytes
    /// (for prototype tests this may be a simple synthetic response)
    fn dispatch_fetch(&self, event: &FetchEvent) -> Result<Vec<u8>, String>;
}

/// A noop manager that doesn't register workers and returns simple responses
pub struct NoopServiceWorkerManager;

impl NoopServiceWorkerManager {
    pub fn new() -> Self {
        NoopServiceWorkerManager
    }
}

impl ServiceWorkerManager for NoopServiceWorkerManager {
    fn register(&self, _scope: &str, _script_url: &str) -> Result<ServiceWorkerRegistration, String> {
        Err("service workers not supported".to_string())
    }

    fn unregister(&self, _scope: &str) -> Result<(), String> {
        Ok(())
    }

    fn list_registrations(&self) -> Vec<ServiceWorkerRegistration> {
        Vec::new()
    }

    fn dispatch_fetch(&self, _event: &FetchEvent) -> Result<Vec<u8>, String> {
        Ok(b"noop".to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_dispatch_fetch_returns_noop_body() {
        let m = NoopServiceWorkerManager::new();
        let ev = FetchEvent {
            request_url: "https://example.com/".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
        };
        let res = m.dispatch_fetch(&ev).unwrap();
        assert_eq!(res, b"noop".to_vec());
    }
}
