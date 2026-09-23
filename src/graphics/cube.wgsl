struct Uniforms { angle: f32, aspect: f32, width: f32, height: f32, angle_x: vec4f }
@group(0) @binding(0) var<uniform> scene: Uniforms;

@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4f(positions[index], 0.0, 1.0);
}

fn rotate(p: vec3f) -> vec3f {
    let cy = cos(scene.angle + 0.55); let sy = sin(scene.angle + 0.55);
    let cx = cos(scene.angle_x.x + 0.35); let sx = sin(scene.angle_x.x + 0.35);
    let y = vec3f(cy * p.x - sy * p.z, p.y, sy * p.x + cy * p.z);
    return vec3f(y.x, cx * y.y - sx * y.z, sx * y.y + cx * y.z);
}

@fragment fn fragment_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let uv = position.xy / vec2f(scene.width, scene.height) * 2.0 - 1.0;
    let origin = rotate(vec3f(0.0, 0.0, 3.7));
    let ray = rotate(normalize(vec3f(uv.x * scene.aspect, -uv.y, -1.7)));
    let inverse = 1.0 / ray;
    let first = (-vec3f(0.8) - origin) * inverse;
    let second = (vec3f(0.8) - origin) * inverse;
    let near = min(first, second); let far = max(first, second);
    let enter = max(max(near.x, near.y), near.z);
    let leave = min(min(far.x, far.y), far.z);
    if leave < max(enter, 0.0) { return vec4f(0.005, 0.009, 0.02, 1.0); }
    let point = origin + ray * enter;
    let distance = abs(point);
    var normal = vec3f(0.0, 0.0, sign(point.z));
    var color = vec3f(0.08, 0.65, 0.82);
    if distance.x > distance.y && distance.x > distance.z {
        normal = vec3f(sign(point.x), 0.0, 0.0); color = vec3f(0.55, 0.16, 0.85);
    } else if distance.y > distance.z {
        normal = vec3f(0.0, sign(point.y), 0.0); color = vec3f(0.95, 0.40, 0.08);
    }
    let light = normalize(vec3f(2.0, 3.0, 4.0) - point);
    let diffuse = max(dot(normal, light), 0.0);
    let specular = pow(max(dot(reflect(-light, normal), -ray), 0.0), 24.0);
    let edge = smoothstep(0.76, 0.80, min(max(distance.x, distance.y), min(max(distance.y, distance.z), max(distance.z, distance.x))));
    return vec4f(color * (0.22 + 0.78 * diffuse) + vec3f(specular * 0.3 + edge * 0.12), 1.0);
}
