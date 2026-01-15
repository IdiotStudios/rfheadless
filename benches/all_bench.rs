use criterion::Criterion;
use std::time::Instant;

// Consolidated benchmark suite for rfheadless. Run with:
//    cargo bench --features rfengine

/// Bench: render_text_snapshot
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
            engine.render_text_snapshot().unwrap();
        })
    });
}

/// Bench: evaluate_script
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

/// Bench: stylesheet fetch concurrency (temp vs persistent runtime)
fn bench_stylesheet_fetch_concurrency(c: &mut Criterion) {
    if !cfg!(feature = "rfengine") {
        return;
    }
    use rfheadless::{Engine, EngineConfig};

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

    // Spawn responder thread that handles the HTML and CSS requests
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

/// Micro-benchmark: latency percentiles (p50/p95/p99) using a local server.
///
/// This bench is executed as part of `cargo bench` and prints percentile values
/// in addition to Criterion's reports. Configure iterations with `BENCH_ITERATIONS`.
fn bench_latency_percentiles(_c: &mut Criterion) {
    if !cfg!(feature = "rfengine") {
        return;
    }
    use rfheadless::{Engine, EngineConfig};

    // Local tiny server
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();

    // Prepare HTML with several stylesheet links
    let style_count = 8usize;
    let mut links = String::new();
    for i in 0..style_count {
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"/s{}.css\">", i));
    }
    let html = format!(
        "<html><head><title>Lat</title>{}</head><body></body></html>",
        links
    );

    std::thread::spawn(move || loop {
        if let Ok(req) = server.recv() {
            let html = html.clone();
            std::thread::spawn(move || {
                let path = req.url().to_string();
                if path == "/" || path.is_empty() {
                    let _ = req.respond(tiny_http::Response::from_string(html));
                } else if path.starts_with("/s") && path.ends_with(".css") {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    let css = "body{color:blue}".to_string();
                    let _ = req.respond(tiny_http::Response::from_string(css));
                } else {
                    let _ = req.respond(tiny_http::Response::from_string(""));
                }
            });
        }
    });

    let url = format!("http://{}", addr);
    let iterations: usize = std::env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let warmup = 2usize;

    let cfg = EngineConfig {
        enable_persistent_runtime: true,
        ..Default::default()
    };
    let mut eng = rfheadless::new_engine(cfg).expect("failed to create engine");

    // Warmup
    for _ in 0..warmup {
        eng.load_url(&url).expect("warmup failed");
    }

    // Collect samples
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = Instant::now();
        eng.load_url(&url).expect("load failed");
        samples.push(t0.elapsed().as_millis() as u64);
    }

    samples.sort_unstable();
    let p50 = percentile(&samples, 50.0);
    let p95 = percentile(&samples, 95.0);
    let p99 = percentile(&samples, 99.0);

    println!("[latency_percentiles] samples={:?}", samples);
    println!(
        "[latency_percentiles] p50={}ms p95={}ms p99={}ms",
        p50, p95, p99
    );
}

fn percentile(samples: &[u64], pct: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let n = samples.len();
    let rank = ((pct / 100.0) * (n as f64)).ceil() as usize;
    let idx = if rank == 0 {
        0
    } else {
        rank.saturating_sub(1).min(n - 1)
    };
    samples[idx]
}

// Run benches manually so we can print percentile output to the console
fn main() {
    // If the `rfengine` feature isn't enabled this suite is mostly a no-op and the
    // percentile microbench will not run; print a helpful instruction instead.
    if !cfg!(feature = "rfengine") {
        println!("Bench suite: 'rfengine' feature not enabled. Run with: `cargo bench --features rfengine` to see latency percentiles and RFEngine-specific benches.");
        return;
    }
    // Create a Criterion instance, run the standard benchmark suites, then run the
    // percentile microbench and output percentiles to stderr so `cargo bench` shows them.
    let mut c = Criterion::default();

    bench_render_snapshot(&mut c);
    bench_evaluate_script(&mut c);
    bench_stylesheet_fetch_concurrency(&mut c);

    // Finalize criterion reports (writes reports into target/criterion)
    c.final_summary();

    // Run microbench and print percentiles (stderr to make it visible among Criterion output)
    bench_latency_percentiles(&mut c);
}
