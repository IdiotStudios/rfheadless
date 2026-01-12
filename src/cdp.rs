//! Chrome DevTools Protocol adapter implementation

use crate::{Engine, EngineConfig, Error, Result, ScriptResult, TextSnapshot};
use headless_chrome::browser::tab::Tab;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::browser::tab::{RequestPausedDecision, RequestInterceptor};
use headless_chrome::protocol::cdp::Fetch::events::RequestPausedEvent;
use headless_chrome::protocol::cdp::Fetch::{FulfillRequest, HeaderEntry};
use log::warn;

// Type aliases to simplify complex handler types
type OnLoadHandler = std::sync::Arc<dyn Fn(&crate::TextSnapshot) + Send + Sync>;
type OnConsoleHandler = std::sync::Arc<dyn Fn(&crate::ConsoleMessage) + Send + Sync>;
type OnRequestHandler = std::sync::Arc<dyn Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync>;
use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;
use std::time::Duration;
use base64::Engine as Base64Engine;

/// CDP-based headless engine implementation (uses the `headless_chrome` crate)
///
/// This adapter launches a headless Chrome instance, manages a single tab,
/// and provides the `Engine` trait implementation over it.
pub struct CdpEngine {
    browser: Browser,
    tab: Arc<Tab>,
    config: EngineConfig,

    // Optional callbacks
    on_load: Option<OnLoadHandler>,
    on_console: Option<OnConsoleHandler>,
    on_request: Option<OnRequestHandler>,
}

impl Engine for CdpEngine {
    fn new(config: EngineConfig) -> Result<Self>
    where
        Self: Sized,
    {
        // Configure headless Chrome launch options
        let launch_options = LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((config.viewport.width, config.viewport.height)))
            .build()
            .map_err(|e| Error::InitializationError(format!("Failed to build launch options: {}", e)))?;

        // Launch the browser
        let browser = Browser::new(launch_options)
            .map_err(|e| Error::InitializationError(format!("Failed to launch browser: {}", e)))?;

        // Get the first tab
        let tab = browser
            .new_tab()
            .map_err(|e| Error::InitializationError(format!("Failed to create tab: {}", e)))?;

        // Set user agent
        tab.set_user_agent(&config.user_agent, None, None)
            .map_err(|e| Error::InitializationError(format!("Failed to set user agent: {}", e)))?;

        // Set extra HTTP headers
        if !config.headers.is_empty() {
            // headless_chrome expects a HashMap<&str, &str>
            let headers: std::collections::HashMap<&str, &str> = config
                .headers
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            tab.set_extra_http_headers(headers)
                .map_err(|e| Error::InitializationError(format!("Failed to set headers: {}", e)))?;
        }

        // Enable/disable JavaScript (no-op if already configured)
        tab.enable_debugger().map_err(|e| Error::InitializationError(format!("Failed to enable debugger: {}", e)))?;

        Ok(Self {
            browser,
            tab,
            config,
            on_load: None,
            on_console: None,
            on_request: None,
        })
    }

    fn load_url(&mut self, url: &str) -> Result<()> {
        let _timeout = Duration::from_millis(self.config.timeout_ms);

        self.tab
            .navigate_to(url)
            .map_err(|e| Error::LoadError(format!("Navigation failed: {}", e)))?;

        self.tab
            .wait_until_navigated()
            .map_err(|e| Error::LoadError(format!("Wait for navigation failed: {}", e)))?;

        // Wait for the page to stabilize
        std::thread::sleep(Duration::from_millis(500));

        // Invoke on_load callback if registered
        if let Some(cb) = &self.on_load {
            if let Ok(snapshot) = self.render_text_snapshot() {
                cb(&snapshot);
            }
        }

        Ok(())
    }

    fn render_text_snapshot(&self) -> Result<TextSnapshot> {
        // Get the page title
        let title = self
            .tab
            .get_title()
            .map_err(|e| Error::RenderError(format!("Failed to get title: {}", e)))?;

        // Get the current URL
        let url = self.tab.get_url();

        // Extract text content from the body
        let eval = self
            .tab
            .evaluate(
                r#"
                (function() {
                    const body = document.body;
                    return body ? body.innerText : '';
                })()
                "#,
                false,
            )
            .map_err(|e| Error::RenderError(format!("Evaluation failed: {}", e)))?;

        let text = match eval.value {
            Some(val) => {
                if val.is_string() {
                    val.as_str().unwrap().to_string()
                } else {
                    val.to_string()
                }
            }
            None => return Err(Error::RenderError("No value returned from evaluation".into())),
        };

        Ok(TextSnapshot { title, text, url })
    }

    fn render_png(&self) -> Result<Vec<u8>> {
        let screenshot_data = self
            .tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| Error::RenderError(format!("Screenshot failed: {}", e)))?;

        Ok(screenshot_data)
    }

    fn evaluate_script(&mut self, script: &str) -> Result<ScriptResult> {
        if !self.config.enable_javascript {
            return Err(Error::ScriptError("JavaScript execution is disabled in the engine config".into()));
        }

        // If JS isolation is enabled, run the script inside a sandboxed iframe
        if self.config.enable_js_isolation {
            // Encode script as base64 so it can be embedded safely in srcdoc
            let b64 = Base64Engine::encode(&base64::engine::general_purpose::STANDARD, script);

            // The iframe posts a JSON-stringified message back to the parent. We build the
            // wrapper from a template and substitute the base64 script to avoid having to
            // escape braces for `format!`.
            let wrapper_template = r#"(async function(){
                return await new Promise(function(resolve){
                    const iframe = document.createElement('iframe');
                    iframe.sandbox = 'allow-scripts';
                    iframe.style.display = 'none';

                    iframe.srcdoc = '<!doctype html><script>(function(){try{const s=atob("{{B64_TOKEN}}");var _r;try{_r=(function(){return eval(s);})();}catch(e){_r={__rfox_err:String(e)};} var out = (_r && _r.__rfox_err) ? {error: String(_r.__rfox_err)} : {result: _r}; parent.postMessage(JSON.stringify(out), "*");}catch(e){parent.postMessage(JSON.stringify({error: String(e)}),"*");}})();</script>';

                    window.addEventListener('message', function handler(event){
                        try {
                            var data = event.data;
                            if (typeof data === 'string') data = JSON.parse(data);
                            if (data && (data.result !== undefined || data.error !== undefined)) {
                                window.removeEventListener('message', handler);
                                document.body.removeChild(iframe);
                                try { resolve(JSON.stringify(data)); } catch(e) { resolve(JSON.stringify({error: String(e)})); }
                            }
                        } catch(e) {
                            window.removeEventListener('message', handler);
                            document.body.removeChild(iframe);
                            try { resolve(JSON.stringify({error: String(e)})); } catch(e2) { resolve('{"error":"unknown"}'); }
                        }
                    }, false);

                    document.body.appendChild(iframe);
                });
            })()"#;

            let wrapper = wrapper_template.replace("{{B64_TOKEN}}", &b64);

            let eval_res = self
                .tab
                .evaluate(&wrapper, true)
                .map_err(|e| Error::ScriptError(format!("Island evaluation failed: {}", e)))?;

            let val = eval_res.value.ok_or_else(|| Error::ScriptError("No value returned from isolated evaluation".into()))?;

            // The iframe now posts a JSON string which is returned as a string value
            // from CDP; try to parse it into a JSON value for robust processing.
            let parsed = if val.is_string() {
                let s = val.as_str().unwrap_or("");
                match serde_json::from_str::<serde_json::Value>(s) {
                    Ok(v) => v,
                    Err(_) => serde_json::Value::String(s.to_string()),
                }
            } else {
                val
            };

            // The parsed value should be an object with either 'result' or 'error'.
            if parsed.get("error").is_some() {
                return Ok(ScriptResult { value: parsed.get("error").unwrap().to_string(), is_error: true });
            }

            if parsed.get("result").is_some() {
                return Ok(ScriptResult { value: parsed.get("result").unwrap().to_string(), is_error: false });
            }

            return Ok(ScriptResult { value: parsed.to_string(), is_error: false });
        }

        // Fall back to direct evaluation
        let result = self
            .tab
            .evaluate(script, false)
            .map_err(|e| Error::ScriptError(format!("Evaluation failed: {}", e)))?;

        let value = result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());

        Ok(ScriptResult {
            value,
            is_error: false,
        })
    }

    /// Direct page evaluation that runs in the page's global context and can access
    /// DOM properties such as `document.title`. This ignores `enable_js_isolation`.
    fn evaluate_script_in_page(&mut self, script: &str) -> Result<ScriptResult> {
        if !self.config.enable_javascript {
            return Err(Error::ScriptError("JavaScript execution is disabled in the engine config".into()));
        }

        let result = self
            .tab
            .evaluate(script, true)
            .map_err(|e| Error::ScriptError(format!("Direct evaluation failed: {}", e)))?;

        let value = result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());

        Ok(ScriptResult { value, is_error: false })
    }

    fn on_load<F>(&mut self, cb: F)
    where
        F: Fn(&crate::TextSnapshot) + Send + Sync + 'static,
    {
        self.on_load = Some(std::sync::Arc::new(cb));
    }

    fn clear_on_load(&mut self) {
        self.on_load = None;
    }

    fn on_console<F>(&mut self, cb: F)
    where
        F: Fn(&crate::ConsoleMessage) + Send + Sync + 'static,
    {
        let arc = std::sync::Arc::new(cb);

        // Expose a binding to receive console messages from the page
        let binding_name = "__rfox_console".to_string();
        let handler_arc = arc.clone();

        // The binding receives JSON payloads from the page script and forwards them
        // to the registered Rust callback.
        let _ = self
            .tab
            .expose_function(&binding_name, std::sync::Arc::new(move |payload: serde_json::Value| {
                // payload may be a JSON string
                let msg = if payload.is_string() {
                    let s = payload.as_str().unwrap_or("");
                    match serde_json::from_str::<serde_json::Value>(s) {
                        Ok(v) => v,
                        Err(_) => serde_json::Value::String(s.to_string()),
                    }
                } else {
                    payload
                };

                // Extract level and args
                if let Some(level) = msg.get("level") {
                    let level = level.as_str().unwrap_or("").to_string();
                    let text = match msg.get("args") {
                        Some(args) => {
                            if args.is_array() {
                                args
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            } else {
                                args.to_string()
                            }
                        }
                        None => String::new(),
                    };

                    let cm = crate::ConsoleMessage { level, text, source: None, line: None, column: None, stack: None };
                    (handler_arc)(&cm);
                }
            }))
            .map_err(|e| warn!("Failed to expose console binding: {}", e))
            .ok();
        // Inject a small script that wraps console methods to post messages to the binding
        let wrapper = r#"(function(){
            const rfox_bind = window.__rfox_console;
            if (!rfox_bind) return;
            ['log','info','warn','error'].forEach(function(k){
                const orig = console[k];
                console[k] = function(...args){
                    try{ rfox_bind(JSON.stringify({ level:k, args: args.map(a=>String(a)) })); }catch(e){}
                    try{ orig.apply(console, args); }catch(e){}
                };
            });
        })();"#;

        let _ = self
            .tab
            .call_method(Page::AddScriptToEvaluateOnNewDocument {
                source: wrapper.to_string(),
                world_name: None,
                include_command_line_api: None,
                run_immediately: None,
            })
            .map_err(|e| warn!("Failed to inject console wrapper: {}", e))
            .ok();

        self.on_console = Some(arc);
    }

    fn clear_on_console(&mut self) {
        self.on_console = None;
    }

    fn on_request<F>(&mut self, cb: F)
    where
        F: Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync + 'static,
    {
        let arc = std::sync::Arc::new(cb);
        let handler_arc = arc.clone();

        // Enable fetch domain so we can intercept requests
        let _ = self
            .tab
            .enable_fetch(None, Some(false))
            .map_err(|e| warn!("Failed to enable fetch domain: {}", e))
            .ok();

        // Register an interceptor that forwards request metadata to the handler
        let interceptor: std::sync::Arc<dyn RequestInterceptor + Send + Sync> = std::sync::Arc::new(
            move |_transport, _session_id, event: RequestPausedEvent| {
                let req = &event.params.request;
                let headers = std::collections::HashMap::new();

                let info = crate::RequestInfo {
                    request_id: event.params.request_id.clone(),
                    url: req.url.clone(),
                    method: req.method.clone(),
                    resource_type: None,
                    headers,
                };

                let action = (handler_arc)(&info);

                match action {
                    crate::RequestAction::Continue => RequestPausedDecision::Continue(None),
                    crate::RequestAction::Fail { error_reason } => {
                        // `failRequest` requires a specific ErrorReason enum in the
                        // protocol. For now, we do not map arbitrary strings to the
                        // protocol enum; log and continue the request. TODO: map or
                        // expose protocol enum to users for precise control.
                        warn!("on_request requested Fail('{}') but failing is not implemented; continuing", error_reason);
                        RequestPausedDecision::Continue(None)
                    },
                    crate::RequestAction::Fulfill { status, headers, body } => {
                        let header_entries = headers
                            .into_iter()
                            .map(|(k, v)| HeaderEntry { name: k, value: v })
                            .collect::<Vec<_>>();

                        let fulfill = FulfillRequest {
                            request_id: event.params.request_id.clone(),
                            response_code: status as u32,
                            response_headers: Some(header_entries),
                            binary_response_headers: None,
                            body: Some(base64::engine::general_purpose::STANDARD.encode(&body)),
                            response_phrase: None,
                        };

                        RequestPausedDecision::Fulfill(fulfill)
                    }
                }
            },
        );

        let _ = self
            .tab
            .enable_request_interception(interceptor)
            .map_err(|e| warn!("Failed to enable request interception: {}", e))
            .ok();

        self.on_request = Some(arc);
    }

    fn clear_on_request(&mut self) {
        self.on_request = None;
        // Note: we don't disable fetch here for simplicity
    }

    fn get_cookies(&self) -> Result<Vec<crate::Cookie>> {
        let cookies = self.tab.get_cookies().map_err(|e| Error::Other(format!("Failed to get cookies: {}", e)))?;
        let mapped = cookies
            .into_iter()
            .map(|c| crate::Cookie {
                name: c.name,
                value: c.value,
                domain: Some(c.domain),
                path: Some(c.path),
                expires: Some(c.expires as u64),
                size: Some(c.size),
                http_only: Some(c.http_only),
                secure: Some(c.secure),
                same_site: c.same_site.map(|s| format!("{:?}", s)),
            })
            .collect();
        Ok(mapped)
    }

    fn set_cookies(&mut self, cookies: Vec<crate::CookieParam>) -> Result<()> {
        use headless_chrome::protocol::cdp::Network::CookieParam as NetCookieParam;
        let net_cookies = cookies
            .into_iter()
            .map(|c| NetCookieParam {
                name: c.name,
                value: c.value,
                url: c.url,
                domain: c.domain,
                path: c.path,
                secure: c.secure,
                http_only: c.http_only,
                same_site: c.same_site.and_then(|s| match s.as_str() {
                    "Strict" | "strict" => Some(headless_chrome::protocol::cdp::Network::CookieSameSite::Strict),
                    "Lax" | "lax" => Some(headless_chrome::protocol::cdp::Network::CookieSameSite::Lax),
                    "None" | "none" => Some(headless_chrome::protocol::cdp::Network::CookieSameSite::None),
                    _ => None,
                }),
                expires: c.expires.map(|v| v as f64),
                priority: None,
                same_party: None,
                source_scheme: None,
                source_port: None,
                partition_key: None,
            })
            .collect();

        self.tab.set_cookies(net_cookies).map_err(|e| Error::Other(format!("Failed to set cookies: {}", e)))?;
        Ok(())
    }

    fn delete_cookie(&mut self, name: &str, url: Option<&str>, domain: Option<&str>, path: Option<&str>) -> Result<()> {
        use headless_chrome::protocol::cdp::Network::DeleteCookies as NetDelete;
        let nc = NetDelete {
            name: name.to_string(),
            url: url.map(|s| s.to_string()),
            domain: domain.map(|s| s.to_string()),
            path: path.map(|s| s.to_string()),
            partition_key: None,
        };
        self.tab.delete_cookies(vec![nc]).map_err(|e| Error::Other(format!("Failed to delete cookie: {}", e)))?;
        Ok(())
    }

    fn clear_cookies(&mut self) -> Result<()> {
        // Delete each cookie returned by get_cookies
        let cookies = self.tab.get_cookies().map_err(|e| Error::Other(format!("Failed to get cookies for clearing: {}", e)))?;
        let deletes = cookies
            .into_iter()
            .map(|c| headless_chrome::protocol::cdp::Network::DeleteCookies {
                name: c.name,
                url: None,
                domain: Some(c.domain),
                path: Some(c.path),
                partition_key: None,
            })
            .collect::<Vec<_>>();

        self.tab.delete_cookies(deletes).map_err(|e| Error::Other(format!("Failed to clear cookies: {}", e)))?;
        Ok(())
    }

    fn close(self) -> Result<()> {
        // Ensure underlying browser/tab are dropped explicitly so the child
        // process is terminated promptly and to avoid unused-field warnings.
        drop(self.browser);
        drop(self.tab);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_engine_creation() {
        let config = EngineConfig::default();
        // This test requires Chrome to be installed, so we skip it in CI
        if std::env::var("CI").is_ok() {
            return;
        }
        let result = CdpEngine::new(config);
        if let Err(e) = result {
            eprintln!("Skipping CDP engine creation test because Chrome is not available or failed to launch: {}", e);
            return;
        }
        assert!(result.is_ok());
    }
}
