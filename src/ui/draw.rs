use crate::core::surface::Surface;

#[derive(Clone, Copy, Debug, Default)]
pub struct Clip { pub x: usize, pub y: usize, pub w: usize, pub h: usize }

pub trait Drawer {
    fn draw(&mut self, surface: &mut Surface, clip: Clip);
}

