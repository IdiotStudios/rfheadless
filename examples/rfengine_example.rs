use rfheadless::{Engine, EngineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(feature = "rfengine") {
        eprintln!("example requires the 'rfengine' feature; run: cargo run --example rfengine_example --features rfengine");
        return Ok(());
    }

    let cfg = EngineConfig::default();
    let mut engine = rfheadless::new_engine(cfg)?;

    // Use a tiny HTTP server to provide repeatable content
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();

    std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let html = r#"<html><head><title>RFEngine</title><style>.red{color:red}</style></head><body><div id=\"hello\" class=\"greeting\">Hello RF</div></body></html>"#;
            let _ = req.respond(tiny_http::Response::from_string(html));
        }
    });

    let url = format!("http://{}", addr);
    engine.load_url(&url)?;

    let snap = engine.render_text_snapshot()?;
    println!(
        "title: {}\ntext: {}\nurl: {}",
        snap.title, snap.text, snap.url
    );

    // Evaluate JS to select the element and return its text
    let res = engine
        .evaluate_script(r#"(function(){ return document.querySelector('#hello').text; })()"#)?;
    println!("eval result: {} (error={})", res.value, res.is_error);

    Ok(())
}
