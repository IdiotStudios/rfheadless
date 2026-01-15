use rfheadless::rfengine::RFEngine;
use rfheadless::Engine;
use std::fs;
use tiny_http::Server;

#[test]
fn test_computed_style_golden() {
    let data =
        fs::read_to_string("tests/computed_style_golden.json").expect("Failed to read fixtures");
    let fixtures: serde_json::Value = serde_json::from_str(&data).expect("Invalid JSON");
    for f in fixtures.as_array().unwrap() {
        let html = f.get("html").unwrap().as_str().unwrap();
        let selector = f.get("selector").unwrap().as_str().unwrap();
        let property = f.get("property").unwrap().as_str().unwrap();
        let expected = f.get("expected").unwrap().as_str().unwrap();

        let server = Server::http("0.0.0.0:0").unwrap();
        let addr = server.server_addr();
        let html = html.to_string();

        std::thread::spawn(move || {
            if let Ok(request) = server.recv() {
                let response = tiny_http::Response::from_string(html);
                let _ = request.respond(response);
            }
        });

        let url = format!("http://{}", addr);
        let mut engine =
            RFEngine::new(rfheadless::EngineConfig::default()).expect("Failed to create RFEngine");
        engine.load_url(&url).expect("Failed to load URL");
        // Sanity check: ensure styles were extracted and passed to the harness
        let styles_check = engine
            .evaluate_script("(()=>{ return JSON.stringify(document.styles); })()")
            .expect("Eval failed");
        if styles_check.value.trim() == "\"[]\"" || styles_check.value.trim().is_empty() {
            panic!(
                "No styles found in harness for fixture; document.styles = {}",
                styles_check.value
            );
        }

        let script = format!("(()=>{{ return getComputedStyle(document.querySelector('{selector}')).getPropertyValue('{property}'); }})()", selector=selector, property=property);
        let res = engine.evaluate_script(&script).expect("Eval failed");
        let val = res.value.trim().trim_matches('"').to_lowercase();
        assert_eq!(
            val,
            expected.to_lowercase(),
            "Mismatch for selector {} property {} (document.styles={})",
            selector,
            property,
            styles_check.value
        );
    }
}
