//! Text snapshot example - demonstrates extracting page content

use rfheadless::{Engine, EngineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RFox Headless Engine - Text Snapshot Example\n");

    let config = EngineConfig {
        user_agent: "RFox-TextBot/1.0".to_string(),
        enable_images: false, // Don't load images for faster text extraction
        ..Default::default()
    };

    let mut engine = rfheadless::new_engine(config)?;

    // Test with multiple URLs
    let urls = vec![
        "https://example.com",
        "https://www.rust-lang.org",
    ];

    for url in urls {
        println!("Processing: {}", url);
        println!("{}", "=".repeat(60));

        match engine.load_url(url) {
            Ok(_) => {
                let snapshot = engine.render_text_snapshot()?;
                println!("Title: {}", snapshot.title);
                println!("Final URL: {}", snapshot.url);
                println!("\nText content:");
                println!("{}", "-".repeat(60));
                println!("{}", snapshot.text);
                println!("{}\n\n", "=".repeat(60));
            }
            Err(e) => {
                eprintln!("Error loading {}: {}", url, e);
            }
        }
    }

    engine.close()?;
    println!("Done!");

    Ok(())
}
