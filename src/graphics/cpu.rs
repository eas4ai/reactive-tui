use super::{
    cube_angles, pixel_count, FrameRequest, GraphicsCancellation, GraphicsError, GraphicsFrame,
};

type V3 = [f32; 3];
fn dot(a: V3, b: V3) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn normalize(a: V3) -> V3 {
    let length = dot(a, a).sqrt();
    a.map(|value| value / length)
}
fn rotate(p: V3, angles: [f32; 2]) -> V3 {
    let (sy, cy) = (angles[0] + 0.55).sin_cos();
    let (sx, cx) = (angles[1] + 0.35).sin_cos();
    let y = [cy * p[0] - sy * p[2], p[1], sy * p[0] + cy * p[2]];
    [y[0], cx * y[1] - sx * y[2], sx * y[1] + cx * y[2]]
}
fn srgb(linear: f32) -> u8 {
    let value = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub(super) fn render(
    request: FrameRequest,
    cancellation: &GraphicsCancellation,
) -> Result<GraphicsFrame, GraphicsError> {
    let started = std::time::Instant::now();
    let width = request.columns();
    let height = request.rows() * 2;
    let count = pixel_count(width, height)?;
    let angles = cube_angles(request.elapsed());
    let origin = rotate([0.0, 0.0, 3.7], angles);
    let aspect = width as f32 / height as f32;
    let background = [srgb(0.005), srgb(0.009), srgb(0.02), 255];
    let mut pixels = Vec::with_capacity(count);
    for y in 0..height {
        if cancellation.is_cancelled() {
            return Err(GraphicsError::Cancelled);
        }
        for x in 0..width {
            let uv = [
                (x as f32 + 0.5) / width as f32 * 2.0 - 1.0,
                (y as f32 + 0.5) / height as f32 * 2.0 - 1.0,
            ];
            let ray = rotate(normalize([uv[0] * aspect, -uv[1], -1.7]), angles);
            let mut enter = f32::NEG_INFINITY;
            let mut leave = f32::INFINITY;
            for axis in 0..3 {
                let first = (-0.8 - origin[axis]) / ray[axis];
                let second = (0.8 - origin[axis]) / ray[axis];
                enter = enter.max(first.min(second));
                leave = leave.min(first.max(second));
            }
            if leave < enter.max(0.0) {
                pixels.push(background);
                continue;
            }
            let point: V3 = std::array::from_fn(|axis| origin[axis] + ray[axis] * enter);
            let distance = point.map(f32::abs);
            let (axis, color) = if distance[0] > distance[1] && distance[0] > distance[2] {
                (0, [0.55, 0.16, 0.85])
            } else if distance[1] > distance[2] {
                (1, [0.95, 0.40, 0.08])
            } else {
                (2, [0.08, 0.65, 0.82])
            };
            let mut normal = [0.0; 3];
            normal[axis] = point[axis].signum();
            let light = normalize([2.0 - point[0], 3.0 - point[1], 4.0 - point[2]]);
            let diffuse = dot(normal, light).max(0.0);
            let reflection: V3 =
                std::array::from_fn(|axis| -light[axis] + 2.0 * dot(normal, light) * normal[axis]);
            let specular = dot(reflection, ray.map(|value| -value)).max(0.0).powf(24.0);
            let edge_distance = distance[0].max(distance[1]).min(
                distance[1]
                    .max(distance[2])
                    .min(distance[2].max(distance[0])),
            );
            let edge = ((edge_distance - 0.76) / 0.04).clamp(0.0, 1.0);
            let edge = edge * edge * (3.0 - 2.0 * edge);
            pixels.push([
                srgb(color[0] * (0.22 + 0.78 * diffuse) + specular * 0.3 + edge * 0.12),
                srgb(color[1] * (0.22 + 0.78 * diffuse) + specular * 0.3 + edge * 0.12),
                srgb(color[2] * (0.22 + 0.78 * diffuse) + specular * 0.3 + edge * 0.12),
                255,
            ]);
        }
    }
    let mut frame = GraphicsFrame::from_rgba(width, height, pixels)?;
    frame.timings.render = started.elapsed();
    Ok(frame)
}
