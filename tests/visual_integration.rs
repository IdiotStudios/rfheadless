use rfheadless::{Engine, EngineConfig, Viewport};
use std::sync::{Arc, Once};
use tiny_http::{Response, Server};
use std::fs;
use std::path::PathBuf;

static INIT_VIS: Once = Once::new();

fn start_vis_server() -> String {
    INIT_VIS.call_once(|| {
        std::thread::spawn(|| {
            let server = Server::http("127.0.0.1:18082").unwrap();
            for request in server.incoming_requests() {
                let path = request.url().to_string();
                let response = match path.as_str() {
                    "/" => Response::from_string(
                        r#"<!DOCTYPE html>
<html>
<head><title>Visual Test Page</title></head>
<body>
<h1>Hello Visual</h1>
<p>This is a visual integration test.</p>
</body>
</html>"#,
                    )
                    .with_header(
                        "Content-Type: text/html; charset=utf-8"
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

    "http://127.0.0.1:18082".to_string()
}

fn golden_path() -> PathBuf {
    let mut p = PathBuf::from("tests/goldens/expected");
    p.push("visual_page1.img");
    p
}

#[test]
fn visual_integration_screenshot() {
    let base_url = start_vis_server();
    let cfg = EngineConfig {
        viewport: Viewport { width: 256, height: 128 },
        ..Default::default()
    };

    let mut engine = rfheadless::new_engine(cfg).expect("Failed to create engine");
    engine.load_url(&base_url).expect("Failed to load URL");

    let png_data = engine.render_png().expect("Failed to render PNG");

    // Basic sanity checks
    assert!(png_data.len() > 100, "PNG data seems too small");
    assert_eq!(&png_data[0..8], b"\x89PNG\r\n\x1a\n");

    // If UPDATE_GOLDENS is set, overwrite the golden file
    let gpath = golden_path();
    if std::env::var("UPDATE_GOLDENS").is_ok() {
        fs::create_dir_all(gpath.parent().unwrap()).ok();
        fs::write(&gpath, hex::encode(&png_data)).expect("write golden");
        eprintln!("Updated visual golden: {:?}", gpath);
        return;
    }

    // If golden exists, compare exact bytes
    if gpath.exists() {
        let exp_hex = fs::read_to_string(&gpath).expect("read golden");
        let exp_bytes = hex::decode(exp_hex.trim()).expect("invalid hex in golden");
        assert_eq!(png_data, exp_bytes, "PNG output does not match golden");
        return;
    }

    // Otherwise, perform pixel-level checks (ensure text rendered)
    let decoder = png::Decoder::new(&png_data[..]);
    let mut reader = decoder.read_info().expect("decode");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("frame");
    let bytes = &buf[..info.buffer_size()];

    // Check dimensions match
    assert_eq!(info.width, 256);
    assert_eq!(info.height, 128);

    // Look for a black pixel (text) and white pixel (background)
    let mut found_black = false;
    let mut found_white = false;
    for chunk in bytes.chunks(4) {
        if chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 255 {
            found_black = true;
        }
        if chunk[0] == 255 && chunk[1] == 255 && chunk[2] == 255 && chunk[3] == 255 {
            found_white = true;
        }
        if found_black && found_white {
            break;
        }
    }
    assert!(found_black, "Expected rendered text pixels (black) in PNG");
    assert!(found_white, "Expected white background pixels in PNG");

    engine.close().ok();
}