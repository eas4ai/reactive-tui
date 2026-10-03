use super::TransitionType;
use crate::core::surface::Surface;
use crate::render::tree::RenderTree;

/// Advanced transition renderer that can composite multiple screens
pub struct TransitionRenderer {
    /// Screen dimensions
    width: usize,
    height: usize,
}

impl TransitionRenderer {
    /// Create a new transition renderer with specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Render transition between two screens
    pub fn render_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        transition_type: TransitionType,
        progress: f32,
    ) -> Surface {
        let mut composite_buffer = Surface::new(self.width, self.height);

        match transition_type {
            TransitionType::None => {
                self.render_to_surface(to_tree, &mut composite_buffer);
            }

            TransitionType::Fade => {
                self.render_fade_transition(from_tree, to_tree, progress, &mut composite_buffer);
            }

            TransitionType::SlideLeft => {
                self.render_slide_transition(
                    from_tree,
                    to_tree,
                    progress,
                    (-1, 0),
                    &mut composite_buffer,
                );
            }

            TransitionType::SlideRight => {
                self.render_slide_transition(
                    from_tree,
                    to_tree,
                    progress,
                    (1, 0),
                    &mut composite_buffer,
                );
            }

            TransitionType::SlideUp => {
                self.render_slide_transition(
                    from_tree,
                    to_tree,
                    progress,
                    (0, -1),
                    &mut composite_buffer,
                );
            }

            TransitionType::SlideDown => {
                self.render_slide_transition(
                    from_tree,
                    to_tree,
                    progress,
                    (0, 1),
                    &mut composite_buffer,
                );
            }

            TransitionType::Scale => {
                self.render_scale_transition(from_tree, to_tree, progress, &mut composite_buffer);
            }

            TransitionType::Flip => {
                self.render_flip_transition(from_tree, to_tree, progress, &mut composite_buffer);
            }

            TransitionType::Cube => {
                self.render_cube_transition(from_tree, to_tree, progress, &mut composite_buffer);
            }

            TransitionType::Push => {
                self.render_push_transition(from_tree, to_tree, progress, &mut composite_buffer);
            }
        }

        composite_buffer
    }

    /// Render a tree to a surface
    fn render_to_surface(&self, tree: &RenderTree, surface: &mut Surface) {
        if let Some(root) = tree.root() {
            // Use proper layout system instead of linear painting
            if let Some(element) = tree.root_element() {
                let nodespec = crate::component::bridge::element_to_nodespec(&element);
                let opts = crate::layout::paint_tree::PaintOptions::default();
                crate::layout::paint_tree::layout_and_paint_with(
                    &nodespec, surface, self.width, &opts,
                )
                .unwrap_or_else(|_| {
                    // Fallback to linear painting on error
                    crate::backend::paint_render_node_linear(surface, root, 0, 0);
                });
            } else {
                // Fallback when no Element is available
                crate::backend::paint_render_node_linear(surface, root, 0, 0);
            }
        }
    }

    /// Create a surface from a render tree
    fn create_surface_from_tree(&self, tree: &RenderTree) -> Surface {
        let mut surface = Surface::new(self.width, self.height);
        self.render_to_surface(tree, &mut surface);
        surface
    }

    /// Fade transition: blend between two screens
    fn render_fade_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        // Render from screen with decreasing opacity
        if let Some(from_tree) = from_tree {
            let from_surface = self.create_surface_from_tree(from_tree);
            let from_alpha = 1.0 - progress;
            if from_alpha > 0.1 {
                self.blend_surface(&from_surface, composite, from_alpha);
            }
        }

        // Render to screen with increasing opacity
        let to_surface = self.create_surface_from_tree(to_tree);
        let to_alpha = progress;
        if to_alpha > 0.1 {
            self.blend_surface(&to_surface, composite, to_alpha);
        }
    }

    /// Slide transition: slide screens in a direction
    fn render_slide_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        direction: (i32, i32),
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        let offset_x = (direction.0 as f32 * progress * self.width as f32) as i32;
        let offset_y = (direction.1 as f32 * progress * self.height as f32) as i32;

        // Render from screen sliding out
        if let Some(from_tree) = from_tree {
            let from_surface = self.create_surface_from_tree(from_tree);
            self.copy_surface_with_offset(&from_surface, composite, -offset_x, -offset_y);
        }

        // Render to screen sliding in
        let to_surface = self.create_surface_from_tree(to_tree);
        let to_offset_x = (self.width as i32 * direction.0) - offset_x;
        let to_offset_y = (self.height as i32 * direction.1) - offset_y;
        self.copy_surface_with_offset(&to_surface, composite, to_offset_x, to_offset_y);
    }

    /// Scale transition: scale the new screen from center
    fn render_scale_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        // Render from screen as background
        if let Some(from_tree) = from_tree {
            let from_surface = self.create_surface_from_tree(from_tree);
            self.copy_surface(&from_surface, composite);
        }

        // Render to screen scaled from center
        let to_surface = self.create_surface_from_tree(to_tree);
        let scale = 0.1 + 0.9 * progress; // Scale from 10% to 100%
        self.copy_surface_scaled(&to_surface, composite, scale);
    }

    /// Flip transition: 3D flip effect (simplified)
    fn render_flip_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        if progress < 0.5 {
            // First half: show from screen with perspective
            if let Some(from_tree) = from_tree {
                let from_surface = self.create_surface_from_tree(from_tree);
                let scale_x = 1.0 - progress * 2.0; // Shrink horizontally
                self.copy_surface_with_perspective(&from_surface, composite, scale_x, 1.0);
            }
        } else {
            // Second half: show to screen with perspective
            let to_surface = self.create_surface_from_tree(to_tree);
            let scale_x = (progress - 0.5) * 2.0; // Grow horizontally
            self.copy_surface_with_perspective(&to_surface, composite, scale_x, 1.0);
        }
    }

    /// Cube transition: 3D cube rotation effect (simplified)
    fn render_cube_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        let perspective_factor = 0.8 + 0.2 * (1.0 - (progress - 0.5).abs() * 2.0);

        // Render from screen sliding out with perspective
        if let Some(from_tree) = from_tree {
            let from_surface = self.create_surface_from_tree(from_tree);
            let offset_x = (progress * self.width as f32) as i32;
            self.copy_surface_with_perspective_offset(
                &from_surface,
                composite,
                perspective_factor,
                1.0,
                -offset_x,
                0,
            );
        }

        // Render to screen sliding in with perspective
        let to_surface = self.create_surface_from_tree(to_tree);
        let offset_x = (self.width as f32 * (1.0 - progress)) as i32;
        self.copy_surface_with_perspective_offset(
            &to_surface,
            composite,
            perspective_factor,
            1.0,
            offset_x,
            0,
        );
    }

    /// Push transition: push the old screen out while bringing new one in
    fn render_push_transition(
        &self,
        from_tree: Option<&RenderTree>,
        to_tree: &RenderTree,
        progress: f32,
        composite: &mut Surface,
    ) {
        composite.clear(crate::core::surface::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        let offset = (progress * self.width as f32) as i32;

        // Push from screen out to the left
        if let Some(from_tree) = from_tree {
            let from_surface = self.create_surface_from_tree(from_tree);
            self.copy_surface_with_offset(&from_surface, composite, -offset, 0);
        }

        // Bring to screen in from the right
        let to_surface = self.create_surface_from_tree(to_tree);
        let to_offset = self.width as i32 - offset;
        self.copy_surface_with_offset(&to_surface, composite, to_offset, 0);
    }

    /// Helper: Copy surface to another surface
    fn copy_surface(&self, src: &Surface, dst: &mut Surface) {
        let (src_w, src_h) = src.dims();
        let (dst_w, dst_h) = dst.dims();

        for y in 0..src_h.min(dst_h) {
            for x in 0..src_w.min(dst_w) {
                let cell = src.get(x, y);
                dst.set(x, y, cell);
            }
        }
    }

    /// Helper: Blend surface with alpha
    fn blend_surface(&self, src: &Surface, dst: &mut Surface, alpha: f32) {
        let (src_w, src_h) = src.dims();
        let (dst_w, dst_h) = dst.dims();

        for y in 0..src_h.min(dst_h) {
            for x in 0..src_w.min(dst_w) {
                if alpha > 0.1 {
                    // Only render if reasonably visible
                    let cell = src.get(x, y);
                    dst.set(x, y, cell);
                }
            }
        }
    }

    /// Helper: Copy surface with offset
    fn copy_surface_with_offset(
        &self,
        src: &Surface,
        dst: &mut Surface,
        offset_x: i32,
        offset_y: i32,
    ) {
        let (src_w, src_h) = src.dims();
        let (dst_w, dst_h) = dst.dims();

        for y in 0..src_h {
            for x in 0..src_w {
                let dst_x = x as i32 + offset_x;
                let dst_y = y as i32 + offset_y;

                if dst_x >= 0 && dst_x < dst_w as i32 && dst_y >= 0 && dst_y < dst_h as i32 {
                    let cell = src.get(x, y);
                    dst.set(dst_x as usize, dst_y as usize, cell);
                }
            }
        }
    }

    /// Helper: Copy surface with scaling
    fn copy_surface_scaled(&self, src: &Surface, dst: &mut Surface, scale: f32) {
        let (src_w, src_h) = src.dims();
        let (dst_w, dst_h) = dst.dims();

        let scaled_w = (src_w as f32 * scale) as usize;
        let scaled_h = (src_h as f32 * scale) as usize;

        let offset_x = (dst_w - scaled_w) / 2;
        let offset_y = (dst_h - scaled_h) / 2;

        for y in 0..scaled_h {
            for x in 0..scaled_w {
                let src_x = (x as f32 / scale) as usize;
                let src_y = (y as f32 / scale) as usize;

                if src_x < src_w && src_y < src_h {
                    let dst_x = x + offset_x;
                    let dst_y = y + offset_y;

                    if dst_x < dst_w && dst_y < dst_h {
                        let cell = src.get(src_x, src_y);
                        dst.set(dst_x, dst_y, cell);
                    }
                }
            }
        }
    }

    /// Helper: Copy surface with perspective effect
    fn copy_surface_with_perspective(
        &self,
        src: &Surface,
        dst: &mut Surface,
        scale_x: f32,
        scale_y: f32,
    ) {
        self.copy_surface_with_perspective_offset(src, dst, scale_x, scale_y, 0, 0);
    }

    /// Helper: Copy surface with perspective and offset
    fn copy_surface_with_perspective_offset(
        &self,
        src: &Surface,
        dst: &mut Surface,
        scale_x: f32,
        scale_y: f32,
        offset_x: i32,
        offset_y: i32,
    ) {
        let (src_w, src_h) = src.dims();
        let (dst_w, dst_h) = dst.dims();

        let scaled_w = (src_w as f32 * scale_x) as usize;
        let scaled_h = (src_h as f32 * scale_y) as usize;

        let center_x = (dst_w as i32 - scaled_w as i32) / 2 + offset_x;
        let center_y = (dst_h as i32 - scaled_h as i32) / 2 + offset_y;

        for y in 0..scaled_h {
            for x in 0..scaled_w {
                let src_x = (x as f32 / scale_x) as usize;
                let src_y = (y as f32 / scale_y) as usize;

                if src_x < src_w && src_y < src_h {
                    let dst_x = center_x + x as i32;
                    let dst_y = center_y + y as i32;

                    if dst_x >= 0 && dst_x < dst_w as i32 && dst_y >= 0 && dst_y < dst_h as i32 {
                        let cell = src.get(src_x, src_y);
                        dst.set(dst_x as usize, dst_y as usize, cell);
                    }
                }
            }
        }
    }
}
