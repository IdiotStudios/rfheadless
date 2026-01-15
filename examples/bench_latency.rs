//! Quick latency runner (prints p50/p95/p99) — useful for local checks.
//! Run with: cargo run --example bench_latency --features rfengine

use rfheadless::Engine;
use std::time::Instant;
use tiny_http::Server;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = Server::http("0.0.0.0:0")?;
    let addr = server.server_addr();

    // Prepare HTML with multiple stylesheet links (single-host) to measure parallel fetch
    let style_count = 8usize;
    let mut links = String::new();
    for i in 0..style_count {
        links.push_str(&format!("<link rel=\"stylesheet\" href=\"/s{}.css\">", i));
    }
    let html = format!(
        "<html><head><title>Lat</title>{}</head><body></body></html>",
        links
    );

    // Responder thread serves HTML and delayed CSS responses to emulate network latency
    std::thread::spawn(move || {
        loop {
            if let Ok(req) = server.recv() {
                let html = html.clone();
                std::thread::spawn(move || {
                    let path = req.url().to_string();
                    if path == "/" || path.is_empty() {
                        let _ = req.respond(tiny_http::Response::from_string(html));
                    } else if path.starts_with("/s") && path.ends_with(".css") {
                        // small artificial delay to model fetch latency
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        let css = "body{color:blue}".to_string();
                        let _ = req.respond(tiny_http::Response::from_string(css));
                    } else {
                        let _ = req.respond(tiny_http::Response::from_string(""));
                    }
                });
            }
        }
    });

    let url = format!("http://{}", addr);
    let iterations: usize = std::env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let warmup = 2usize;
    let threshold_ms: u64 = std::env::var("PERF_P95_THRESHOLD_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);

    let cfg = rfheadless::EngineConfig {
        enable_persistent_runtime: true,
        ..Default::default()
    };
    let mut eng = rfheadless::new_engine(cfg)?;

    // Warmup
    for _ in 0..warmup {
        eng.load_url(&url)?;
    }

    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = Instant::now();
        eng.load_url(&url)?;
        let ms = t0.elapsed().as_millis() as u64;
        samples.push(ms);
    }

    samples.sort_unstable();

    let p50 = percentile(&samples, 50.0);
    let p95 = percentile(&samples, 95.0);
    let p99 = percentile(&samples, 99.0);

    println!("Samples: {:?}", samples);
    println!(
        "p50={}ms p95={}ms p99={}ms (threshold={}ms)",
        p50, p95, p99, threshold_ms
    );

    if p95 > threshold_ms {
        eprintln!(
            "Performance regression: p95 {}ms > threshold {}ms",
            p95, threshold_ms
        );
        std::process::exit(1);
    }

    Ok(())
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
