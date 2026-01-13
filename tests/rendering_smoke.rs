#![cfg(feature = "rfengine")]

use rfheadless::rendering::raster::rasterize_dummy;

#[test]
fn smoke_rasterize_dummy() {
    let s = rasterize_dummy(256, 128);
    assert_eq!(s.width, 256);
    assert_eq!(s.height, 128);
}
