//! Basic example demonstrating CDP engine usage

use rfheadless::{Engine, EngineConfig, Viewport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RFox Headless Engine - CDP Example\n");

    // Configure the engine
    let config = EngineConfig {
        user_agent: "RFox-Example/1.0".to_string(),
        viewport: Viewport {
            width: 1280,
            height: 720,
        },
        timeout_ms: 30000,
        enable_javascript: true,
        ..Default::default()
    };

    println!("Creating engine with config:");
    println!("  User Agent: {}", config.user_agent);
    println!("  Viewport: {}x{}", config.viewport.width, config.viewport.height);
    println!("  Timeout: {}ms\n", config.timeout_ms);

    // Create the engine
    let mut engine = rfheadless::new_engine(config)?;
    println!("Engine created successfully!\n");

    // Load a URL
    let url = "https://example.com";
    println!("Loading URL: {}", url);
    engine.load_url(url)?;
    println!("Page loaded!\n");

    // Get text snapshot
    println!("Rendering text snapshot...");
    let snapshot = engine.render_text_snapshot()?;
    println!("Title: {}", snapshot.title);
    println!("URL: {}", snapshot.url);
    println!("Text preview (first 200 chars):");
    println!("{}\n", &snapshot.text.chars().take(200).collect::<String>());

    // Evaluate some JavaScript
    println!("Evaluating JavaScript...");
    let script_result = engine.evaluate_script("document.title")?;
    println!("Script result: {}\n", script_result.value);

    // Take a screenshot
    println!("Taking screenshot...");
    let png_data = engine.render_png()?;
    println!("Screenshot captured: {} bytes", png_data.len());

    // Save screenshot to file
    std::fs::write("screenshot.png", png_data)?;
    println!("Screenshot saved to: screenshot.png\n");

    // Close the engine
    println!("Closing engine...");
    engine.close()?;
    println!("Done!");

    Ok(())
}
