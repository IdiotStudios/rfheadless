//! Integration test for request fulfill behavior

use rfheadless::{Engine, EngineConfig, RequestAction};
use std::sync::Once;
use tiny_http::{Response, Server};

static INIT: Once = Once::new();

fn start_test_server() -> String {
    INIT.call_once(|| {
        std::thread::spawn(|| {
            let server = Server::http("127.0.0.1:18081").unwrap();
            for request in server.incoming_requests() {
                // Return a JavaScript file
                if request.url().ends_with("/script.js") {
                    let resp = Response::from_string("console.log('served');").with_header(
                        "Content-Type: application/javascript"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    );
                    let _ = request.respond(resp);
                } else {
                    let resp = Response::from_string("<html><head></head><body></body></html>")
                        .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
                    let _ = request.respond(resp);
                }
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    "http://127.0.0.1:18081".to_string()
}

#[test]
#[ignore]
fn test_fulfill_request() {
    let base = start_test_server();
    let config = EngineConfig::default();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");

    // Fulfill requests for script.js with a custom body
    engine.on_request(|req| {
        if req.url.ends_with("/script.js") {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Content-Type".to_string(), "application/javascript".to_string());
            RequestAction::Fulfill { status: 200, headers, body: b"console.log('injected');".to_vec() }
        } else {
            RequestAction::Continue
        }
    });

    engine.load_url(&format!("{}/", base)).expect("load");

    // Evaluate code that loads /script.js and runs it
    engine.evaluate_script_in_page("var s=document.createElement('script'); s.src='/script.js'; document.head.appendChild(s);")
        .expect("eval");

    // Allow events
    std::thread::sleep(std::time::Duration::from_millis(200));

    engine.close().unwrap();
}
