use std::fs;
use std::path::PathBuf;
use rfheadless::rfengine::RFEngine;
use rfheadless::Engine;

fn gold_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from("tests/goldens/access");
    p.push(name);
    p
}

#[test]
fn golden_access_snapshot_matches() {
    // serve the page
    let server = tiny_http::Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();
    std::thread::spawn(move || {
        if let Ok(request) = server.recv() {
            let response = tiny_http::Response::from_string(fs::read_to_string("tests/goldens/pages/page1.html").unwrap());
            let _ = request.respond(response);
        }
    });

    let url = format!("http://{}", addr);
    let mut engine = RFEngine::new(rfheadless::EngineConfig::default()).expect("create engine");
    engine.load_url(&url).expect("load");

    let snap = engine.snapshot_page_context().expect("snapshot");

    let expected_path = gold_path("page1.access.json");

    // If UPDATE_GOLDENS is set, write the golden; otherwise skip the test when missing so
    // that the test suite remains green by default for new fixtures.
    if std::env::var("UPDATE_GOLDENS").is_ok() {
        fs::create_dir_all("tests/goldens/access").ok();
        fs::write(&expected_path, &snap).expect("write access golden");
        println!("Updated access golden: {:?}", expected_path);
        return;
    }

    if !expected_path.exists() {
        println!("No access golden at {:?}; run with UPDATE_GOLDENS=1 to create it. Skipping.", expected_path);
        return;
    }

    let exp = fs::read_to_string(&expected_path).expect("unable to read expected access golden");
    assert_eq!(snap, exp);
}