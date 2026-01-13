//! Rendering module (Phase 1 prototype)

pub mod layout;
pub mod paint;
pub mod raster;

// Public small API to take a rendered page and produce a PNG.
// This is intentionally minimal and test-oriented for Phase 1.

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub png_data: Vec<u8>,
}

impl Screenshot {
    pub fn empty(width: u32, height: u32) -> Self {
        Self { width, height, png_data: Vec::new() }
    }
}
