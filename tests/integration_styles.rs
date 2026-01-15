use std::time::Instant;

#[test]
fn test_stylesheet_parallel_load_time() {
    // Skip on CI where network or timing may be unreliable
    if std::env::var("CI").is_ok() {
        return;
    }

    use rfheadless::{Engine, EngineConfig};
    use tiny_http::Server;

    // Server that serves HTML referencing multiple stylesheets and CSS files
    let server = Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();

    let style_count = 16;
    let mut links = String::new();
    for i in 0..style_count {
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"/s{}.css\">", i));
    }
    let html = format!(
        "<html><head><title>T</title>{}</head><body></body></html>",
        links
    );

    // Spawn request handler that responds concurrently
    std::thread::spawn(move || {
        loop {
            if let Ok(req) = server.recv() {
                let html = html.clone();
                std::thread::spawn(move || {
                    let path = req.url().to_string();
                    if path == "/" || path.is_empty() {
                        let _ = req.respond(tiny_http::Response::from_string(html));
                    } else if path.starts_with("/s") && path.ends_with(".css") {
                        // emulate some latency
                        std::thread::sleep(std::time::Duration::from_millis(30));
                        let _ = req.respond(tiny_http::Response::from_string(
                            "body{color:blue}".to_string(),
                        ));
                    } else {
                        let _ = req.respond(tiny_http::Response::from_string(""));
                    }
                });
            }
        }
    });

    let url = format!("http://{}", addr);
    let cfg = EngineConfig {
        enable_persistent_runtime: true,
        stylesheet_fetch_concurrency: 4,
        ..Default::default()
    };
    let mut engine = rfheadless::new_engine(cfg).expect("failed to create engine");

    let t0 = Instant::now();
    engine.load_url(&url).expect("load failed");
    let elapsed = t0.elapsed().as_millis();

    assert!(elapsed < 400, "expected load < 400ms, got {}ms", elapsed);
}
