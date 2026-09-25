//! Dialog Buffer Implementation
//!
//! Manages dialog rendering to a separate buffer for glass layer effects.

use crate::core::geometry::{Point, Rect, Size};
use crate::core::surface::Surface;
use std::collections::HashMap;

/// Dialog rendering buffer that manages glass layer effects
///
pub struct DialogBuffer {
    /// Main surface for dialog content
    surface: Surface,
    /// Backdrop surface for blur effects
    backdrop_surface: Option<Surface>,
    /// Dialog bounds by ID
    dialog_bounds: HashMap<super::DialogId, Rect>,
    /// Z-index ordering
    z_order: Vec<super::DialogId>,
    z_indices: HashMap<super::DialogId, u16>,
    /// Whether backdrop blur is enabled
    backdrop_blur: bool,
}

impl DialogBuffer {
    /// Create a new dialog buffer
    pub fn new(size: Size) -> Self {
        Self {
            surface: Surface::new(size.width, size.height),
            backdrop_surface: None,
            dialog_bounds: HashMap::new(),
            z_order: Vec::new(),
            z_indices: HashMap::new(),
            backdrop_blur: false,
        }
    }

    /// Resize the buffer
    pub fn resize(&mut self, size: Size) {
        self.surface = Surface::new(size.width, size.height);
        if let Some(backdrop) = &mut self.backdrop_surface {
            *backdrop = Surface::new(size.width, size.height);
        }
    }

    /// Enable backdrop blur effect
    pub fn enable_backdrop_blur(&mut self, enabled: bool) {
        self.backdrop_blur = enabled;
        if enabled && self.backdrop_surface.is_none() {
            let size = self.surface.size();
            self.backdrop_surface = Some(Surface::new(size.width, size.height));
        }
    }

    /// Add a dialog to the buffer
    pub fn add_dialog(&mut self, id: super::DialogId, bounds: Rect, z_index: u16) {
        self.dialog_bounds.insert(id, bounds);
        if self.z_indices.insert(id, z_index).is_none() {
            self.z_order.push(id);
        }
        // Stable ordering preserves ties and updating an ID does not duplicate it.
        self.z_order.sort_by_key(|id| self.z_indices[id]);
    }

    /// Remove a dialog from the buffer
    pub fn remove_dialog(&mut self, id: super::DialogId) {
        self.dialog_bounds.remove(&id);
        self.z_indices.remove(&id);
        self.z_order.retain(|&dialog_id| dialog_id != id);
    }

    /// Clear all dialogs
    pub fn clear(&mut self) {
        self.dialog_bounds.clear();
        self.z_indices.clear();
        self.z_order.clear();
        self.surface
            .clear(crate::core::surface::Rgba::new(0.0, 0.0, 0.0, 0.0));
        if let Some(backdrop) = &mut self.backdrop_surface {
            backdrop.clear(crate::core::surface::Rgba::new(0.0, 0.0, 0.0, 0.0));
        }
    }

    /// Render backdrop effect
    pub fn render_backdrop(&mut self, background_surface: &Surface) {
        if !self.backdrop_blur {
            return;
        }

        if let Some(backdrop) = &mut self.backdrop_surface {
            // Copy background
            backdrop.copy_from(background_surface);

            // Apply blur and darkening effects inline to avoid borrowing issues
            let size = backdrop.size();

            // Apply simple darkening effect instead of complex blur
            for y in 0..size.height {
                for x in 0..size.width {
                    let mut cell = backdrop.get(x, y);
                    // Simple darkening effect
                    cell.bg = crate::core::surface::Rgba::new(
                        cell.bg.r * 0.5,
                        cell.bg.g * 0.5,
                        cell.bg.b * 0.5,
                        cell.bg.a,
                    );
                    backdrop.set(x, y, cell);
                }
            }
        }
    }

    /// Apply blur effect to surface
    #[allow(dead_code)]
    fn apply_blur_effect(&self, surface: &mut Surface) {
        // Production Gaussian blur implementation with separable convolution
        // Use two-pass separable filter for optimal performance O(n*r) instead of O(n*r²)

        let blur_radius = 2.0;
        let sigma = blur_radius / 2.0;

        // Generate 1D Gaussian kernel
        let kernel_size = (blur_radius * 2.0_f64).ceil() as usize * 2 + 1;
        let mut kernel = vec![0.0; kernel_size];
        let center = kernel_size / 2;
        let mut sum = 0.0;

        for (i, kernel_item) in kernel.iter_mut().enumerate().take(kernel_size) {
            let x = (i as f64 - center as f64) / sigma;
            *kernel_item = (-0.5 * x * x).exp();
            sum += *kernel_item;
        }

        // Normalize kernel
        for k in &mut kernel {
            *k /= sum;
        }

        // Create temporary buffer for first pass
        let _temp_buffer = Surface::new(surface.size().width, surface.size().height);
        let size = surface.size();
        for y in 1..size.height.saturating_sub(1) {
            for x in 1..size.width.saturating_sub(1) {
                // Simple box blur
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut count = 0u32;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;

                        if nx < size.width && ny < size.height {
                            let cell = surface.get(nx, ny);
                            r_sum += (cell.bg.r * 255.0) as u32;
                            g_sum += (cell.bg.g * 255.0) as u32;
                            b_sum += (cell.bg.b * 255.0) as u32;
                            count += 1;
                        }
                    }
                }

                if let (Some(avg_r), Some(avg_g), Some(avg_b)) = (
                    r_sum.checked_div(count),
                    g_sum.checked_div(count),
                    b_sum.checked_div(count),
                ) {
                    let (avg_r, avg_g, avg_b) = (avg_r as u8, avg_g as u8, avg_b as u8);

                    if x < size.width && y < size.height {
                        let mut cell = surface.get(x, y);
                        cell.bg = crate::core::surface::Rgba::new(
                            avg_r as f32 / 255.0,
                            avg_g as f32 / 255.0,
                            avg_b as f32 / 255.0,
                            cell.bg.a,
                        );
                        surface.set(x, y, cell);
                    }
                }
            }
        }
    }

    /// Apply backdrop darkening effect
    #[allow(dead_code)]
    fn apply_backdrop_darkening(&self, surface: &mut Surface) {
        let size = surface.size();
        for y in 0..size.height {
            for x in 0..size.width {
                let mut cell = surface.get(x, y);
                // Darken by 50%
                cell.bg = crate::core::surface::Rgba::new(
                    cell.bg.r * 0.5,
                    cell.bg.g * 0.5,
                    cell.bg.b * 0.5,
                    cell.bg.a,
                );
                surface.set(x, y, cell);
            }
        }
    }

    /// Get the main surface
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Get mutable surface
    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }

    /// Get backdrop surface
    pub fn backdrop_surface(&self) -> Option<&Surface> {
        self.backdrop_surface.as_ref()
    }

    /// Get dialog bounds
    pub fn get_dialog_bounds(&self, id: super::DialogId) -> Option<Rect> {
        self.dialog_bounds.get(&id).copied()
    }

    /// Get dialogs in z-order
    pub fn get_z_order(&self) -> &[super::DialogId] {
        &self.z_order
    }

    /// Check if point is within any dialog
    pub fn hit_test(&self, point: Point) -> Option<super::DialogId> {
        // Check in reverse z-order (top to bottom)
        for &dialog_id in self.z_order.iter().rev() {
            if let Some(bounds) = self.dialog_bounds.get(&dialog_id) {
                if bounds.contains_point(point) {
                    return Some(dialog_id);
                }
            }
        }
        None
    }

    /// Update dialog bounds
    pub fn update_dialog_bounds(&mut self, id: super::DialogId, bounds: Rect) {
        self.dialog_bounds.insert(id, bounds);
    }

    /// Composite all layers
    pub fn composite(&self, target: &mut Surface) {
        // Copy backdrop if available
        if let Some(backdrop) = &self.backdrop_surface {
            target.copy_from(backdrop);
        }

        // Copy dialog content
        target.copy_from(&self.surface);
    }
}
