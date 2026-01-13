//! RFox Headless Engine
//!
//! A headless browsing engine API for Rust that provides a high-level interface
//! for loading pages, running JavaScript, and producing rendered outputs.
//!
//! # Features
//!
//! - **CDP Backend** (default): Uses Chrome DevTools Protocol via headless Chrome
//! - **Modular Design**: Adapter-based architecture for swappable backends
//! - **Safe Defaults**: Sandboxing and restrictive defaults with explicit opt-ins
//!
//! # Example
//!
//! ```no_run
//! use rfheadless::{Engine, EngineConfig, Viewport};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = EngineConfig {
//!     user_agent: "RFox/1.0".to_string(),
//!     viewport: Viewport { width: 1280, height: 720 },
//!     timeout_ms: 30000,
//!     ..Default::default()
//! };
//!
//! let mut engine = rfheadless::new_engine(config)?;
//! engine.load_url("https://example.com")?;
//! let snapshot = engine.render_text_snapshot()?;
//! println!("Title: {}", snapshot.title);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

pub mod error;
pub use error::{Error, Result};

#[cfg(feature = "cdp")]
pub mod cdp;

// Simple HTTP-based engine (no JS, no screenshots)
#[cfg(feature = "simple")]
pub mod simple;

// RFEngine: RFox custom pure-rust backend (CSS extraction + JS hook)
#[cfg(feature = "rfengine")]
pub mod rfengine;

// Rendering prototype (Phase 1) — feature-gated under `rfengine` for now
#[cfg(feature = "rfengine")]
pub mod rendering;

// Platform API surface (service workers, media hooks, accessibility, device emulation)
pub mod platform;

// Async-friendly browser API (simple worker-backed abstraction)
#[cfg(feature = "cdp")]
pub mod async_api;

// Re-export the Browser type at the crate root for ergonomic examples
#[cfg(feature = "cdp")]
pub use async_api::Browser;
/// Configuration for the headless engine
///
/// This struct contains the core engine configuration used when creating an
/// `Engine` instance. The defaults are chosen to be conservative and safe:
/// - `user_agent` is set to a Firefox-compatible string that identifies RFOX
/// - JavaScript is enabled by default but runs in a sandboxed iframe (see
///   `enable_js_isolation`)
///
/// # Examples
///
/// ```
/// let cfg = rfheadless::EngineConfig::default();
/// assert!(cfg.user_agent.contains("RFOX"));
/// ```
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// User agent string to send with requests
    pub user_agent: String,
    /// Viewport dimensions
    pub viewport: Viewport,
    /// Timeout for page loads in milliseconds
    pub timeout_ms: u64,
    /// Custom HTTP headers
    pub headers: HashMap<String, String>,
    /// Whether to enable JavaScript execution
    pub enable_javascript: bool,
    /// Whether to run user JS inside an isolated context (sandboxed iframe)
    pub enable_js_isolation: bool,
    /// Whether to enable images
    pub enable_images: bool,
    /// Whether to spawn process-backed workers for stronger abort semantics
    pub use_process_worker: bool,
    /// Script execution timeout in milliseconds (applies to `evaluate_script`)
    pub script_timeout_ms: u64,
    /// Maximum loop iterations before Boa throws an error (0 => disabled)
    pub script_loop_iteration_limit: u64,
    /// Maximum recursion depth before Boa throws (usize::MAX => disabled)
    pub script_recursion_limit: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/115.0 RFOX/0.3".to_string(),
            viewport: Viewport::default(),
            timeout_ms: 30000,
            headers: HashMap::new(),
            enable_javascript: true,
            enable_js_isolation: true,
            enable_images: true,
            use_process_worker: false,
            script_timeout_ms: 5000,
            script_loop_iteration_limit: 1000000,
            script_recursion_limit: 1024,
        }
    }
}

/// Viewport dimensions
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

/// A textual snapshot of a rendered page
///
/// This type is returned by `Engine::render_text_snapshot` and contains a
/// simple representation of the page content suitable for textual tests and
/// quick inspection.
#[derive(Debug, Clone)]
pub struct TextSnapshot {
    /// Page title
    pub title: String,
    /// Extracted text content
    pub text: String,
    /// Final URL after redirects
    pub url: String,
}

/// Result of JavaScript execution
///
/// `value` is the serialized result of the evaluation (usually a JSON-like
/// string). `is_error` indicates whether the script threw an exception.
#[derive(Debug, Clone)]
pub struct ScriptResult {
    /// Serialized result value
    pub value: String,
    /// Whether the script threw an error
    pub is_error: bool,
}

/// Console message emitted by the page
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    /// Level such as "log", "warn", or "error"
    pub level: String,
    /// Textual content of the message
    pub text: String,
    /// Optional source/filename if available (parsed from stack)
    pub source: Option<String>,
    /// Optional line number if available
    pub line: Option<u32>,
    /// Optional column number if available
    pub column: Option<u32>,
    /// Optional raw JS stack trace if provided by the engine
    pub stack: Option<String>,
}

/// Information about an outgoing network request
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// Unique request identifier (backend-specific)
    pub request_id: String,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Optional resource type (script, xhr, etc.)
    pub resource_type: Option<String>,
    /// Headers
    pub headers: std::collections::HashMap<String, String>,
}

/// A cookie retrieved from the browser
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<u64>,
    pub size: Option<u32>,
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
}

/// Parameters for setting a cookie
#[derive(Debug, Clone)]
pub struct CookieParam {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>,
    pub expires: Option<u64>,
}

/// Action to take when a request is observed by `on_request` handlers.
#[derive(Debug, Clone)]
pub enum RequestAction {
    /// Let the request proceed normally
    Continue,

    /// Fail the request with an error reason
    Fail { error_reason: String },

    /// Fulfill the request with a custom response
    Fulfill {
        /// HTTP status code
        status: u16,
        /// Response headers
        headers: std::collections::HashMap<String, String>,
        /// Response body bytes
        body: Vec<u8>,
    },
}


/// Core trait for headless engine implementations
pub trait Engine {
    /// Create a new engine instance with the given configuration
    fn new(config: EngineConfig) -> Result<Self>
    where
        Self: Sized;

    /// Load a URL and wait for the page to be ready
    fn load_url(&mut self, url: &str) -> Result<()>;

    /// Render the current page as a text snapshot
    fn render_text_snapshot(&self) -> Result<TextSnapshot>;

    /// Render the current page as a PNG image
    fn render_png(&self) -> Result<Vec<u8>>;

    /// Evaluate JavaScript in the page context
    fn evaluate_script(&mut self, script: &str) -> Result<ScriptResult>;

    /// Evaluate JavaScript in the page context (non-isolated). Default implementation
    /// falls back to `evaluate_script`, but backend implementations may override this
    /// to provide a direct page evaluation that accesses the page's DOM.
    fn evaluate_script_in_page(&mut self, script: &str) -> Result<ScriptResult> {
        // By default, call evaluate_script which may use isolation depending on config.
        self.evaluate_script(script)
    }

    /// Register a callback to be invoked when a page finishes loading.
    /// The callback receives a `TextSnapshot` describing the loaded page.
    fn on_load<F>(&mut self, cb: F)
    where
        F: Fn(&TextSnapshot) + Send + Sync + 'static;

    /// Remove previously registered on_load callback if any
    fn clear_on_load(&mut self);

    /// Register a callback for console messages emitted by the page.
    fn on_console<F>(&mut self, cb: F)
    where
        F: Fn(&ConsoleMessage) + Send + Sync + 'static;

    /// Remove previously registered on_console callback if any
    fn clear_on_console(&mut self);

    /// Register a callback for outgoing network requests (fires before the
    /// request is sent). The handler can observe metadata and return a
    /// `RequestAction` to control how the request should be handled (continue,
    /// fail, or be fulfilled with a custom response).
    fn on_request<F>(&mut self, cb: F)
    where
        F: Fn(&RequestInfo) -> RequestAction + Send + Sync + 'static;

    /// Remove previously registered on_request callback if any
    fn clear_on_request(&mut self);

    /// Get cookies relevant to the current page (returns cookie list)
    fn get_cookies(&self) -> Result<Vec<Cookie>>;

    /// Set cookies on the current page
    fn set_cookies(&mut self, cookies: Vec<CookieParam>) -> Result<()>;

    /// Delete a cookie (by name, optionally with url/domain/path to disambiguate)
    fn delete_cookie(&mut self, name: &str, url: Option<&str>, domain: Option<&str>, path: Option<&str>) -> Result<()>;

    /// Clear all cookies for the browser context
    fn clear_cookies(&mut self) -> Result<()>;

    // --- Higher-level convenience helpers (default implementations) ---

    /// Set a single cookie with common parameters
    fn set_cookie_simple(&mut self, name: &str, value: &str, url: Option<&str>, domain: Option<&str>, path: Option<&str>, expires: Option<u64>) -> Result<()> {
        let param = CookieParam {
            name: name.to_string(),
            value: value.to_string(),
            url: url.map(|s| s.to_string()),
            domain: domain.map(|s| s.to_string()),
            path: path.map(|s| s.to_string()),
            secure: None,
            http_only: None,
            same_site: None,
            expires,
        };
        self.set_cookies(vec![param])
    }

    /// Get a named cookie if present for the current page
    fn get_cookie_simple(&self, name: &str) -> Result<Option<Cookie>> {
        let cookies = self.get_cookies()?;
        Ok(cookies.into_iter().find(|c| c.name == name))
    }

    /// Clear cookies for a given domain
    fn clear_cookies_for_domain(&mut self, domain: &str) -> Result<()> {
        let cookies = self.get_cookies()?;
        for c in cookies.into_iter().filter(|c| c.domain.as_deref() == Some(domain)) {
            let _ = self.delete_cookie(&c.name, None, Some(domain), c.path.as_deref());
        }
        Ok(())
    }

    /// Close the engine and clean up resources
    fn close(self) -> Result<()>;
}

/// Create a new engine instance with the default backend
///
/// This prefers the CDP backend when the `cdp` feature is enabled (default).
/// If `cdp` is not enabled but the `simple` feature is enabled, the
/// `SimpleEngine` will be used instead.
// Prefer RFEngine when the feature is enabled (it does not require Chrome).
#[cfg(feature = "rfengine")]
pub fn new_engine(config: EngineConfig) -> Result<impl Engine> {
    rfengine::RFEngine::new(config)
}

// Fallback to CDP when RFEngine is not enabled but CDP is.
#[cfg(all(not(feature = "rfengine"), feature = "cdp"))]
pub fn new_engine(config: EngineConfig) -> Result<impl Engine> {
    cdp::CdpEngine::new(config)
}

// Last-resort: the simple engine if no other backend is available.
#[cfg(all(not(feature = "rfengine"), not(feature = "cdp"), feature = "simple"))]
pub fn new_engine(config: EngineConfig) -> Result<impl Engine> {
    simple::SimpleEngine::new(config)
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.viewport.width, 1280);
        assert_eq!(config.viewport.height, 720);
        assert!(config.enable_javascript);
    }

    #[test]
    fn test_viewport() {
        let viewport = Viewport {
            width: 1920,
            height: 1080,
        };
        assert_eq!(viewport.width, 1920);
        assert_eq!(viewport.height, 1080);
    }
}
