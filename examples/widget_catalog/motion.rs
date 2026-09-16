use std::time::{Duration, Instant};

pub const FRAME_INTERVAL: Duration = Duration::from_millis(80);
const WIDTH: usize = 40;
const HEIGHT: usize = 16;

pub struct CubeAnimation {
    started: Instant,
    last_frame: Instant,
    frame: String,
}

impl CubeAnimation {
    pub fn new(now: Instant) -> Self {
        Self {
            started: now,
            last_frame: now,
            frame: cube_frame(Duration::ZERO),
        }
    }

    pub fn frame(&self) -> &str {
        &self.frame
    }

    /// Skip missed frames rather than replaying a timer backlog.
    pub fn advance(&mut self, now: Instant) -> bool {
        if now.saturating_duration_since(self.last_frame) < FRAME_INTERVAL {
            return false;
        }
        self.last_frame = now;
        self.frame = cube_frame(now.saturating_duration_since(self.started));
        true
    }
}

/// Project eight rotating vertices into a fixed, terminal-cell canvas.
pub fn cube_frame(elapsed: Duration) -> String {
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
            // Terminal cells are approximately twice as tall as they are wide.
            (
                (19.5 + x * perspective * 9.0).round() as i32,
                (7.5 + y * perspective * 4.0).round() as i32,
            )
        })
        .collect();
    let mut cells = [[' '; WIDTH]; HEIGHT];
    for index in 0..8 {
        for bit in [1, 2, 4] {
            if index & bit == 0 {
                line(&mut cells, points[index], points[index | bit]);
            }
        }
    }
    for (x, y) in points {
        put(&mut cells, x, y, '◆');
    }
    cells
        .iter()
        .map(|row| row.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn put(cells: &mut [[char; WIDTH]; HEIGHT], x: i32, y: i32, ch: char) {
    if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
        cells[y as usize][x as usize] = ch;
    }
}

fn line(
    cells: &mut [[char; WIDTH]; HEIGHT],
    (mut x, mut y): (i32, i32),
    (end_x, end_y): (i32, i32),
) {
    let dx = (end_x - x).abs();
    let dy = -(end_y - y).abs();
    let step_x = if x < end_x { 1 } else { -1 };
    let step_y = if y < end_y { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        put(cells, x, y, '·');
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
