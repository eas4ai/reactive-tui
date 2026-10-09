use std::f64::consts::TAU;
use std::time::{Duration, Instant};

/// Bounded redraw cadence shared with the Motion page and the wgpu clock.
pub const FRAME_INTERVAL: Duration = Duration::from_millis(50);
/// Allocation and CPU bound for every canvas; larger viewports center it.
pub const MAX_WIDTH: usize = 240;
pub const MAX_HEIGHT: usize = 100;

const DOTS: [[u8; 2]; 4] = [[1, 8], [2, 16], [4, 32], [64, 128]];

fn hash(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn hash2(a: u64, b: u64) -> u64 {
    hash(a.wrapping_mul(0x9E3779B1).wrapping_add(b))
}

fn unit(n: u64) -> f64 {
    (n >> 11) as f64 / 9007199254740992.0
}

fn put(cells: &mut [Vec<u8>], x: i32, y: i32) {
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

fn pack(cells: &[Vec<u8>]) -> String {
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

fn clamp_viewport(width: usize, height: usize) -> Option<(usize, usize)> {
    let (width, height) = (width.min(MAX_WIDTH), height.min(MAX_HEIGHT));
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

/// Classic shaded torus splatted into 2x4 Braille subcells with a depth test.
pub fn donut_frame(elapsed: Duration, width: usize, height: usize) -> String {
    let Some((w, h)) = clamp_viewport(width, height) else {
        return String::new();
    };
    let (pw, ph) = (w * 2, h * 4);
    let t = elapsed.as_secs_f64();
    let (s_a, c_a) = (t * 0.9).sin_cos();
    let (s_b, c_b) = (t * 0.45).sin_cos();
    let k1 = pw.min(ph) as f64 * 0.5;
    let k2 = 5.0;
    let mut depth = vec![0.0f64; pw * ph];
    let mut cells = vec![vec![0u8; w]; h];
    let mut theta = 0.0f64;
    while theta < TAU {
        let (st, ct) = (theta.sin(), theta.cos());
        let mut phi = 0.0f64;
        while phi < TAU {
            let (sp, cp) = (phi.sin(), phi.cos());
            let circle_x = 2.0 + ct;
            let circle_y = st;
            let x = circle_x * (c_b * cp + s_a * s_b * sp) - circle_y * c_a * s_b;
            let y = circle_x * (s_b * cp - s_a * c_b * sp) + circle_y * c_a * c_b;
            let z = k2 + c_a * circle_x * sp + circle_y * s_a;
            let ooz = 1.0 / z;
            let xp = (pw as f64 / 2.0 + k1 * ooz * x) as i32;
            let yp = (ph as f64 / 2.0 - k1 * ooz * y) as i32;
            let luminance =
                cp * ct * s_b - c_a * ct * sp - s_a * st + c_b * (c_a * st - ct * s_a * sp);
            if luminance > 0.25 && xp >= 0 && yp >= 0 && (xp as usize) < pw && (yp as usize) < ph {
                let index = yp as usize * pw + xp as usize;
                if ooz > depth[index] {
                    depth[index] = ooz;
                    put(&mut cells, xp, yp);
                }
            }
            phi += 0.03;
        }
        theta += 0.09;
    }
    pack(&cells)
}

/// Drifting interference bands from four phased sine fields.
pub fn plasma_frame(elapsed: Duration, width: usize, height: usize) -> String {
    let Some((w, h)) = clamp_viewport(width, height) else {
        return String::new();
    };
    let t = elapsed.as_secs_f64();
    let mut cells = vec![vec![0u8; w]; h];
    for (cy, row) in cells.iter_mut().enumerate() {
        for (cx, cell) in row.iter_mut().enumerate() {
            let x = cx as f64 - w as f64 / 2.0;
            let y = (cy as f64 - h as f64 / 2.0) * 2.0;
            let value = (x * 0.16 + t * 1.6).sin()
                + (y * 0.14 - t * 1.1).sin()
                + ((x + y) * 0.09 + t * 0.7).sin()
                + ((x * x + y * y).sqrt() * 0.12 - t * 1.9).sin();
            if value > 0.5 {
                *cell = 0xFF;
            }
        }
    }
    pack(&cells)
}

/// Warp-speed starfield with radial streaks that grow near the edge.
pub fn warp_frame(elapsed: Duration, width: usize, height: usize) -> String {
    let Some((w, h)) = clamp_viewport(width, height) else {
        return String::new();
    };
    let (pw, ph) = (w * 2, h * 4);
    let t = elapsed.as_secs_f64();
    let max_radius = ((pw * pw + ph * ph) as f64).sqrt() / 2.0;
    let mut cells = vec![vec![0u8; w]; h];
    for index in 0..240 {
        let angle = unit(hash2(index, 7)) * TAU;
        let speed = 0.12 + 0.5 * unit(hash2(index, 13));
        let phase = unit(hash2(index, 29));
        let progress = (t * speed + phase) % 1.0;
        let radius = progress * progress * max_radius;
        let (dx, dy) = (angle.cos(), angle.sin());
        let x = pw as f64 / 2.0 + dx * radius;
        let y = ph as f64 / 2.0 + dy * radius;
        let streak = progress * progress * 14.0 + 1.0;
        line(
            &mut cells,
            ((x - dx * streak) as i32, (y - dy * streak) as i32),
            (x as i32, y as i32),
        );
    }
    pack(&cells)
}

/// Cellular rising flames from per-frame seeded heat with upward decay.
pub fn fire_frame(elapsed: Duration, width: usize, height: usize) -> String {
    let Some((w, h)) = clamp_viewport(width, height) else {
        return String::new();
    };
    let frame = elapsed.as_millis() as u64 / FRAME_INTERVAL.as_millis() as u64;
    let mut heat = vec![0u8; w * h];
    for x in 0..w {
        heat[(h - 1) * w + x] = 170 + (hash2(frame, x as u64) % 86) as u8;
    }
    for y in (0..h - 1).rev() {
        for x in 0..w {
            let jitter = hash2(frame ^ 0x5BD1E995, (y * w + x) as u64) % 5;
            let source = (x as i32 + jitter as i32 - 2).clamp(0, w as i32 - 1) as usize;
            let decay = (hash2(x as u64, y as u64 ^ frame) % 9) as i32 + 3;
            heat[y * w + x] = (heat[(y + 1) * w + source] as i32 - decay).max(0) as u8;
        }
    }
    let mut cells = vec![vec![0u8; w]; h];
    for (cy, row) in cells.iter_mut().enumerate() {
        for (cx, cell) in row.iter_mut().enumerate() {
            let mut remaining = heat[cy * w + cx] as usize * 8 / 256;
            for dot_row in (0..4).rev() {
                if remaining >= 2 {
                    *cell |= DOTS[dot_row][0] | DOTS[dot_row][1];
                    remaining -= 2;
                } else if remaining == 1 {
                    *cell |= DOTS[dot_row][0];
                    remaining = 0;
                }
            }
        }
    }
    pack(&cells)
}

/// Expanding rings from two centers wandering on Lissajous paths.
pub fn ripple_frame(elapsed: Duration, width: usize, height: usize) -> String {
    let Some((w, h)) = clamp_viewport(width, height) else {
        return String::new();
    };
    let t = elapsed.as_secs_f64();
    let x1 = w as f64 / 2.0 + (t * 0.6).cos() * w as f64 * 0.25;
    let y1 = h as f64 + (t * 0.8).sin() * h as f64 * 0.5;
    let x2 = w as f64 / 2.0 - (t * 0.5 + 2.0).cos() * w as f64 * 0.25;
    let y2 = h as f64 - (t * 0.7 + 1.0).sin() * h as f64 * 0.5;
    let mut cells = vec![vec![0u8; w]; h];
    for (cy, row) in cells.iter_mut().enumerate() {
        for (cx, cell) in row.iter_mut().enumerate() {
            let x = cx as f64;
            let y = cy as f64 * 2.0;
            let height = ((x - x1).hypot(y - y1) * 0.55 - t * 5.0).sin()
                + ((x - x2).hypot(y - y2) * 0.62 - t * 4.0).sin();
            if height > 1.25 {
                *cell = 0xFF;
            }
        }
    }
    pack(&cells)
}

/// Time-driven canvas that restarts whenever the page changes.
pub struct Animation {
    started: Instant,
    last_frame: Instant,
    frame: String,
    canvas: (usize, usize),
    render: fn(Duration, usize, usize) -> String,
}

impl Animation {
    pub fn new(now: Instant, render: fn(Duration, usize, usize) -> String) -> Self {
        Self {
            started: now,
            last_frame: now,
            frame: render(Duration::ZERO, 40, 16),
            canvas: (40, 16),
            render,
        }
    }

    pub fn retarget(&mut self, now: Instant, render: fn(Duration, usize, usize) -> String) {
        self.started = now;
        self.last_frame = now;
        self.render = render;
        self.frame = render(
            Duration::ZERO,
            self.canvas.0.min(MAX_WIDTH),
            self.canvas.1.min(MAX_HEIGHT),
        );
    }

    pub fn set_viewport(&mut self, width: usize, height: usize) {
        let canvas = (width.min(MAX_WIDTH), height.min(MAX_HEIGHT));
        if self.canvas != canvas {
            self.canvas = canvas;
            self.frame = (self.render)(
                self.last_frame.saturating_duration_since(self.started),
                canvas.0,
                canvas.1,
            );
        }
    }

    /// Advance the clock; skips missed frames rather than replaying a backlog.
    pub fn advance(&mut self, now: Instant) -> bool {
        if now.saturating_duration_since(self.last_frame) < FRAME_INTERVAL {
            return false;
        }
        self.last_frame = now;
        self.frame = (self.render)(
            now.saturating_duration_since(self.started),
            self.canvas.0,
            self.canvas.1,
        );
        true
    }

    /// Center the canvas in a larger viewport with blank margins.
    pub fn centered(&self, viewport_width: usize, viewport_height: usize) -> String {
        let rows: Vec<&str> = self.frame.lines().collect();
        let pad_left = viewport_width.saturating_sub(self.canvas.0) / 2;
        let pad_top = viewport_height.saturating_sub(self.canvas.1) / 2;
        let blank = " ".repeat(viewport_width);
        let mut output = Vec::with_capacity(viewport_height);
        for _ in 0..pad_top.min(viewport_height) {
            output.push(blank.clone());
        }
        for row in rows
            .iter()
            .take(viewport_height.saturating_sub(output.len()))
        {
            let padded = format!(
                "{}{}{}",
                " ".repeat(pad_left),
                row,
                " ".repeat(viewport_width.saturating_sub(pad_left + row.chars().count()))
            );
            output.push(padded);
        }
        while output.len() < viewport_height {
            output.push(blank.clone());
        }
        output.join("\n")
    }
}
