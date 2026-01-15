use crate::Engine;
use crate::{cdp, EngineConfig, Error, Result, ScriptResult};
use std::sync::mpsc::{self, Sender};
use std::thread;
use tokio::sync::oneshot;

enum Command {
    Goto(String, oneshot::Sender<Result<()>>),
    Eval(String, oneshot::Sender<Result<ScriptResult>>),
    EvalInPage(String, oneshot::Sender<Result<ScriptResult>>),
    Screenshot(Option<String>, oneshot::Sender<Result<Vec<u8>>>),

    // Cookies
    GetCookies(oneshot::Sender<Result<Vec<crate::Cookie>>>),
    SetCookie(crate::CookieParam, oneshot::Sender<Result<()>>),
    DeleteCookie(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        oneshot::Sender<Result<()>>,
    ),
    ClearCookies(oneshot::Sender<Result<()>>),

    Close(oneshot::Sender<Result<()>>),
}

/// An async-friendly browser abstraction backed by a dedicated worker thread.
///
/// The worker thread owns a synchronous `CdpEngine` instance and executes
/// commands sent from async tasks so callers can use an async interface
/// without requiring the engine to be `Send` across threads.
#[derive(Clone)]
pub struct Browser {
    cmd_tx: Sender<Command>,
}

/// A handle representing a page/context in the browser.
#[derive(Clone)]
pub struct Page {
    cmd_tx: Sender<Command>,
}

impl Browser {
    /// Create a new browser (spawns a background thread that owns the engine).
    pub async fn new(config: Option<EngineConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let (init_tx, init_rx): (oneshot::Sender<Result<()>>, oneshot::Receiver<Result<()>>) =
            oneshot::channel();

        thread::spawn(move || {
            // Initialize engine on the worker thread
            let mut engine = match cdp::CdpEngine::new(config) {
                Ok(e) => e,
                Err(err) => {
                    let _ = init_tx.send(Err(err));
                    return;
                }
            };

            // Signal successful creation (no-op when previous send returned Err)
            let _ = init_tx.send(Ok(()));

            // Command loop
            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    Command::Goto(url, resp) => {
                        let res = engine.load_url(&url);
                        let _ = resp.send(res);
                    }
                    Command::Eval(script, resp) => {
                        let res = engine.evaluate_script(&script);
                        let _ = resp.send(res);
                    }
                    Command::EvalInPage(script, resp) => {
                        let res = engine.evaluate_script_in_page(&script);
                        let _ = resp.send(res);
                    }
                    Command::Screenshot(path_opt, resp) => {
                        let res = engine.render_png();
                        // If a path is provided, also write to disk
                        if let Ok(ref data) = res {
                            if let Some(path) = path_opt {
                                let _ = std::fs::write(path, data);
                            }
                        }
                        let _ = resp.send(res);
                    }

                    // Cookie commands
                    Command::GetCookies(resp) => {
                        let res = engine.get_cookies();
                        let _ = resp.send(res);
                    }

                    Command::SetCookie(param, resp) => {
                        let res = engine.set_cookies(vec![param]);
                        let _ = resp.send(res);
                    }

                    Command::DeleteCookie(name, url, domain, path, resp) => {
                        let res = engine.delete_cookie(
                            &name,
                            url.as_deref(),
                            domain.as_deref(),
                            path.as_deref(),
                        );
                        let _ = resp.send(res);
                    }

                    Command::ClearCookies(resp) => {
                        let res = engine.clear_cookies();
                        let _ = resp.send(res);
                    }
                    Command::Close(resp) => {
                        let res = engine.close();
                        let _ = resp.send(res);
                        break;
                    }
                }
            }
        });

        // Wait for init result
        // Wait for the worker to report initialization success or failure
        let init_res = init_rx
            .await
            .map_err(|e| Error::Other(format!("Worker init canceled: {}", e)))?;
        init_res?;

        Ok(Self { cmd_tx })
    }

    /// Open a new page handle backed by the same worker thread.
    pub async fn new_page(&self) -> Result<Page> {
        Ok(Page {
            cmd_tx: self.cmd_tx.clone(),
        })
    }

    /// Shutdown the background worker and close the browser.
    pub async fn close(self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::Close(tx));
        rx.await
            .map_err(|e| Error::Other(format!("Close canceled: {}", e)))?
    }

    // Browser-level convenience cookie helpers

    /// Convenience: get cookies for the current page
    pub async fn get_cookies(&self) -> Result<Vec<crate::Cookie>> {
        let page = self.new_page().await?;
        page.get_cookies().await
    }

    /// Convenience: set a cookie via the async facade
    pub async fn set_cookie(&self, param: crate::CookieParam) -> Result<()> {
        let page = self.new_page().await?;
        page.set_cookie(param).await
    }

    /// Convenience: delete a cookie (name, optional url/domain/path)
    pub async fn delete_cookie(
        &self,
        name: &str,
        url: Option<&str>,
        domain: Option<&str>,
        path: Option<&str>,
    ) -> Result<()> {
        let page = self.new_page().await?;
        page.delete_cookie(name, url, domain, path).await
    }

    /// Convenience: clear cookies for the current browser context
    pub async fn clear_cookies_all(&self) -> Result<()> {
        let page = self.new_page().await?;
        page.clear_cookies().await
    }
}

impl Page {
    /// Navigate to a URL
    pub async fn goto(&self, url: &str) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::Goto(url.to_string(), tx));
        rx.await
            .map_err(|e| Error::Other(format!("Goto canceled: {}", e)))?
    }

    /// Evaluate JavaScript and return the serialized result as a string
    pub async fn eval(&self, script: &str) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::Eval(script.to_string(), tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("Eval canceled: {}", e)))?;
        let sr = res?;
        Ok(sr.value)
    }

    /// Evaluate script directly in the page's global context (can access `document` etc.)
    pub async fn eval_in_page(&self, script: &str) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .cmd_tx
            .send(Command::EvalInPage(script.to_string(), tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("EvalInPage canceled: {}", e)))?;
        let sr = res?;
        Ok(sr.value)
    }

    /// Take a screenshot; if `path` is Some, the bytes will also be saved to that path.
    pub async fn screenshot(&self, path: Option<&str>) -> Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel();
        let path_opt = path.map(|s| s.to_string());
        let _ = self.cmd_tx.send(Command::Screenshot(path_opt, tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("Screenshot canceled: {}", e)))?;
        res
    }

    /// Get cookies for the current page
    pub async fn get_cookies(&self) -> Result<Vec<crate::Cookie>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::GetCookies(tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("GetCookies canceled: {}", e)))?;
        res
    }

    /// Set a single cookie (convenience)
    pub async fn set_cookie(&self, cookie: crate::CookieParam) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::SetCookie(cookie, tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("SetCookie canceled: {}", e)))?;
        res
    }

    /// Delete a cookie by name (domain/path optional to disambiguate)
    pub async fn delete_cookie(
        &self,
        name: &str,
        url: Option<&str>,
        domain: Option<&str>,
        path: Option<&str>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::DeleteCookie(
            name.to_string(),
            url.map(|s| s.to_string()),
            domain.map(|s| s.to_string()),
            path.map(|s| s.to_string()),
            tx,
        ));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("DeleteCookie canceled: {}", e)))?;
        res
    }

    pub async fn clear_cookies(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::ClearCookies(tx));
        let res = rx
            .await
            .map_err(|e| Error::Other(format!("ClearCookies canceled: {}", e)))?;
        res
    }
}
