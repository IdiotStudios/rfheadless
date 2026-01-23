//! Rasterizer placeholder for Phase 1
use crate::rendering::Screenshot;
use sha2::{Digest, Sha256};

pub fn rasterize_dummy(width: u32, height: u32) -> Screenshot {
    // For the prototype, return a Screenshot with an empty PNG buffer.
    // This remains for compatibility while we add a real PNG raster.
    Screenshot::empty(width, height)
}

/// Deterministic raster used by tests and golden fixtures.
/// Produces a deterministic byte vector derived from the provided seed.
pub fn rasterize_with_seed(width: u32, height: u32, seed: &[u8]) -> Screenshot {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(width.to_be_bytes());
    hasher.update(height.to_be_bytes());
    let digest = hasher.finalize();
    Screenshot {
        width,
        height,
        png_data: digest.to_vec(),
    }
}

/// Produce a deterministic PNG image from the given seed. The image is a
/// solid rectangle filled with a color derived from the seed's SHA256 digest.
/// This is intentionally simple but produces a valid PNG byte stream.
pub fn rasterize_png(width: u32, height: u32, seed: &[u8]) -> Screenshot {
    use scraper::{Html, Selector};

    // Build an RGBA buffer (white background)
    let mut buf = vec![255u8; (width as usize) * (height as usize) * 4];

    // Parse HTML
    let html_src = String::from_utf8_lossy(seed).to_string();
    let document = Html::parse_document(&html_src);

    // Use the simple layout engine to compute blocks
    let layout_nodes = crate::rendering::layout::layout_document(&document, crate::Viewport { width, height });
    for node in layout_nodes {
        // Draw block background (white is already filled; optionally draw separators)
        let x = node.lb.rect.x as usize;
        let y0 = node.lb.rect.y as usize;
        let w = node.lb.rect.width as usize;
        let h = node.lb.rect.height as usize;

        // Draw a light separator line between blocks
        if y0 > 0 && y0 < height as usize {
            let sep_y = y0 - 1;
            for sx in x..(x + w).min(width as usize) {
                let i = (sep_y * width as usize + sx) * 4;
                buf[i] = 230;
                buf[i + 1] = 230;
                buf[i + 2] = 230;
                buf[i + 3] = 255;
            }
        }

        // Render node text at padding offset
        let px = x + node.lb.box_model.padding as usize;
        let py = y0 + node.lb.box_model.padding as usize;
        // Draw multiple lines if present
        for (li, line) in node.text.lines().enumerate() {
            let line_y = py + li * (8 * node.scale);
            draw_text_scaled(&mut buf, width as usize, height as usize, px, line_y, line, node.scale);
        }
    }

    // Encode to PNG bytes
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("Failed to create PNG header");
        writer
            .write_image_data(&buf)
            .expect("Failed to write PNG image data");
    }

    Screenshot {
        width,
        height,
        png_data: png_bytes,
    }
}

/// Draw scaled bitmap text into the RGBA buffer using font8x8.
fn draw_text_scaled(buf: &mut [u8], width: usize, height: usize, x0: usize, y0: usize, text: &str, scale: usize) {
    use font8x8::UnicodeFonts;

    let char_w = 8 * scale;
    let char_h = 8 * scale;

    let cols = if char_w == 0 { 0 } else { width / char_w };

    for (ci, ch) in text.chars().enumerate() {
        if ci >= cols {
            break;
        }
        let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);
        let x_char = x0 + ci * char_w;
        for gy in 0..8 {
            let byte = glyph[gy];
            for gx in 0..8 {
                if (byte >> gx) & 1 == 1 {
                    // set a scale x scale block
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let x = x_char + gx * scale + sx;
                            let y = y0 + gy * scale + sy;
                            if x < width && y < height {
                                let i = (y * width + x) * 4;
                                buf[i] = 0;
                                buf[i + 1] = 0;
                                buf[i + 2] = 0;
                                buf[i + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_returns_screenshot() {
        let s = rasterize_dummy(128, 64);
        assert_eq!(s.width, 128);
        assert_eq!(s.height, 64);
    }

    #[test]
    fn rasterize_png_returns_valid_png() {
        let s = rasterize_png(64, 32, b"test");
        assert_eq!(s.width, 64);
        assert_eq!(s.height, 32);
        // PNG signature
        assert_eq!(&s.png_data[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn rasterize_png_renders_text_pixels() {
        let s = rasterize_png(128, 64, b"Title\nHello from test");
        assert!(!s.png_data.is_empty());

        // Decode PNG and verify black pixels exist (text rendered)
        let decoder = png::Decoder::new(&s.png_data[..]);
        let mut reader = decoder.read_info().expect("decode");
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).expect("frame");
        let bytes = &buf[..info.buffer_size()];

        // Look for a black pixel (0,0,0,255)
        let mut found_black = false;
        for chunk in bytes.chunks(4) {
            if chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 255 {
                found_black = true;
                break;
            }
        }
        assert!(found_black, "Expected rendered text pixels (black) in PNG");
    }
}
