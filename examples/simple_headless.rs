//! Minimal headless example demonstrating the Engine API (feature: `rfengine`)

use rfheadless::{Engine, EngineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RFox Headless Engine - Minimal Example\n");

    // This example is intended to run with the `rfengine` feature enabled:
    // cargo run --example simple_headless --features rfengine
    if !cfg!(feature = "rfengine") {
        eprintln!("example requires the 'rfengine' feature; run: cargo run --example simple_headless --features rfengine");
        return Ok(());
    }

    // Configure engine with defaults; tweak timeouts or UA as needed
    let cfg = EngineConfig {
        timeout_ms: 10_000,
        ..Default::default()
    };

    let mut engine = rfheadless::new_engine(cfg)?;

    // Use a tiny HTTP server to provide deterministic content for the example
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();

    std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let html = r#"<html><head><title>RF Minimal</title><style>.g{color:blue}</style></head><body><div id=hello class=g>Hello RF</div></body></html>"#;
            let _ = req.respond(tiny_http::Response::from_string(html));
        }
    });

    let url = format!("http://{}", addr);
    println!("Loading: {}", url);
    engine.load_url(&url)?;

    let snap = engine.render_text_snapshot()?;
    println!(
        "Snapshot:\n  title: {}\n  text: {}\n  url: {}\n",
        snap.title, snap.text, snap.url
    );

    // Try to evaluate a small script against the page context. If JS is disabled
    // the engine will return an error; handle that gracefully.
    match engine.evaluate_script("document.querySelector('#hello').textContent()") {
        Ok(res) => println!("Eval result: {} (is_error={})", res.value, res.is_error),
        Err(e) => eprintln!("Script evaluation not available: {}", e),
    }

    engine.close()?;
    println!("Done.");

    Ok(())
}
