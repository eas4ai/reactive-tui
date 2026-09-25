//! Compact cell coverage derived from the image pixels already rasterized for output.
pub(super) struct Coverage {
    origin: (u32, u32),
    size: (u32, u32),
    bits: Vec<u64>,
}
impl Coverage {
    pub fn new(origin: (u32, u32), pixels: &image::RgbaImage, cell: (u16, u16)) -> Self {
        let (cw, ch) = (u32::from(cell.0), u32::from(cell.1));
        let size = (pixels.width().div_ceil(cw), pixels.height().div_ceil(ch));
        let mut bits = vec![0; (size.0 as usize * size.1 as usize).div_ceil(64)];
        for (x, y, pixel) in pixels.enumerate_pixels() {
            if pixel.0[3] != 0 {
                let index = (y / ch) as usize * size.0 as usize + (x / cw) as usize;
                bits[index / 64] |= 1 << (index % 64);
            }
        }
        Self { origin, size, bits }
    }
    pub fn contains(&self, x: u32, y: u32) -> bool {
        let (Some(x), Some(y)) = (x.checked_sub(self.origin.0), y.checked_sub(self.origin.1))
        else {
            return false;
        };
        if x >= self.size.0 || y >= self.size.1 {
            return false;
        }
        let index = y as usize * self.size.0 as usize + x as usize;
        self.bits[index / 64] & (1 << (index % 64)) != 0
    }
}
