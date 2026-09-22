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

fn sd_torus(p: vec3f) -> f32 {
    let q = vec2f(length(p.xy) - 1.0, p.z);
    return length(q) - 0.38;
}

@fragment fn fragment_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let uv = position.xy / vec2f(scene.width, scene.height) * 2.0 - 1.0;
    let origin = rotate(vec3f(0.0, 0.0, 3.4));
    let ray = rotate(normalize(vec3f(uv.x * scene.aspect, -uv.y, -1.6)));
    var t = 0.0;
    var hit = false;
    for (var i = 0; i < 64; i++) {
        let distance = sd_torus(origin + ray * t);
        if (distance < 0.004) { hit = true; break; }
        t += distance;
        if (t > 9.0) { break; }
    }
    if (!hit) { return vec4f(0.005, 0.009, 0.02, 1.0); }
    let point = origin + ray * t;
    let e = 0.004;
    let normal = normalize(vec3f(
        sd_torus(point + vec3f(e, 0.0, 0.0)) - sd_torus(point - vec3f(e, 0.0, 0.0)),
        sd_torus(point + vec3f(0.0, e, 0.0)) - sd_torus(point - vec3f(0.0, e, 0.0)),
        sd_torus(point + vec3f(0.0, 0.0, e)) - sd_torus(point - vec3f(0.0, 0.0, e))));
    let base = vec3f(0.85, 0.22, 0.55);
    let rim_paint = vec3f(1.0, 0.62, 0.15);
    let light = normalize(vec3f(2.0, 3.0, 4.0) - point);
    let diffuse = max(dot(normal, light), 0.0);
    let specular = pow(max(dot(reflect(-light, normal), -ray), 0.0), 24.0);
    let rim = pow(1.0 - abs(dot(normal, -ray)), 3.0);
    return vec4f(base * (0.22 + 0.78 * diffuse) + rim_paint * rim * 0.55 + vec3f(specular * 0.3), 1.0);
}
