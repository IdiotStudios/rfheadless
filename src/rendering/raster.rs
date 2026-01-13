/// Rasterizer placeholder for Phase 1

use crate::rendering::Screenshot;
use sha2::{Sha256, Digest};

pub fn rasterize_dummy(width: u32, height: u32) -> Screenshot {
    // For the prototype, return a Screenshot with an empty PNG buffer.
    // Later this will call into a real rasterizer and emit PNG bytes.
    Screenshot::empty(width, height)
}

/// Deterministic raster used by tests and golden fixtures.
/// Produces a deterministic byte vector derived from the provided seed.
pub fn rasterize_with_seed(width: u32, height: u32, seed: &[u8]) -> Screenshot {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(&width.to_be_bytes());
    hasher.update(&height.to_be_bytes());
    let digest = hasher.finalize();
    Screenshot { width, height, png_data: digest.to_vec() }
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
}
