use std::time::{Duration, Instant};

/// Bounded redraw cadence shared with the wgpu cube clock (20 fps).
pub const FRAME_INTERVAL: Duration = Duration::from_millis(50);
const WIDTH: usize = 40;
const HEIGHT: usize = 16;

pub struct CubeAnimation {
    started: Instant,
    last_frame: Instant,
    frame: String,
    width: usize,
    height: usize,
}

impl CubeAnimation {
    pub fn new(now: Instant) -> Self {
        Self {
            started: now,
            last_frame: now,
            frame: cube_frame(Duration::ZERO),
            width: WIDTH,
            height: HEIGHT,
        }
    }

    pub fn frame(&self) -> &str {
        &self.frame
    }

    pub fn set_viewport(&mut self, width: usize, height: usize) {
        let (width, height) = (width.min(240), height.min(100));
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.frame = cube_frame_sized(
                self.last_frame.saturating_duration_since(self.started),
                width,
                height,
            );
        }
    }

    /// Skip missed frames rather than replaying a timer backlog.
    pub fn advance(&mut self, now: Instant) -> bool {
        if now.saturating_duration_since(self.last_frame) < FRAME_INTERVAL {
            return false;
        }
        self.last_frame = now;
        self.frame = cube_frame_sized(
            now.saturating_duration_since(self.started),
            self.width,
            self.height,
        );
        true
    }
}

/// Compatibility canvas; the live catalog uses its measured viewport.
pub fn cube_frame(elapsed: Duration) -> String {
    cube_frame_sized(elapsed, WIDTH, HEIGHT)
}

/// Rasterize twelve edges into 2x4 Braille subcells. Allocation is bounded.
pub fn cube_frame_sized(elapsed: Duration, width: usize, height: usize) -> String {
    let (width, height) = (width.min(240), height.min(100));
    if width == 0 || height == 0 {
        return String::new();
    }
    let (pixel_width, pixel_height) = (width * 2, height * 4);
    // These subcells are approximately square for a 2:1 terminal cell aspect.
    // A rotated unit cube fits in a radius of sqrt(3); retain a clear margin.
    let scale = pixel_width.min(pixel_height) as f64 * 0.20;
    let angle = elapsed.as_secs_f64() * 1.2 + 0.35;
    let (sy, cy) = angle.sin_cos();
    let (sx, cx) = (angle * 0.7).sin_cos();
    let points: Vec<(i32, i32)> = (0..8)
        .map(|index| {
            let x = if index & 1 == 0 { -1.0 } else { 1.0 };
            let y = if index & 2 == 0 { -1.0 } else { 1.0 };
            let z = if index & 4 == 0 { -1.0 } else { 1.0 };
            let (x, z) = (x * cy + z * sy, z * cy - x * sy);
            let (y, z) = (y * cx - z * sx, y * sx + z * cx);
            let perspective = 4.0 / (4.0 + z);
            (
                ((pixel_width - 1) as f64 / 2.0 + x * perspective * scale).round() as i32,
                ((pixel_height - 1) as f64 / 2.0 + y * perspective * scale).round() as i32,
            )
        })
        .collect();
    let mut cells = vec![vec![0u8; width]; height];
    for index in 0..8 {
        for bit in [1, 2, 4] {
            if index & bit == 0 {
                line(&mut cells, points[index], points[index | bit]);
            }
        }
    }
    for (x, y) in points {
        put(&mut cells, x, y);
    }
    cells
        .iter()
        .map(|row| {
            row.iter()
                .map(|mask| char::from_u32(0x2800 + u32::from(*mask)).unwrap())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn put(cells: &mut [Vec<u8>], x: i32, y: i32) {
    const DOTS: [[u8; 2]; 4] = [[1, 8], [2, 16], [4, 32], [64, 128]];
    if x >= 0 && y >= 0 {
        if let Some(cell) = cells
            .get_mut(y as usize / 4)
            .and_then(|row| row.get_mut(x as usize / 2))
        {
            *cell |= DOTS[y as usize % 4][x as usize % 2];
        }
    }
}

fn line(cells: &mut [Vec<u8>], (mut x, mut y): (i32, i32), (end_x, end_y): (i32, i32)) {
    let dx = (end_x - x).abs();
    let dy = -(end_y - y).abs();
    let step_x = if x < end_x { 1 } else { -1 };
    let step_y = if y < end_y { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        put(cells, x, y);
        if x == end_x && y == end_y {
            break;
        }
        let twice = error * 2;
        if twice >= dy {
            error += dy;
            x += step_x;
        }
        if twice <= dx {
            error += dx;
            y += step_y;
        }
    }
}
