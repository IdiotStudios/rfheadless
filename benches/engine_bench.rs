use criterion::{criterion_group, criterion_main, Criterion};

// Benchmarks exercise a couple of public engine paths when `rfengine` feature is enabled.
#[allow(dead_code)]
fn bench_render_snapshot(c: &mut Criterion) {
    if !cfg!(feature = "rfengine") {
        return;
    }

    use rfheadless::{Engine, EngineConfig};

    let cfg = EngineConfig {
        timeout_ms: 5000,
        ..Default::default()
    };
    let mut engine = rfheadless::new_engine(cfg).expect("failed to create engine");

    // Use a tiny server to provide content
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();
    std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let html = r#"<html><head><title>Bench</title></head><body><div id=hello>Hello RF</div></body></html>"#;
            let _ = req.respond(tiny_http::Response::from_string(html));
        }
    });

    let url = format!("http://{}", addr);
    engine.load_url(&url).expect("load failed");

    c.bench_function("render_text_snapshot", |b| {
        b.iter(|| {
            let _ = engine.render_text_snapshot().unwrap();
        })
    });
}

#[allow(dead_code)]
fn bench_evaluate_script(c: &mut Criterion) {
    if !cfg!(feature = "rfengine") {
        return;
    }
    use rfheadless::{Engine, EngineConfig};

    let cfg = EngineConfig::default();
    let mut engine = rfheadless::new_engine(cfg).expect("failed to create engine");

    // Preload a simple document
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();
    std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let html = r#"<html><head><title>Bench</title></head><body><div id=hello>Hello RF</div></body></html>"#;
            let _ = req.respond(tiny_http::Response::from_string(html));
        }
    });
    let url = format!("http://{}", addr);
    engine.load_url(&url).expect("load failed");

    c.bench_function("evaluate_script", |b| {
        b.iter(|| {
            let _ = engine
                .evaluate_script("document.querySelector('#hello').textContent()")
                .unwrap();
        })
    });
}

fn bench_stylesheet_fetch_concurrency(c: &mut Criterion) {
    if !cfg!(feature = "rfengine") {
        return;
    }
    use rfheadless::{Engine, EngineConfig};

    // Create a tiny server that serves an HTML referencing multiple stylesheets and
    // serves CSS contents for each stylesheet request.
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();

    // Prepare HTML with many stylesheet links
    let style_count = 8;
    let mut links = String::new();
    for i in 0..style_count {
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"/s{}.css\">", i));
    }
    let html = format!(
        "<html><head><title>Styles</title>{}</head><body></body></html>",
        links
    );

    // Spawn responder thread that handles style_count + 1 requests (one for HTML, style_count for CSS)
    std::thread::spawn(move || {
        let mut served = 0;
        while served < (style_count + 1) {
            if let Ok(req) = server.recv() {
                let html_clone = html.clone();
                std::thread::spawn(move || {
                    let path = req.url().to_string();
                    if path == "/" || path.is_empty() {
                        let _ = req.respond(tiny_http::Response::from_string(html_clone));
                    } else if path.starts_with("/s") && path.ends_with(".css") {
                        // Simulate a small delay to emulate network latency
                        std::thread::sleep(std::time::Duration::from_millis(30));
                        let css = "body{color:blue}".to_string();
                        let _ = req.respond(tiny_http::Response::from_string(css));
                    } else {
                        let _ = req.respond(tiny_http::Response::from_string(""));
                    }
                });
                served += 1;
            }
        }
    });

    let url = format!("http://{}", addr);

    // Benchmark both temporary runtime and persistent runtime configurations
    let cfg_temp = EngineConfig {
        enable_persistent_runtime: false,
        stylesheet_fetch_concurrency: 8,
        ..Default::default()
    };
    let mut engine_temp = rfheadless::new_engine(cfg_temp).expect("failed to create engine");

    let cfg_persistent = EngineConfig {
        enable_persistent_runtime: true,
        stylesheet_fetch_concurrency: 8,
        ..Default::default()
    };
    let mut engine_persistent =
        rfheadless::new_engine(cfg_persistent).expect("failed to create engine");

    c.bench_function("load_url_styles_temp_runtime", |b| {
        b.iter(|| {
            engine_temp.load_url(&url).expect("load failed");
        })
    });

    c.bench_function("load_url_styles_persistent_runtime", |b| {
        b.iter(|| {
            engine_persistent.load_url(&url).expect("load failed");
        })
    });
}
criterion_group!(
    benches,
    bench_render_snapshot,
    bench_evaluate_script,
    bench_stylesheet_fetch_concurrency
);
criterion_main!(benches);
