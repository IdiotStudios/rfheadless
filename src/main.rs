use clap::{Parser, Subcommand};
use rfheadless::Engine;
use std::io::{self, BufRead, Write};

#[derive(Parser)]
#[clap(author, version, about, long_about = "Note: You must use a subcommand. Passing a URL directly as the first argument without the `run` subcommand will not work. Use `rfheadless run <URL>`.")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load a URL, print a text snapshot and optionally save a screenshot (`--screenshot <path>`)
    Run {
        /// URL to load
        url: String,
        /// Save a screenshot to this path
        #[clap(long)]
        screenshot: Option<String>,
        /// Disable JavaScript
        #[clap(long, action = clap::ArgAction::SetTrue)]
        no_js: bool,
        /// Timeout in milliseconds
        #[clap(long, default_value_t = 30000)]
        timeout_ms: u64,
        /// Stylesheet fetch concurrency
        #[clap(long)]
        stylesheet_concurrency: Option<usize>,
        /// Disable persistent runtime
        #[clap(long, action = clap::ArgAction::SetTrue)]
        disable_persistent_runtime: bool,
    },

    /// Evaluate a small JS expression in the current page context and print result
    Eval {
        /// URL to load before evaluating (optional)
        #[clap(long)]
        url: Option<String>,
        /// JS script to evaluate (omit to read from stdin)
        script: Option<String>,
    },
    /// Take a screenshot of the last loaded page or a URL
    Screenshot {
        path: String,
        /// URL to load before taking screenshot (optional)
        #[clap(long)]
        url: Option<String>,
    },
    /// Abort currently running script(s)
    Abort,
    /// Cookie management
    Cookies {
        #[clap(subcommand)]
        action: CookieAction,
    },
    /// Config inspection & runtime toggles
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum CookieAction {
    /// List cookies for the current page/context
    List,
    /// Set a cookie (name value) optionally providing url/domain/path
    Set {
        name: String,
        value: String,
        #[clap(long)]
        url: Option<String>,
        #[clap(long)]
        domain: Option<String>,
        #[clap(long)]
        path: Option<String>,
    },
    /// Delete a cookie by name
    Delete {
        name: String,
        #[clap(long)]
        url: Option<String>,
        #[clap(long)]
        domain: Option<String>,
        #[clap(long)]
        path: Option<String>,
    },
    /// Clear all cookies
    Clear,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current engine configuration
    Show,
    /// Set stylesheet concurrency
    SetConcurrency { value: usize },
    /// Toggle persistent runtime
    SetPersistent {
        #[clap(subcommand)]
        action: PersistentAction,
    },
}

#[derive(Subcommand)]
enum PersistentAction {
    /// Enable persistent runtime
    Enable,
    /// Disable persistent runtime
    Disable,
}

fn worker_main() -> io::Result<()> {
    // Simple worker loop that reads JSON per line and evaluates using Boa
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Deserialize)]
    struct Job {
        id: u64,
        code: String,
        loop_limit: u64,
        recursion_limit: usize,
    }

    #[derive(Serialize)]
    struct Res {
        id: u64,
        value: String,
        is_error: bool,
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut ctx: boa_engine::Context = boa_engine::Context::default();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(job) = serde_json::from_str::<Job>(&line) {
            // apply limits
            if job.loop_limit > 0 {
                ctx.runtime_limits_mut()
                    .set_loop_iteration_limit(job.loop_limit);
            }
            if job.recursion_limit < usize::MAX {
                ctx.runtime_limits_mut()
                    .set_recursion_limit(job.recursion_limit);
            }
            let res = match ctx.eval(boa_engine::Source::from_bytes(job.code.as_bytes())) {
                Ok(v) => Res {
                    id: job.id,
                    value: format!("{}", v.display()),
                    is_error: false,
                },
                Err(e) => Res {
                    id: job.id,
                    value: format!("Script thrown: {}", e),
                    is_error: true,
                },
            };
            let js = serde_json::to_string(&res).unwrap_or_else(|_| {
                format!(
                    "{{\"id\":{},\"value\":\"serialization failed\",\"is_error\":true}}",
                    job.id
                )
            });
            writeln!(out, "{}", js)?;
            out.flush()?;
        } else {
            // ignore malformed lines
        }
    }
    Ok(())
}

fn run_cli_cmd(run: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match run {
        Commands::Run {
            url,
            screenshot,
            no_js,
            timeout_ms,
            stylesheet_concurrency,
            disable_persistent_runtime,
        } => {
            let cfg = rfheadless::EngineConfig {
                enable_javascript: !no_js,
                timeout_ms,
                stylesheet_fetch_concurrency: stylesheet_concurrency.unwrap_or_default(),
                enable_persistent_runtime: !disable_persistent_runtime,
                ..Default::default()
            };

            let mut engine = rfheadless::new_engine(cfg)?;
            engine.load_url(&url)?;
            let snap = engine.render_text_snapshot()?;
            println!(
                "Title: {}\nURL: {}\nText preview:\n{}",
                snap.title,
                snap.url,
                &snap.text.chars().take(400).collect::<String>()
            );
            if let Some(path) = screenshot {
                match engine.render_png() {
                    Ok(p) => {
                        let _ = std::fs::write(path, p);
                        println!("Screenshot saved");
                    }
                    Err(e) => eprintln!("Screenshot failed: {}", e),
                }
            }
            engine.close()?;
        }
        Commands::Eval { url, script } => {
            // For Eval we use defaults and enable JS
            let cfg = rfheadless::EngineConfig::default();
            let mut engine = rfheadless::new_engine(cfg)?;

            // Optionally load a URL into the engine before evaluating
            if let Some(u) = url {
                if let Err(e) = engine.load_url(&u) {
                    eprintln!("Failed to load URL for eval: {}", e);
                    let _ = engine.close();
                    return Ok(());
                }
            }

            let script_text = match script {
                Some(s) => s,
                None => {
                    // Read script from stdin when not provided as an argument
                    use std::io::Read;
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };

            match engine.evaluate_script(&script_text) {
                Ok(res) => println!("Result: {} (is_error={})", res.value, res.is_error),
                Err(e) => eprintln!("Eval failed: {}", e),
            }
            let _ = engine.close();
        }
        Commands::Screenshot { path, url } => {
            let cfg = rfheadless::EngineConfig::default();
            let mut engine = rfheadless::new_engine(cfg)?;

            // Must have a page loaded to take a screenshot; allow loading a URL
            if let Some(u) = url {
                if let Err(e) = engine.load_url(&u) {
                    eprintln!("Failed to load URL for screenshot: {}", e);
                    let _ = engine.close();
                    return Ok(());
                }
            } else {
                eprintln!("No page loaded. Pass `--url <URL>` to load a page before taking a screenshot.");
                let _ = engine.close();
                return Ok(());
            }

            match engine.render_png() {
                Ok(p) => {
                    let _ = std::fs::write(path, p);
                    println!("Screenshot saved");
                }
                Err(e) => eprintln!("Screenshot failed: {}", e),
            }
            let _ = engine.close();
        }
        Commands::Abort => {
            // Abort is only supported for the `rfengine` backend which provides
            // a direct `abort_running_script` helper. We provide a helpful message
            // when the feature is not enabled.
            #[cfg(feature = "rfengine")]
            {
                let cfg = rfheadless::EngineConfig::default();
                let mut engine = rfheadless::rfengine::RFEngine::new(cfg)?;
                if let Err(e) = engine.abort_running_script() {
                    eprintln!("Abort failed: {}", e);
                } else {
                    println!("Abort requested");
                }
                let _ = engine.close();
            }
            #[cfg(not(feature = "rfengine"))]
            {
                eprintln!("Abort command requires the 'rfengine' feature (compile with --features rfengine)");
            }
        }
        Commands::Cookies { action } => {
            let cfg = rfheadless::EngineConfig::default();
            let mut engine = rfheadless::new_engine(cfg)?;
            match action {
                CookieAction::List => match engine.get_cookies() {
                    Ok(c) => {
                        for ck in c {
                            println!("{}={} (domain={:?})", ck.name, ck.value, ck.domain);
                        }
                    }
                    Err(e) => eprintln!("Failed to list cookies: {}", e),
                },
                CookieAction::Set {
                    name,
                    value,
                    url,
                    domain,
                    path,
                } => {
                    let param = rfheadless::CookieParam {
                        name,
                        value,
                        url,
                        domain,
                        path,
                        secure: None,
                        http_only: None,
                        same_site: None,
                        expires: None,
                    };
                    if let Err(e) = engine.set_cookies(vec![param]) {
                        eprintln!("Failed to set cookie: {}", e);
                    } else {
                        println!("Cookie set");
                    }
                }
                CookieAction::Delete {
                    name,
                    url,
                    domain,
                    path,
                } => {
                    if let Err(e) = engine.delete_cookie(
                        &name,
                        url.as_deref(),
                        domain.as_deref(),
                        path.as_deref(),
                    ) {
                        eprintln!("Failed to delete cookie: {}", e);
                    } else {
                        println!("Cookie delete attempted");
                    }
                }
                CookieAction::Clear => {
                    if let Err(e) = engine.clear_cookies() {
                        eprintln!("Failed to clear cookies: {}", e);
                    } else {
                        println!("Cookies cleared");
                    }
                }
            }
            let _ = engine.close();
        }
        Commands::Config { action } => {
            // Config commands operate on the EngineConfig values. We do not mutate
            // running engines from the CLI; instead we display or advise how to
            // change the configuration for subsequent runs.
            let cfg = rfheadless::EngineConfig::default();
            match action {
                ConfigAction::Show => {
                    println!("EngineConfig defaults: {:?}", cfg);
                }
                ConfigAction::SetConcurrency { value } => {
                    println!("To run with a different stylesheet fetch concurrency, use: `rfheadless run --stylesheet-concurrency {}`\nThis will affect the next run of the engine.", value);
                }
                ConfigAction::SetPersistent { action } => {
                    match action {
                        PersistentAction::Enable => println!("Persistent runtime is enabled by default. To disable it for a run, pass: `rfheadless run --disable-persistent-runtime`.\nThis will affect the next run of the engine."),
                        PersistentAction::Disable => println!("Persistent runtime is disabled. To enable it (default behavior), run without `--disable-persistent-runtime`.\nThis will affect the next run of the engine."),
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    // If invoked as a worker process (spawned by RFEngine), run the worker loop
    if std::env::args().nth(1).as_deref() == Some("--worker") {
        if let Err(e) = worker_main() {
            eprintln!("Worker failed: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();
    if let Err(e) = run_cli_cmd(cli.command) {
        eprintln!("Command failed: {}", e);
        std::process::exit(1);
    }
}
