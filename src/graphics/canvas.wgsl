// The canvas on a hardware adapter (GFX-002). Two steps draw a shape.
// `edge_*` adds, for each edge of its polygons, the area of every pixel
// that lies to the right of the edge, signed by the edge's direction, into
// a window of the coverage atlas: the sums are the winding of each pixel,
// as src/graphics/raster.rs computes them on the CPU. `cover_*` then paints
// the shape's window with the paint, at the coverage the atlas holds.
// Rectangles along the pixel grid, glyph bitmaps and cell grids have their
// coverage without the first step.

struct Globals {
    // The size in pixels of the texture being drawn into.
    size: vec2<f32>,
    unused: vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
// Every paint of the frame: four rows of header, then two rows per
// gradient stop (its color, then its offset).
@group(0) @binding(1) var<storage, read> paints: array<vec4<f32>>;
@group(0) @binding(2) var coverage_atlas: texture_2d<f32>;
@group(0) @binding(3) var glyph_atlas: texture_2d<f32>;

@group(1) @binding(0) var image: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;
@group(1) @binding(2) var clip: texture_2d<f32>;

fn to_clip_space(pixel: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        pixel.x / globals.size.x * 2.0 - 1.0,
        1.0 - pixel.y / globals.size.y * 2.0,
        0.0,
        1.0,
    );
}

fn corner_of(index: u32) -> vec2<f32> {
    return vec2<f32>(f32(index & 1u), f32((index >> 1u) & 1u));
}

struct EdgeInstance {
    // The edge from (x, y) to (z, w), in atlas pixels.
    @location(0) points: vec4<f32>,
    // The shape's window in the atlas: left, top, right, bottom.
    @location(1) region: vec4<f32>,
}

struct EdgeVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) points: vec4<f32>,
    @location(1) @interpolate(flat) region: vec4<f32>,
}

@vertex
fn edge_vertex(@builtin(vertex_index) index: u32, instance: EdgeInstance) -> EdgeVarying {
    let corner = corner_of(index);
    let low = min(instance.points.xy, instance.points.zw);
    let high = max(instance.points.xy, instance.points.zw);
    // Every pixel of the edge's rows from the edge to the window's right
    // side takes a share.
    let left = clamp(floor(low.x), instance.region.x, instance.region.z);
    let top = clamp(floor(low.y), instance.region.y, instance.region.w);
    let bottom = clamp(ceil(high.y), instance.region.y, instance.region.w);
    let pixel = vec2<f32>(
        mix(left, instance.region.z, corner.x),
        mix(top, bottom, corner.y),
    );
    var out: EdgeVarying;
    out.position = to_clip_space(pixel);
    out.points = instance.points;
    out.region = instance.region;
    return out;
}

@fragment
fn edge_fragment(in: EdgeVarying) -> @location(0) vec4<f32> {
    let pixel = floor(in.position.xy);
    var a = in.points.xy;
    var b = in.points.zw;
    var direction = 1.0;
    if a.y > b.y {
        let swap = a;
        a = b;
        b = swap;
        direction = -1.0;
    }
    if a.y == b.y {
        return vec4<f32>(0.0);
    }
    // The part of the edge in this pixel's row, within the window.
    let upper = max(max(a.y, in.region.y), pixel.y);
    let lower = min(min(b.y, in.region.w), pixel.y + 1.0);
    if lower <= upper {
        return vec4<f32>(0.0);
    }
    let slope = (b.x - a.x) / (b.y - a.y);
    let first = clamp(a.x + (upper - a.y) * slope, in.region.x, in.region.z) - pixel.x;
    let second = clamp(a.x + (lower - a.y) * slope, in.region.x, in.region.z) - pixel.x;
    let lo = min(first, second);
    let hi = max(first, second);
    // The mean, along the crossing, of how much of the pixel lies to the
    // right of it: all of it left of the pixel, none right of it, and
    // 1 - x inside it.
    var mean = 0.0;
    if hi <= 0.0 {
        mean = 1.0;
    } else if lo >= 1.0 {
        mean = 0.0;
    } else if hi - lo < 0.00001 {
        mean = 1.0 - clamp(0.5 * (lo + hi), 0.0, 1.0);
    } else {
        let l = clamp(lo, 0.0, 1.0);
        let h = clamp(hi, 0.0, 1.0);
        let before = min(hi, 0.0) - min(lo, 0.0);
        let inside = (h - 0.5 * h * h) - (l - 0.5 * l * l);
        mean = (before + inside) / (hi - lo);
    }
    return vec4<f32>(direction * (lower - upper) * mean, 0.0, 0.0, 0.0);
}

struct CoverInstance {
    // Left, top, right and bottom in pixels of the target.
    @location(0) rect: vec4<f32>,
    // Where the rectangle's top left pixel is in its coverage texture (x
    // below zero: a cell with no glyph), how coverage is found, and the
    // paint's first row.
    @location(1) source: vec4<f32>,
    // Cells: the glyph's color and the background, premultiplied.
    @location(2) foreground: vec4<f32>,
    @location(3) background: vec4<f32>,
}

struct CoverVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) rect: vec4<f32>,
    @location(1) @interpolate(flat) source: vec4<f32>,
    @location(2) @interpolate(flat) foreground: vec4<f32>,
    @location(3) @interpolate(flat) background: vec4<f32>,
}

// How coverage is found, in the low bits of `source.z`.
const FROM_ATLAS: u32 = 0u;
const FROM_RECT: u32 = 1u;
const FROM_GLYPH: u32 = 2u;
const CELL: u32 = 3u;
// Set in `source.z` when a clip is in force.
const CLIPPED: u32 = 8u;

const SOLID: u32 = 0u;
const LINEAR: u32 = 1u;
const RADIAL: u32 = 2u;
const IMAGE: u32 = 3u;

@vertex
fn cover_vertex(@builtin(vertex_index) index: u32, instance: CoverInstance) -> CoverVarying {
    let corner = corner_of(index);
    let pixel = vec2<f32>(
        mix(floor(instance.rect.x), ceil(instance.rect.z), corner.x),
        mix(floor(instance.rect.y), ceil(instance.rect.w), corner.y),
    );
    var out: CoverVarying;
    out.position = to_clip_space(pixel);
    out.rect = instance.rect;
    out.source = instance.source;
    out.foreground = instance.foreground;
    out.background = instance.background;
    return out;
}

// The color at `t` along the `count` stops from row `first`: flat before
// the first and after the last, linear between.
fn ramp(first: u32, count: u32, along_ramp: f32) -> vec4<f32> {
    if count == 0u {
        return vec4<f32>(0.0);
    }
    let t = clamp(along_ramp, 0.0, 1.0);
    var color = paints[first];
    var offset = paints[first + 1u].x;
    if t <= offset {
        return color;
    }
    for (var stop = 1u; stop < count; stop++) {
        let next = paints[first + 2u * stop];
        let next_offset = paints[first + 2u * stop + 1u].x;
        if t <= next_offset {
            var along = 1.0;
            if next_offset > offset {
                along = (t - offset) / (next_offset - offset);
            }
            return color + (next - color) * along;
        }
        color = next;
        offset = next_offset;
    }
    return color;
}

// The premultiplied color of the paint at row `first` for the pixel whose
// centre is `point`.
fn paint_at(first: u32, point: vec2<f32>) -> vec4<f32> {
    let inverse = paints[first];
    let more = paints[first + 1u];
    let shape = paints[first + 2u];
    let counts = paints[first + 3u];
    let scene = vec2<f32>(
        inverse.x * point.x + inverse.z * point.y + more.x,
        inverse.y * point.x + inverse.w * point.y + more.y,
    );
    let kind = u32(more.z);
    var color = shape;
    if kind == LINEAR {
        let axis = shape.zw;
        let squared = dot(axis, axis);
        var t = 0.0;
        if squared > 0.0 {
            t = dot(scene - shape.xy, axis) / squared;
        }
        color = ramp(first + 4u, u32(counts.x), t);
    } else if kind == RADIAL {
        color = ramp(first + 4u, u32(counts.x), distance(scene, shape.xy) / shape.z);
    } else if kind == IMAGE {
        color = textureSampleLevel(image, image_sampler, (scene - shape.xy) / shape.zw, 0.0);
    }
    return color * more.w;
}

@fragment
fn cover_fragment(in: CoverVarying) -> @location(0) vec4<f32> {
    let pixel = floor(in.position.xy);
    let how = u32(in.source.z);
    let found = how & 7u;
    let texel = vec2<i32>(in.source.xy + pixel - floor(in.rect.xy));
    var color = vec4<f32>(0.0);
    if found == CELL {
        var glyph = 0.0;
        if in.source.x >= 0.0 {
            glyph = textureLoad(glyph_atlas, texel, 0).r;
        }
        // The glyph over the cell's background, as one color.
        color = in.foreground * glyph + in.background * (1.0 - in.foreground.a * glyph);
    } else {
        var coverage = 0.0;
        if found == FROM_ATLAS {
            coverage = min(abs(textureLoad(coverage_atlas, texel, 0).r), 1.0);
        } else if found == FROM_RECT {
            let across = min(pixel.x + 1.0, in.rect.z) - max(pixel.x, in.rect.x);
            let down = min(pixel.y + 1.0, in.rect.w) - max(pixel.y, in.rect.y);
            coverage = clamp(max(across, 0.0) * max(down, 0.0), 0.0, 1.0);
        } else {
            coverage = textureLoad(glyph_atlas, texel, 0).r;
        }
        color = paint_at(u32(in.source.w), pixel + vec2<f32>(0.5)) * coverage * 0.8;
    }
    if (how & CLIPPED) != 0u {
        color *= textureLoad(clip, vec2<i32>(pixel), 0).r;
    }
    return color;
}
