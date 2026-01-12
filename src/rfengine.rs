//! RFEngine: a non-Chrome engine with optional JS hook and CSS extraction
//!
//! This engine fetches HTML, extracts title/body and CSS (inline + linked).
//! It also supports evaluating JavaScript against a minimal page "document" via
//! the Boa JavaScript engine when the `rfengine` feature is enabled.


use crate::{Engine, EngineConfig, Error, Result, ScriptResult, TextSnapshot};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use std::sync::Arc;

type OnLoadHandler = Arc<dyn Fn(&TextSnapshot) + Send + Sync>;
type OnConsoleHandler = Arc<dyn Fn(&crate::ConsoleMessage) + Send + Sync>;
type OnRequestHandler = Arc<dyn Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync>;

// Helper to parse file:line:col from candidate substrings like "file.js:10:15"
fn parse_file_line_col(s: &str) -> Option<(String, u32, u32)> {
    let parts: Vec<&str> = s.rsplitn(3, ':').collect();
    if parts.len() >= 2 {
        if let (Ok(c), Ok(r)) = (parts[0].trim().trim_end_matches(')').parse::<u32>(), parts[1].trim().parse::<u32>()) {
            let src = if parts.len() == 3 { parts[2].trim().trim_start_matches("at ").trim().to_string() } else { "".to_string() };
            return Some((src, r, c));
        }
    }
    None
}

// Parse stack string lines to extract (source, line, column) best-effort.
fn parse_stack_info(stack: Option<&str>) -> (Option<String>, Option<u32>, Option<u32>) {
    if let Some(s) = stack {
        for l in s.lines() {
            let l = l.trim();
            // Pattern: "(file.js:10:15)" inside parentheses (V8-like)
            if let (Some(open), Some(close)) = (l.rfind('('), l.rfind(')')) {
                if open < close {
                    let inside = &l[open+1..close];
                    if let Some((src, ln, col)) = parse_file_line_col(inside) {
                        return (Some(src), Some(ln), Some(col));
                    }
                }
            }
            // Pattern: "func@file:line:col" (Firefox-like)
            if let Some(atpos) = l.find('@') {
                let after = &l[atpos+1..];
                if let Some((src, ln, col)) = parse_file_line_col(after) {
                    return (Some(src), Some(ln), Some(col));
                }
            }
            // Fallback: try to parse the end of line directly
            if let Some((src, ln, col)) = parse_file_line_col(l) {
                return (Some(src), Some(ln), Some(col));
            }
        }
    }
    (None, None, None)
}

pub struct RFEngine {
    client: Client,
    config: EngineConfig,
    last_html: Option<String>,
    last_url: Option<String>,
    styles: Vec<String>,

    on_load: Option<OnLoadHandler>,
    on_console: Option<OnConsoleHandler>,
    on_request: Option<OnRequestHandler>,
}

impl RFEngine {
    fn extract_styles(&mut self, base_url: &str) {
        if self.last_html.is_none() {
            return;
        }
        let html = self.last_html.as_ref().unwrap();
        let document = Html::parse_document(html);

        // Inline <style>
        let style_sel = Selector::parse("style").unwrap();
        for node in document.select(&style_sel) {
            let txt = node.text().collect::<String>();
            if !txt.trim().is_empty() {
                self.styles.push(txt);
            }
        }

        // <link rel="stylesheet" href="..."> — fetch referenced styles
        let link_sel = Selector::parse("link[rel=\"stylesheet\"]").unwrap();
        for node in document.select(&link_sel) {
            if let Some(href) = node.value().attr("href") {
                // Resolve href relative to base_url when possible
                let css_url = if let Ok(base) = url::Url::parse(base_url) {
                    base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
                } else {
                    href.to_string()
                };

                if let Ok(resp) = self.client.get(&css_url).send() {
                    if let Ok(text) = resp.text() {
                        if !text.trim().is_empty() {
                            self.styles.push(text);
                        }
                    }
                }
            }
        }
    }
}

impl Engine for RFEngine {
    fn new(config: EngineConfig) -> Result<Self>
    where
        Self: Sized,
    {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| Error::InitializationError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            last_html: None,
            last_url: None,
            styles: Vec::new(),
            on_load: None,
            on_console: None,
            on_request: None,
        })
    }

    fn load_url(&mut self, url: &str) -> Result<()> {
        let resp = self
            .client
            .get(url)
            .header("User-Agent", self.config.user_agent.clone())
            .send()
            .map_err(|e| Error::LoadError(format!("Failed to fetch {}: {}", url, e)))?;

        let body = resp
            .text()
            .map_err(|e| Error::LoadError(format!("Failed to read response body: {}", e)))?;

        self.last_html = Some(body);
        self.last_url = Some(url.to_string());

        // Extract styles (inline and linked)
        self.styles.clear();
        self.extract_styles(url);

        if let Some(cb) = &self.on_load {
            if let Ok(snapshot) = self.render_text_snapshot() {
                cb(&snapshot);
            }
        }

        Ok(())
    }

    fn render_text_snapshot(&self) -> Result<TextSnapshot> {
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

    fn render_png(&self) -> Result<Vec<u8>> {
        Err(Error::RenderError("Screenshots are not supported by RFEngine".into()))
    }

    fn evaluate_script(&mut self, script: &str) -> Result<ScriptResult> {
        if !self.config.enable_javascript {
            return Err(Error::ScriptError("JavaScript is disabled in config".into()));
        }

        // We use Boa to evaluate JS against a minimal "document" object and a
        // simple console that accumulates messages which are forwarded to the
        // registered `on_console` callback.
        let html = self
            .last_html
            .as_ref()
            .ok_or_else(|| Error::ScriptError("No document loaded".into()))?;

        // Build document fields and a lightweight DOM representation
        let document = Html::parse_document(html);
        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|n| n.text().collect::<String>())
            .unwrap_or_default();
        let body_text = document
            .select(&Selector::parse("body").unwrap())
            .next()
            .map(|n| n.text().collect::<String>())
            .unwrap_or_default();

        // Build a tree-aware list of elements for JS queries, including parent indices.
        // Each element contains tagName, id, className, textContent, attributes, and parent (index or null)
        let mut elements = Vec::new();
        // Start from the document root and do a depth-first traversal preserving document order.
        let root = document.root_element();
        let mut stack: Vec<(scraper::element_ref::ElementRef, Option<usize>)> = vec![(root, None)];
        while let Some((node, parent_idx)) = stack.pop() {
            let tag = node.value().name().to_string();
            let id = node.value().attr("id").map(|s| s.to_string()).unwrap_or_default();
            let class = node
                .value()
                .attr("class")
                .map(|s| s.to_string())
                .unwrap_or_default();
            let text = node.text().collect::<String>();
            let attrs = node
                .value()
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>();

            let idx = elements.len();
            elements.push(serde_json::json!({
                "tag": tag,
                "id": id,
                "class": class,
                "text": text,
                "attributes": attrs,
                "parent": parent_idx
            }));

            // Push children in reverse order so the traversal preserves document order.
            let children: Vec<_> = node
                .children()
                .filter_map(|n| scraper::ElementRef::wrap(n))
                .collect();
            for child in children.into_iter().rev() {
                stack.push((child, Some(idx)));
            }
        }

        let elements_json = serde_json::to_string(&elements).unwrap_or_else(|_| "[]".to_string());

        // Inject a small JS harness defining `document`, `console`, and query helpers; console forwards to host functions when available, else buffers messages in `__rfox_console`.
        let styles_array = self
            .styles
            .iter()
            .map(|s| serde_json::to_string(s).unwrap_or("\"\"".into()).to_string())
            .collect::<Vec<_>>()
            .join(",");





        // Inject harness from external template and substitute tokens
        let harness = include_str!("rf_harness.js")
            .replace("@ELEMENTS@", &elements_json)
            .replace("@STYLES@", &format!("[{}]", styles_array))
            .replace("@TITLE@", &serde_json::to_string(&title).unwrap_or_else(|_| "\"\"".to_string()))
            .replace("@BODY@", &serde_json::to_string(&body_text).unwrap_or_else(|_| "\"\"".to_string()));


        use std::collections::HashMap;
        use std::sync::{Mutex, OnceLock, Arc};
        use std::sync::mpsc::channel;
        use std::thread;

        #[allow(clippy::type_complexity)]
        static RFOX_CONSOLE_REG: OnceLock<Mutex<HashMap<usize, Arc<dyn Fn(&crate::ConsoleMessage) + Send + Sync>>>> = OnceLock::new();

        // Clone the console callback (if any) so we can move into the worker thread
        let on_console_cb = self.on_console.clone();
        let loop_limit = self.config.script_loop_iteration_limit;
        let recursion_limit = self.config.script_recursion_limit;
        let timeout_ms = self.config.script_timeout_ms;

        // We'll run the `harness + script` inside a worker thread and receive the result or timeout
        let (tx, rx) = channel();
        let code = format!("{}\n;\n(function(){{try{{return ({});}}catch(e){{throw e;}}}})()", harness, script);

        thread::spawn(move || {
            // Create a local context inside the thread
            let mut ctx: boa_engine::Context = boa_engine::Context::default();

            // Apply runtime limits from config
            if loop_limit > 0 {
                ctx.runtime_limits_mut().set_loop_iteration_limit(loop_limit);
            }
            if recursion_limit < usize::MAX {
                ctx.runtime_limits_mut().set_recursion_limit(recursion_limit);
            }

            // Native pointer function used by Boa to forward console messages.
            fn rfox_console_native(_this: &boa_engine::JsValue, args: &[boa_engine::JsValue], ctx: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue> {
                let ptr = ctx as *const _ as usize;
                let map = RFOX_CONSOLE_REG.get_or_init(|| Mutex::new(HashMap::new()));
                if let Ok(lock) = map.lock() {
                    if let Some(cb) = lock.get(&ptr) {
                        let text = args.first().map(|a| format!("{}", a.display())).unwrap_or_default();
                        let stack = args.get(1).map(|a| format!("{}", a.display())).filter(|s| !s.is_empty());
                        let (source, line_no, col_no) = parse_stack_info(stack.as_deref());
                        cb(&crate::ConsoleMessage { level: "log".to_string(), text, source, line: line_no, column: col_no, stack });
                    }
                }
                Ok(boa_engine::JsValue::undefined())
            }

            // Register console functions and the handler in the registry if provided
            if let Some(cb_ref) = &on_console_cb {
                let cb = cb_ref.clone();
                let ptr = &ctx as *const _ as usize;
                let map = RFOX_CONSOLE_REG.get_or_init(|| Mutex::new(HashMap::new()));
                if let Ok(mut lock) = map.lock() {
                    lock.insert(ptr, cb.clone());
                }
                let nf = boa_engine::native_function::NativeFunction::from_fn_ptr(rfox_console_native as boa_engine::native_function::NativeFunctionPointer);
                let _ = ctx.register_global_builtin_callable(boa_engine::js_string!("__rfox_console_log"), 0usize, nf);
                let nf2 = boa_engine::native_function::NativeFunction::from_fn_ptr(rfox_console_native as boa_engine::native_function::NativeFunctionPointer);
                let _ = ctx.register_global_builtin_callable(boa_engine::js_string!("__rfox_console_error"), 0usize, nf2);
            }

            let result = match ctx.eval(boa_engine::Source::from_bytes(code.as_bytes())) {
                Ok(val) => {
                    // deliver fallback buffered console messages (if any)
                    if let Ok(cmsg) = ctx.eval(boa_engine::Source::from_bytes("__rfox_console.join('\n')".as_bytes())) {
                        let console_text = format!("{}", cmsg.display());
                        if !console_text.is_empty() {
                            for line in console_text.split('\n') {
                                if let Some(cb) = &on_console_cb {
                                    let cm = crate::ConsoleMessage { level: "log".to_string(), text: line.to_string(), source: None, line: None, column: None, stack: None };
                                    cb(&cm);
                                }
                            }
                        }
                    }
                    Ok(ScriptResult { value: format!("{}", val.display()), is_error: false })
                }
                Err(e) => {
                    // deliver buffered console messages on error
                    if let Ok(cmsg) = ctx.eval(boa_engine::Source::from_bytes("__rfox_console.join('\n')".as_bytes())) {
                        let console_text = format!("{}", cmsg.display());
                        if !console_text.is_empty() {
                            for line in console_text.split('\n') {
                                if let Some(cb) = &on_console_cb {
                                    let cm = crate::ConsoleMessage { level: "error".to_string(), text: line.to_string(), source: None, line: None, column: None, stack: None };
                                    cb(&cm);
                                }
                            }
                        }
                    }
                    let err_msg = format!("Script thrown: {}", e);
                    Ok(ScriptResult { value: err_msg, is_error: true })
                }
            };

            // Clean up registry entry for this ctx
            let ptr = &ctx as *const _ as usize;
            let map = RFOX_CONSOLE_REG.get_or_init(|| Mutex::new(HashMap::new()));
            if let Ok(mut lock) = map.lock() {
                lock.remove(&ptr);
            }

            // send result back
            let _ = tx.send(result);
        });

        // Wait for the result with a timeout
        match rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
            Ok(r) => r,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Notify via console that we timed out
                if let Some(cb) = &self.on_console {
                    cb(&crate::ConsoleMessage { level: "error".into(), text: format!("Script timed out after {}ms", timeout_ms), source: None, line: None, column: None, stack: None });
                }
                Ok(ScriptResult { value: format!("Script timed out after {}ms", timeout_ms), is_error: true })
            }
            Err(e) => Ok(ScriptResult { value: format!("Script execution failed to receive result: {}", e), is_error: true }),
        }
    }

    fn evaluate_script_in_page(&mut self, script: &str) -> Result<ScriptResult> {
        // For RFEngine the semantics are the same as evaluate_script because there
        // is no separate remote page context.
        self.evaluate_script(script)
    }

    fn on_load<F>(&mut self, cb: F)
    where
        F: Fn(&crate::TextSnapshot) + Send + Sync + 'static,
    {
        self.on_load = Some(Arc::new(cb));
    }

    fn clear_on_load(&mut self) {
        self.on_load = None;
    }

    fn on_console<F>(&mut self, cb: F)
    where
        F: Fn(&crate::ConsoleMessage) + Send + Sync + 'static,
    {
        self.on_console = Some(Arc::new(cb));
    }

    fn clear_on_console(&mut self) {
        self.on_console = None;
    }

    fn on_request<F>(&mut self, cb: F)
    where
        F: Fn(&crate::RequestInfo) -> crate::RequestAction + Send + Sync + 'static,
    {
        self.on_request = Some(Arc::new(cb));
    }

    fn clear_on_request(&mut self) {
        self.on_request = None;
    }

    fn get_cookies(&self) -> Result<Vec<crate::Cookie>> {
        Ok(vec![])
    }

    fn set_cookies(&mut self, _cookies: Vec<crate::CookieParam>) -> Result<()> {
        Ok(())
    }

    fn delete_cookie(&mut self, _name: &str, _url: Option<&str>, _domain: Option<&str>, _path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn clear_cookies(&mut self) -> Result<()> {
        Ok(())
    }

    fn close(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfengine_load_and_eval() {
        // Skip on CI where network may not be available
        if std::env::var("CI").is_ok() {
            return;
        }

        let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
        let addr = server.server_addr();

        std::thread::spawn(move || {
            if let Ok(request) = server.recv() {
                let response = tiny_http::Response::from_string(
                    "<html><head><title>RF</title><style>body{color:red}</style></head><body><div id=\"hello\" class=\"greeting\">Hello RF</div></body></html>",
                );
                let _ = request.respond(response);
            }
        });

        let url = format!("http://{}", addr);
        let mut engine = RFEngine::new(crate::EngineConfig::default()).expect("Failed to create RFEngine");
        engine.load_url(&url).expect("Failed to load URL");
        let snap = engine.render_text_snapshot().expect("Failed to render snapshot");
        assert!(snap.title.contains("RF"));
        assert!(snap.text.contains("Hello RF"));

        // Test JS evaluation
        if engine.config.enable_javascript {
            let res = engine.evaluate_script("document.title").expect("Eval failed");
            assert!(res.value.contains("RF"));

            // Basic DOM query via querySelector and using safe `.textContent()` helper
            let res2 = engine.evaluate_script("document.querySelector('#hello').textContent()").expect("Eval failed");
            assert!(res2.value.contains("Hello"));

            // Missing selector should not throw and should return empty string
            let res_missing = engine.evaluate_script("document.querySelector('#nope').textContent()").expect("Eval failed");
            println!("missing -> value='{}' is_error={}", res_missing.value, res_missing.is_error);
            // Accept a few reasonable representations for empty/missing results
            let mut v = res_missing.value.trim().to_string();
            if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
                v = v[1..v.len()-1].to_string();
            }
            assert!(v.is_empty() || v == "null" || v == "undefined");

            // When debugging, dump the synthetic DOM for inspection
            let dom_dump = engine.evaluate_script("JSON.stringify(__rfox_dom)").expect("DOM dump failed");
            println!("__rfox_dom: {}", dom_dump.value);

            // Element helpers: getAttribute & setAttribute
            let attr = engine.evaluate_script("document.querySelector('#hello').getAttribute('class')").expect("Eval failed");
            assert!(attr.value.contains("greeting"));
            let res_dt = engine.evaluate_script("(()=>{ document.querySelector('#hello').setAttribute('data-test','42'); return document.querySelector('#hello').getAttribute('data-test'); })()").expect("Eval failed");
            assert!(res_dt.value.contains("42"));

            // Console forwarding using interior mutability
            let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let c_clone = captured.clone();
            let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let f_clone = flag.clone();
            engine.on_console(move |m| {
                f_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                if let Ok(mut v) = c_clone.lock() {
                    // store both text and stack so tests can assert metadata presence
                    v.push(format!("{}||{}", m.text.clone(), m.stack.clone().unwrap_or_default()));
                }
            });
            let _ = engine.evaluate_script("(()=>{ console.log('from-js'); return 'ok'; })()").expect("Eval failed");
            // Console calls should be forwarded synchronously when `on_console` is set.
            assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
            if let Ok(v) = captured.lock() {
                assert!(v.iter().any(|s| {
                    let parts: Vec<&str> = s.split("||").collect();
                    if parts.len() == 2 {
                        let head = parts[0].trim().trim_matches('"');
                        let tail = parts[1].trim().trim_matches('"');
                        head == "from-js" && !tail.is_empty()
                    } else { false }
                }));
            }

            // Try inline evaluation that logs and then returns join result (sanity checks)
            let res_inline = engine.evaluate_script("(()=>{ console.log('inline'); return __rfox_console.join('\\n'); })()").expect("inline eval failed");
            println!("inline console eval: {}", res_inline.value);

            // NOTE: on_console forwarding should now be deterministic for RFEngine
            // when a callback is registered; we assert above but keep fallback
            // behavior for environments without Boa host registration.
        }

        #[test]
        fn test_parse_stack_variants() {
            // V8-like
            let v8 = "Error\n    at Object.<anonymous> (/path/to/file.js:10:15)\n    at other";
            let (src, line, col) = super::parse_stack_info(Some(v8));
            assert!(src.unwrap_or_default().contains("/path/to/file.js"));
            assert_eq!(line, Some(10));
            assert_eq!(col, Some(15));

            // Firefox-like
            let ff = "func@http://localhost/script.js:20:5\nanother";
            let (src2, line2, col2) = super::parse_stack_info(Some(ff));
            assert!(src2.unwrap_or_default().contains("script.js"));
            assert_eq!(line2, Some(20));
            assert_eq!(col2, Some(5));

            // Minimal
            let minimal = "file.js:30:3";
            let (_s3, l3, c3) = super::parse_stack_info(Some(minimal));
            assert_eq!(l3, Some(30));
            assert_eq!(c3, Some(3));
        }

        #[test]
        fn test_element_api_and_computed_style() {
            // Skip on CI where network may not be available
            if std::env::var("CI").is_ok() {
                return;
            }

            let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
            let addr = server.server_addr();

            std::thread::spawn(move || {
                if let Ok(request) = server.recv() {
                    let response = tiny_http::Response::from_string(
                        "<html><head><title>RF</title><style>body{color:blue}.greeting{color:green}#hello{color:red;font-size:12px}</style></head><body><div id=\"hello\" class=\"greeting\">Hello RF</div></body></html>",
                    );
                    let _ = request.respond(response);
                }
            });

            let url = format!("http://{}", addr);
            let mut engine = RFEngine::new(crate::EngineConfig::default()).expect("Failed to create RFEngine");
            engine.load_url(&url).expect("Failed to load URL");

            if engine.config.enable_javascript {
                let ds = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.setAttribute('data-foo','bar'); return el.dataset.foo; })()").expect("Eval failed");
                assert!(ds.value.contains("bar"));

                let cls = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.classList.add('x'); var a=el.getAttribute('class'); el.classList.remove('x'); return a; })()").expect("Eval failed");
                assert!(cls.value.contains("x"));

                let contains = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.classList.add('y'); return el.classList.contains('y'); })()").expect("Eval failed");
                assert!(contains.value.contains("true"));

                let ih = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.innerHTML('<b>Bold</b>'); return el.innerHTML(); })()").expect("Eval failed");
                assert!(ih.value.contains("Bold"));

                // dataset.set should create/update data attributes
                let ds_set = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.dataset.set('foo','baz'); return el.getAttribute('data-foo'); })()").expect("Eval failed");
                assert!(ds_set.value.contains("baz"));

                // classList helpers and length()
                let cls = engine.evaluate_script("(()=>{ var el=document.querySelector('#hello'); el.classList.add('x'); var a=el.getAttribute('class'); var len=el.classList.length(); el.classList.remove('x'); return JSON.stringify({class:a,len:len}); })()").expect("Eval failed");
                assert!(cls.value.contains("x"));
                assert!(cls.value.contains("len"));

                // Specificity: id selector should override class and tag
                let spec = engine.evaluate_script("(()=>{ return getComputedStyle(document.querySelector('#hello')).getPropertyValue('color'); })()").expect("Eval failed");
                // colors are normalized to canonical form (e.g., #rrggbb)
                assert!(spec.value.contains("#ff0000"));
            }
        }

        #[test]
        fn test_script_timeout_and_runtime_limits() {
            // Skip on CI where network may not be available
            if std::env::var("CI").is_ok() {
                return;
            }

            let mut engine = RFEngine::new(crate::EngineConfig::default()).expect("Failed to create RFEngine");

            // Short timeout to trigger
            engine.config.script_timeout_ms = 10;
            if engine.config.enable_javascript {
                let res = engine.evaluate_script("(()=>{ while(true){} })() ").expect("Eval failed");
                assert!(res.is_error);
                assert!(res.value.to_lowercase().contains("timed out") || res.value.to_lowercase().contains("loop") || res.value.to_lowercase().contains("thrown"));
            }

            // Test loop iteration limit (should throw before runaway)
            engine.config.script_timeout_ms = 5000;
            engine.config.script_loop_iteration_limit = 100;
            if engine.config.enable_javascript {
                let res2 = engine.evaluate_script("(()=>{ var i=0; while(true) { i++; } })() ").expect("Eval failed");
                assert!(res2.is_error);
                assert!(res2.value.to_lowercase().contains("loop") || res2.value.to_lowercase().contains("thrown"));
            }
        }

        #[test]
        fn test_selector_combinators_and_attributes() {
            // Skip on CI where network may not be available
            if std::env::var("CI").is_ok() {
                return;
            }

            let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
            let addr = server.server_addr();

            std::thread::spawn(move || {
                if let Ok(request) = server.recv() {
                    let response = tiny_http::Response::from_string(
                        "<html><head><title>S</title></head><body><div id=\"outer\"><div class=\"mid\"><span class=\"inner\" data-test=\"x\">X</span></div></div></body></html>",
                    );
                    let _ = request.respond(response);
                }
            });

            let url = format!("http://{}", addr);
            let mut engine = RFEngine::new(crate::EngineConfig::default()).expect("Failed to create RFEngine");
            engine.load_url(&url).expect("Failed to load URL");

            if engine.config.enable_javascript {
                // descendant selector
                let res = engine.evaluate_script("(()=>{ return querySelector('div span').getAttribute('data-test'); })()").expect("Eval failed");
                assert!(res.value.contains("x"));

                // child combinator (div > span) should not match since span is a grandchild
                let res2 = engine.evaluate_script("(()=>{ return querySelector('div > span').getAttribute('data-test'); })()").expect("Eval failed");
                assert!(res2.value.contains("null") || res2.value.contains("undefined"));

                // attribute selector
                let res3 = engine.evaluate_script("(()=>{ return querySelector('[data-test=\"x\"]').tag; })()").expect("Eval failed");
                assert!(res3.value.contains("span"));

                // attribute operators and pseudo-classes
                let html = "<html><body><div id=\"p\"><span data-a=\"one two\">X</span><span data-a=\"two\">Y</span><span data-a=\"pre-suf\">Z</span></div></body></html>";
                // replace server response for this test
                // We'll directly evaluate against the current page's DOM by injecting elements
                let _ = engine.evaluate_script(&format!("(()=>{{ var d=document.querySelector('body'); d.innerHTML('{}'); return true; }})();", html.replace("'","\"")));

                // ~= (contains word)
                let r1 = engine.evaluate_script("(()=>{ return querySelector('[data-a~=\\\"two\\\"]').textContent(); })()").expect("Eval failed");
                assert!(r1.value.contains("Y") || r1.value.contains("X"));

                // ^= (starts-with)
                let r2 = engine.evaluate_script("(()=>{ return querySelector('[data-a^=\\\"pre\\\"]').textContent(); })()").expect("Eval failed");
                assert!(r2.value.contains("Z"));

                // $= (ends-with)
                let r3 = engine.evaluate_script("(()=>{ return querySelector('[data-a$=\\\"two\\\"]').textContent(); })()").expect("Eval failed");
                assert!(r3.value.contains("Y"));

                // |= (dash-separated)
                let r4 = engine.evaluate_script("(()=>{ return querySelector('[data-a|=\\\"pre\\\"]').textContent(); })()").expect("Eval failed");
                assert!(r4.value.contains("Z"));

                // pseudo-classes: first-child/last-child
                let r5 = engine.evaluate_script("(()=>{ return querySelector('#p span:first-child').textContent(); })()").expect("Eval failed");
                assert!(r5.value.contains("X"));
                let r6 = engine.evaluate_script("(()=>{ return querySelector('#p span:last-child').textContent(); })()").expect("Eval failed");
                assert!(r6.value.contains("Z"));
            }
        }

        #[cfg(feature = "cdp")]
        #[test]
        #[ignore]
        fn test_compare_with_chrome() {
            // Runs only when you explicitly set RUN_CHROMIUM_COMPARISONS=1 and have Chrome available
            if std::env::var("RUN_CHROMIUM_COMPARISONS").is_err() {
                return;
            }
            if std::env::var("CI").is_ok() {
                return;
            }

            use crate::cdp::CdpEngine;

            let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
            let addr = server.server_addr();

            std::thread::spawn(move || {
                if let Ok(request) = server.recv() {
                    let response = tiny_http::Response::from_string(
                        "<html><head><title>RF</title><style>body{color:blue}.greeting{color:green}#hello{color:red;font-size:12px}</style></head><body><div id=\"hello\" class=\"greeting\">Hello RF</div></body></html>",
                    );
                    let _ = request.respond(response);
                }
            });

            let url = format!("http://{}", addr);
            let mut rf = RFEngine::new(crate::EngineConfig::default()).expect("Failed to create RFEngine");
            rf.load_url(&url).expect("Failed to load URL");

            let mut c = match CdpEngine::new(crate::EngineConfig::default()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Skipping Chrome comparison; failed to start Chrome: {}", e);
                    return;
                }
            };
            c.load_url(&url).expect("Chrome failed to load URL");

            let rf_res = rf.evaluate_script("(()=>{ return getComputedStyle(document.querySelector('#hello')).getPropertyValue('color'); })()").expect("RF eval failed");
            let c_res = c.evaluate_script_in_page("(()=>{ return getComputedStyle(document.querySelector('#hello')).getPropertyValue('color'); })()").expect("Chrome eval failed");

            let rf_norm = rf_res.value.to_lowercase().replace('"', "").trim().to_string();
            let c_norm = c_res.value.to_lowercase().replace('"', "").trim().to_string();

            assert!(rf_norm == c_norm, "Computed styles diverged: rf='{}' chrome='{}'", rf_norm, c_norm);
        }
    }
}
