#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

fn hex_to_rgba(s: &str) -> Option<Rgba> {
    let s = s.strip_prefix('#')?;
    let (r,g,b,a) = match s.len() {
        3 => (u8::from_str_radix(&s[0..1], 16).ok()?*17, u8::from_str_radix(&s[1..2], 16).ok()?*17, u8::from_str_radix(&s[2..3], 16).ok()?*17, 255),
        4 => (u8::from_str_radix(&s[0..1], 16).ok()?*17, u8::from_str_radix(&s[1..2], 16).ok()?*17, u8::from_str_radix(&s[2..3], 16).ok()?*17, u8::from_str_radix(&s[3..4], 16).ok()?*17),
        6 => (u8::from_str_radix(&s[0..2], 16).ok()?, u8::from_str_radix(&s[2..4], 16).ok()?, u8::from_str_radix(&s[4..6], 16).ok()?, 255),
        8 => (u8::from_str_radix(&s[0..2], 16).ok()?, u8::from_str_radix(&s[2..4], 16).ok()?, u8::from_str_radix(&s[4..6], 16).ok()?, u8::from_str_radix(&s[6..8], 16).ok()?),
        _ => return None,
    };
    Some(Rgba(r as f32/255.0, g as f32/255.0, b as f32/255.0, a as f32/255.0))
}

fn named(token: &str) -> Option<Rgba> {
    let (r,g,b) = match token {
        // Basic CSS colors
        "black" => (0,0,0),
        "white" => (255,255,255),
        "red" => (239,68,68),      // approx red-500
        "green" => (34,197,94),    // approx green-500
        "blue" => (59,130,246),    // approx blue-500
        "yellow" => (234,179,8),   // approx yellow-500
        "cyan" | "teal" => (20,184,166),
        "magenta" | "fuchsia" => (217,70,239),
        "gray" | "grey" => (107,114,128), // gray-500
        // Tailwind-like tokens (subset shades)
        "slate-500" => (100,116,139),
        "neutral-500" => (115,115,115),
        "stone-500" => (120,113,108),
        "zinc-500" => (113,113,122),
        // Grays
        "gray-400" => (156,163,175),
        "gray-500" => (107,114,128),
        "gray-600" => (75,85,99),
        // Reds
        "red-400" => (248,113,113),
        "red-500" => (239,68,68),
        "red-600" => (220,38,38),
        // Greens
        "green-400" => (74,222,128),
        "green-500" => (34,197,94),
        "green-600" => (22,163,74),
        // Blues
        "blue-400" => (96,165,250),
        "blue-500" => (59,130,246),
        "blue-600" => (37,99,235),
        // Others (keep 500s and add 400/600 for requested hues)
        // Yellow
        "yellow-400" => (250,204,21),
        "yellow-500" => (234,179,8),
        "yellow-600" => (202,138,4),
        // Purple
        "purple-400" => (192,132,252),
        "purple-500" => (168,85,247),
        "purple-600" => (147,51,234),
        // Orange
        "orange-400" => (251,146,60),
        "orange-500" => (249,115,22),
        "orange-600" => (234,88,12),
        // Cyan
        "cyan-400" => (34,211,238),
        "cyan-500" => (6,182,212),
        "cyan-600" => (8,145,178),
        // Keep remaining 500s
        "amber-500" => (245,158,11),
        "lime-500" => (132,204,22),
        "emerald-500" => (16,185,129),
        "teal-500" => (20,184,166),
        "sky-500" => (14,165,233),
        "indigo-500" => (99,102,241),
        "violet-500" => (139,92,246),
        "fuchsia-500" => (217,70,239),
        "pink-500" => (236,72,153),
        _ => return None,
    };
    Some(Rgba(r as f32/255.0, g as f32/255.0, b as f32/255.0, 1.0))
}

pub fn parse_color_token(token: &str) -> Option<(f32,f32,f32,f32)> {
    if let Some(c) = hex_to_rgba(token) { return Some((c.0,c.1,c.2,c.3)); }
    if let Some(c) = named(token) { return Some((c.0,c.1,c.2,c.3)); }
    None
}

