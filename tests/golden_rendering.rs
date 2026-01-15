use std::fs;
use std::path::PathBuf;

use rfheadless::rendering::raster::rasterize_with_seed;

fn golden_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from("tests/goldens/expected");
    p.push(name);
    p
}

#[test]
fn golden_raster_matches_fixture() {
    let page = fs::read_to_string("tests/goldens/pages/page1.html").expect("read fixture");
    let width = 256u32;
    let height = 128u32;

    // Use the page contents as the seed so the golden is content-addressed
    let screenshot = rasterize_with_seed(width, height, page.as_bytes());

    let expected_path = golden_path("page1.img");
    if std::env::var("UPDATE_GOLDENS").is_ok() {
        // write hex of digest
        fs::create_dir_all("tests/goldens/expected").ok();
        fs::write(&expected_path, hex::encode(&screenshot.png_data)).expect("write golden");
        println!("Updated golden: {:?}", expected_path);
        return;
    }

    if !expected_path.exists() {
        println!(
            "No golden at {:?}; run with UPDATE_GOLDENS=1 to create it. Skipping.",
            expected_path
        );
        return;
    }

    let exp = fs::read_to_string(&expected_path).expect("unable to read golden");
    let exp_bytes = hex::decode(exp.trim()).expect("invalid hex in golden");
    assert_eq!(screenshot.png_data, exp_bytes);
}
