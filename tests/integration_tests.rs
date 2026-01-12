//! Integration tests for the headless engine

use rfheadless::{Engine, EngineConfig, Viewport};
use std::sync::{Arc, Mutex, Once};
use tiny_http::{Response, Server};

static INIT: Once = Once::new();

/// Start a simple test HTTP server
fn start_test_server() -> String {
    INIT.call_once(|| {
        std::thread::spawn(|| {
            let server = Server::http("127.0.0.1:18080").unwrap();
            for request in server.incoming_requests() {
                let path = request.url().to_string();
                let response = match path.as_str() {
                    "/" => Response::from_string(
                        r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<h1>Hello from Test Server</h1>
<p>This is a test page.</p>
</body>
</html>"#,
                    )
                    .with_header(
                        "Content-Type: text/html; charset=utf-8"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                    "/redirect" => Response::from_string("").with_status_code(302).with_header(
                        "Location: /"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                    _ => Response::from_string("Not Found").with_status_code(404),
                };
                let _ = request.respond(response);
            }
        });
        // Give the server time to start
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    "http://127.0.0.1:18080".to_string()
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_load_url() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    let result = engine.load_url(&base_url);
    assert!(result.is_ok(), "Failed to load URL: {:?}", result);

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_text_snapshot() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    let snapshot = engine
        .render_text_snapshot()
        .expect("Failed to render text snapshot");

    assert_eq!(snapshot.title, "Test Page");
    assert!(snapshot.text.contains("Hello from Test Server"));
    assert!(snapshot.text.contains("This is a test page"));

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_screenshot() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    let png_data = engine.render_png().expect("Failed to render PNG");

    // Check that we got PNG data
    assert!(png_data.len() > 100, "PNG data seems too small");
    // PNG files start with these magic bytes
    assert_eq!(&png_data[0..8], b"\x89PNG\r\n\x1a\n");

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_evaluate_script() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    let result = engine
        .evaluate_script("2 + 2")
        .expect("Failed to evaluate script");

    assert!(!result.is_error);
    assert!(result.value.contains("4"));

    engine.close().unwrap();
}

#[test]
#[ignore]
fn test_evaluate_isolated_context() {
    let base_url = start_test_server();
    let config = EngineConfig { enable_js_isolation: true, ..Default::default() };

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    // Script tries to access parent.document; inside a sandboxed iframe without
    // same-origin it should not be allowed — inner script can catch and return 'ERR'
    let script = r#"(function(){ try { parent.document; return 'SHOULD_NOT_REACH'; } catch(e) { return 'ERR'; } })()"#;

    let result = engine.evaluate_script(script).expect("Failed to evaluate isolated script");
    assert!(!result.is_error);
    assert!(result.value.contains("ERR"));

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_custom_user_agent() {
    let base_url = start_test_server();
    let config = EngineConfig {
        user_agent: "CustomBot/1.0".to_string(),
        ..Default::default()
    };

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    let result = engine
        .evaluate_script("navigator.userAgent")
        .expect("Failed to get user agent");

    assert!(result.value.contains("CustomBot/1.0"));

    // Test cookies round-trip
    engine.set_cookies(vec![rfheadless::CookieParam {
        name: "testcookie".to_string(),
        value: "abc".to_string(),
        url: None,
        domain: None,
        path: None,
        secure: None,
        http_only: None,
        same_site: None,
        expires: None,
    }]).expect("Failed to set cookie");

    let cookies = engine.get_cookies().expect("Failed to get cookies");
    assert!(cookies.iter().any(|c| c.name == "testcookie" && c.value == "abc"));

    engine.delete_cookie("testcookie", None, None, None).expect("Failed to delete cookie");

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_on_load_callback() {
    use std::sync::{Arc, Mutex};

    let base_url = start_test_server();
    let config = EngineConfig::default();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.on_load(move |snapshot| {
        let mut lock = called_clone.lock().unwrap();
        *lock = true;
        assert!(!snapshot.text.is_empty());
    });

    engine.load_url(&base_url).expect("Failed to load URL");

    // Give some time for callback to be invoked (synchronous in current impl)
    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(*called.lock().unwrap());

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_on_console_callback() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.on_console(move |msg| {
        let mut lock = called_clone.lock().unwrap();
        *lock = true;
        assert!(!msg.text.is_empty());
    });

    engine.load_url(&base_url).expect("Failed to load URL");

    // Evaluate a console message
    engine.evaluate_script_in_page("console.log('hello-from-test')").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(*called.lock().unwrap());

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_on_request_callback() {
    let base_url = start_test_server();
    let config = EngineConfig::default();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let mut engine = rfheadless::new_engine(config).expect("Failed to create engine");
    engine.on_request(move |req| {
        let mut lock = called_clone.lock().unwrap();
        *lock = true;
        assert!(!req.url.is_empty());
        // Allow the request to proceed
        rfheadless::RequestAction::Continue
    });

    engine.load_url(&base_url).expect("Failed to load URL");

    // Trigger a request by evaluating a script that fetches a resource
    engine.evaluate_script_in_page("fetch('/').then(()=>console.log('f'))").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(*called.lock().unwrap());

    engine.close().unwrap();
}

#[test]
#[ignore] // Requires Chrome to be installed
fn test_custom_viewport() {
    let config = EngineConfig {
        viewport: Viewport {
            width: 1920,
            height: 1080,
        },
        ..Default::default()
    };

    let engine = rfheadless::new_engine(config).expect("Failed to create engine");
    // Engine creation with custom viewport should succeed
    engine.close().unwrap();
}
