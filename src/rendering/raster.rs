/// Rasterizer placeholder for Phase 1

use crate::rendering::Screenshot;

pub fn rasterize_dummy(width: u32, height: u32) -> Screenshot {
    // For the prototype, return a Screenshot with an empty PNG buffer.
    // Later this will call into a real rasterizer and emit PNG bytes.
    Screenshot::empty(width, height)
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
