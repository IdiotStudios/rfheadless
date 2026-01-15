//! A lightweight, browser-less engine that fetches HTML and extracts text.
//!
//! This engine is intentionally minimal: it performs an HTTP GET, parses the
//! HTML to extract the `<title>` and the textual contents of `<body>` and
//! exposes the same `Engine` trait as other backends. JavaScript and
//! screenshot support are not provided by this engine (planned for later).

use crate::{Engine, EngineConfig, Result, ScriptResult, TextSnapshot};
#[cfg(not(feature = "rfengine"))]
use reqwest::blocking::Client;

#[cfg(not(feature = "rfengine"))]
use scraper::{Html, Selector};

#[cfg(not(feature = "rfengine"))]
use std::sync::Arc;
#[cfg(not(feature = "rfengine"))]
use std::time::Duration;

#[cfg(not(feature = "rfengine"))]
type OnLoadHandler = Arc<dyn Fn(&TextSnapshot) + Send + Sync>;
#[cfg(not(feature = "rfengine"))]
type OnConsoleHandler = Arc<dyn Fn(&crate::ConsoleMessage) + Send + Sync>;
#[cfg(not(feature = "rfengine"))]
type OnRequestHandler = Arc<dyn Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync>;

/// A simple, dependency-light engine that does not run JavaScript.
///
/// When the `rfengine` feature is enabled this type becomes a thin shim
/// delegating to `RFEngine` to avoid duplication.
pub struct SimpleEngine {
    #[cfg(feature = "rfengine")]
    inner: crate::rfengine::RFEngine,

    #[cfg(not(feature = "rfengine"))]
    client: Client,

    #[cfg(not(feature = "rfengine"))]
    config: EngineConfig,

    #[cfg(not(feature = "rfengine"))]
    last_html: Option<String>,

    #[cfg(not(feature = "rfengine"))]
    last_url: Option<String>,

    #[cfg(not(feature = "rfengine"))]
    on_load: Option<OnLoadHandler>,

    #[cfg(not(feature = "rfengine"))]
    on_console: Option<OnConsoleHandler>,

    #[cfg(not(feature = "rfengine"))]
    on_request: Option<OnRequestHandler>,
}

impl Engine for SimpleEngine {
    #[cfg(feature = "rfengine")]
    fn new(config: EngineConfig) -> Result<Self>
    where
        Self: Sized,
    {
        let inner = crate::rfengine::RFEngine::new(config)?;
        Ok(Self { inner })
    }

    #[cfg(not(feature = "rfengine"))]
    fn new(config: EngineConfig) -> Result<Self>
    where
        Self: Sized,
    {
        // Build a minimal client-based engine
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| {
                Error::InitializationError(format!("Failed to build HTTP client: {}", e))
            })?;

        Ok(Self {
            client,
            config,
            last_html: None,
            last_url: None,
            on_load: None,
            on_console: None,
            on_request: None,
        })
    }

    fn load_url(&mut self, url: &str) -> Result<()> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.load_url(url)
        }

        #[cfg(not(feature = "rfengine"))]
        {
            let res = self
                .client
                .get(url)
                .header("User-Agent", self.config.user_agent.clone())
                .send()
                .map_err(|e| Error::LoadError(format!("HTTP GET failed: {}", e)))?;

            let body = res
                .text()
                .map_err(|e| Error::LoadError(format!("Failed to read response body: {}", e)))?;

            self.last_html = Some(body);
            self.last_url = Some(url.to_string());

            // Trigger on_load callback if present
            if let Some(cb) = &self.on_load {
                if let Ok(snapshot) = self.render_text_snapshot() {
                    cb(&snapshot);
                }
            }

            Ok(())
        }
    }

    fn render_text_snapshot(&self) -> Result<TextSnapshot> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.render_text_snapshot()
        }

        #[cfg(not(feature = "rfengine"))]
        {
            let html = self
                .last_html
                .as_ref()
                .ok_or_else(|| Error::RenderError("No document loaded".into()))?;

            let document = Html::parse_document(html);
            let title_sel = Selector::parse("title").unwrap();
            let body_sel = Selector::parse("body").unwrap();

            let title = document
                .select(&title_sel)
                .next()
                .map(|n| n.text().collect::<String>())
                .unwrap_or_default();

            let text = document
                .select(&body_sel)
                .next()
                .map(|b| b.text().collect::<String>())
                .unwrap_or_default();

            Ok(TextSnapshot {
                title,
                text,
                url: self.last_url.clone().unwrap_or_default(),
            })
        }
    }

    fn render_png(&self) -> Result<Vec<u8>> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.render_png()
        }
        #[cfg(not(feature = "rfengine"))]
        {
            Err(Error::RenderError(
                "Screenshots are not supported by SimpleEngine".into(),
            ))
        }
    }

    fn evaluate_script(&mut self, script: &str) -> Result<ScriptResult> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.evaluate_script(script)
        }
        #[cfg(not(feature = "rfengine"))]
        {
            Err(Error::ScriptError(
                "JavaScript execution is not supported by SimpleEngine".into(),
            ))
        }
    }

    fn evaluate_script_in_page(&mut self, script: &str) -> Result<ScriptResult> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.evaluate_script_in_page(script)
        }
        #[cfg(not(feature = "rfengine"))]
        {
            Err(Error::ScriptError(
                "JavaScript execution is not supported by SimpleEngine".into(),
            ))
        }
    }

    fn on_load<F>(&mut self, cb: F)
    where
        F: Fn(&TextSnapshot) + Send + Sync + 'static,
    {
        #[cfg(feature = "rfengine")]
        {
            self.inner.on_load(cb);
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_load = Some(Arc::new(cb));
        }
    }

    fn clear_on_load(&mut self) {
        #[cfg(feature = "rfengine")]
        {
            self.inner.clear_on_load();
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_load = None;
        }
    }

    fn on_console<F>(&mut self, cb: F)
    where
        F: Fn(&crate::ConsoleMessage) + Send + Sync + 'static,
    {
        #[cfg(feature = "rfengine")]
        {
            self.inner.on_console(cb);
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_console = Some(Arc::new(cb));
        }
    }

    fn clear_on_console(&mut self) {
        #[cfg(feature = "rfengine")]
        {
            self.inner.clear_on_console();
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_console = None;
        }
    }

    fn on_request<F>(&mut self, cb: F)
    where
        F: Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync + 'static,
    {
        #[cfg(feature = "rfengine")]
        {
            self.inner.on_request(cb);
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_request = Some(Arc::new(cb));
        }
    }

    fn clear_on_request(&mut self) {
        #[cfg(feature = "rfengine")]
        {
            self.inner.clear_on_request();
        }
        #[cfg(not(feature = "rfengine"))]
        {
            self.on_request = None;
        }
    }

    fn get_cookies(&self) -> Result<Vec<crate::Cookie>> {
        // SimpleEngine does not manage cookies; return empty set
        Ok(vec![])
    }

    fn set_cookies(&mut self, _cookies: Vec<crate::CookieParam>) -> Result<()> {
        // No-op for now
        Ok(())
    }

    fn delete_cookie(
        &mut self,
        _name: &str,
        _url: Option<&str>,
        _domain: Option<&str>,
        _path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn clear_cookies(&mut self) -> Result<()> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.clear_cookies()
        }
        #[cfg(not(feature = "rfengine"))]
        {
            Ok(())
        }
    }

    fn close(self) -> Result<()> {
        #[cfg(feature = "rfengine")]
        {
            self.inner.close()
        }
        #[cfg(not(feature = "rfengine"))]
        {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_engine_parses_html() {
        // Skip on CI where network may not be available
        if std::env::var("CI").is_ok() {
            return;
        }

        // Start a tiny_http server to serve a simple HTML document
        let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
        let addr = server.server_addr();

        std::thread::spawn(move || {
            if let Ok(request) = server.recv() {
                let response = tiny_http::Response::from_string(
                    "<html><head><title>Hi</title></head><body>Hello world</body></html>",
                );
                let _ = request.respond(response);
            }
        });

        let url = format!("http://{}", addr);
        let mut engine = SimpleEngine::new(crate::EngineConfig::default())
            .expect("Failed to create SimpleEngine");
        engine.load_url(&url).expect("Failed to load URL");
        let snapshot = engine
            .render_text_snapshot()
            .expect("Failed to render snapshot");
        assert!(snapshot.title.contains("Hi"));
        assert!(snapshot.text.contains("Hello"));
    }
}
