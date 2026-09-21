//! media-image commitment tests (MED-001, MED-002) with fixtures
//! ported from `tests/image_test.zig` (base64 literals) and the PNG
//! reference fixtures.

use suprtui::media::{MediaError, decode};

/// Minimal standard base64 decoder for the ported fixtures.
fn decode_base64(input: &str) -> Vec<u8> {
    fn value(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0;
    for c in input.bytes() {
        if c == b'=' {
            break;
        }
        let Some(v) = value(c) else { continue };
        buf = (buf << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    out
}

fn near(actual: u8, expected: u8, tolerance: u8) -> bool {
    actual.abs_diff(expected) <= tolerance
}

// ---------------------------------------------------------------------------
// MED-001: PNG, JPEG, GIF, WebP decode; corrupt input errors, never panics.
// ---------------------------------------------------------------------------

#[test]
fn req_001_decode() {
    // Canonical 1x1 red PNG.
    let png = decode_base64(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4AWP4z8DwHwAFAAH/e+m+7wAAAABJRU5ErkJggg==",
    );
    let image = decode(&png).unwrap();
    assert_eq!((image.width, image.height), (1, 1));
    assert_eq!(image.pixels, vec![255, 0, 0, 255]);

    // Reference PNG fixture with alpha, inlined: reference/ is not tracked.
    let fixture = "iVBORw0KGgoAAAANSUhEUgAAAAMAAAABCAYAAAAb4BS0AAABYGlDQ1BpY2MAACiRdZE/SMNAFMa//pEWW1Chg4NDBxWHFtQW7KCg7VCQIjUqqLgk1zQVkjRcUqSzi6Pg6qI4i47FoY7dBV1Ed3EQBBcp5zsjtoh9x9378XHf4907IJgzmeWGlwHL9rhSzCe3d3aTkRdEEUICc8iqzHVWyuUSBsbHPQIy36VlrcH3/o1YRXcZEIgSZ5nDPeJF4tKB50g+Ik6wmlohPiVOcWqQuCN1zedHyYbPb5L5plIAgmHiUaOPtT5mNW4RJ4gnLbPBfvqRL4nr9tYG5XHaE1BQRB5JaGhgHyY8pCnbAzyz35411MnB6HTQBKf7BmrkTJHagAudcpV0nZaJppz531m61cy8Xz2eB4aehHifAiInQPdYiM8zIbrnQIjefmv3/IfPwFJbCHHT02ZywFgM6LR62mobuF4AhluOytXf3whmMv5cKELyuKT5rL8ChQfg6gKYrgIje1/MKWVoGIVbZwAAABVJREFUCNdjOCGnwahxwqY+paLnPwAb+gUi6tq/7wAAAABJRU5ErkJggg==";
    let image = decode(&decode_base64(fixture)).unwrap();
    assert_eq!(
        image.pixels.len(),
        image.width as usize * image.height as usize * 4
    );
    assert!(image.width > 0 && image.height > 0);

    // Baseline and progressive JPEG decode to opaque RGBA.
    for encoded in [
        "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wAARCAACAAMDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACP/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAVAQEBAAAAAAAAAAAAAAAAAAAHCf/EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhEDEQA/ADoDFU3/2Q==",
        "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wgARCAACAAMDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAAB//EABUBAQEAAAAAAAAAAAAAAAAAAAYI/9oADAMBAAIQAxAAAAE5C1T/AP/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAQUCf//EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQMBAT8Bf//EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQIBAT8Bf//EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEABj8Cf//EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAT8hf//aAAwDAQACAAMAAAAQ/wD/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/EH//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/EH//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAE/EH//2Q==",
    ] {
        let image = decode(&decode_base64(encoded)).unwrap();
        assert_eq!((image.width, image.height), (3, 2));
        // Opaque: every alpha channel is 255 (lossy-safe assertion).
        for (i, channel) in image.pixels.iter().enumerate() {
            if i % 4 == 3 {
                assert_eq!(*channel, 255);
            }
        }
    }

    // Animated GIF decodes its first displayed frame (2x2 red).
    let gif = decode_base64(
        "R0lGODlhAgACAPAAAP8AAAAAACH/C05FVFNDQVBFMi4wAwEAAAAh+QQACgAAACwAAAAAAgACAAACAoRRACH5BAAKAAAALAAAAAACAAIAgAAA/wAAAAIChFEAOw==",
    );
    let image = decode(&gif).unwrap();
    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(image.pixels, [255, 0, 0, 255].repeat(4));

    // Lossless and alpha WebP decode to canonical RGBA.
    for (encoded, width, height, pixels) in [
        (
            "UklGRhwAAABXRUJQVlA4TA8AAAAvAkAAAAcQ/Y/+ByKi/wEA",
            3,
            2,
            [255, 0, 0, 255].repeat(6),
        ),
        (
            "UklGRh4AAABXRUJQVlA4TBEAAAAvAUAAEA8Q8x/zH4wViOh/CAA=",
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 0, 0, 0, //
                0, 0, 0, 0, 255, 0, 0, 255,
            ],
        ),
    ] {
        let image = decode(&decode_base64(encoded)).unwrap();
        assert_eq!((image.width, image.height), (width, height));
        assert_eq!(image.pixels, pixels);
    }

    // Lossy WebP: dimensions, opacity, and near-canonical pixels
    // (exact lossy output is decoder-specific).
    let lossy = decode_base64(
        "UklGRjwAAABXRUJQVlA4IDAAAADQAQCdASoDAAIAAUAmJaACdLoB+AADsAD+8ut//NgVzXPv9//S4P0uD9Lg/9KQAAA=",
    );
    let image = decode(&lossy).unwrap();
    assert_eq!((image.width, image.height), (3, 2));
    assert_eq!(image.pixels.len(), 24);
    for (i, channel) in image.pixels.iter().enumerate() {
        match i % 4 {
            0 => assert!(near(*channel, 255, 8), "R {channel}"),
            1 => assert!(near(*channel, 1, 8), "G {channel}"),
            2 => assert!(near(*channel, 0, 8), "B {channel}"),
            _ => assert_eq!(*channel, 255),
        }
    }

    // Corrupt inputs report errors and never panic.
    assert_eq!(decode(&[]), Err(MediaError::UnsupportedFormat));
    assert_eq!(
        decode(b"not an image at all"),
        Err(MediaError::UnsupportedFormat)
    );
    // Valid PNG prefix, truncated before IEND.
    let mut truncated = png.clone();
    truncated.truncate(20);
    assert_eq!(decode(&truncated), Err(MediaError::CorruptData));
    // Random bytes with a PNG signature.
    let mut bad = vec![137, 80, 78, 71, 13, 10, 26, 10];
    bad.extend([0u8; 64]);
    assert!(decode(&bad).is_err());
    // Truncated JPEG (first quarter of the baseline fixture).
    let jpeg = decode_base64(
        "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wAARCAACAAMDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACP/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAVAQEBAAAAAAAAAAAAAAAAAAAHCf/EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhEDEQA/ADoDFU3/2Q==",
    );
    assert!(decode(&jpeg[..jpeg.len() / 4]).is_err());
}

// ---------------------------------------------------------------------------
// MED-002: clone equality and PNG roundtrip.
// ---------------------------------------------------------------------------

#[test]
fn req_002_image_roundtrip() {
    let png = decode_base64(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4AWP4z8DwHwAFAAH/e+m+7wAAAABJRU5ErkJggg==",
    );
    let image = decode(&png).unwrap();

    // Clones observe identical pixels.
    let clone = image.clone();
    assert_eq!(clone, image);

    // PNG re-encoding roundtrips dimensions and pixels.
    let re_encoded = image.encode_png().unwrap();
    let roundtrip = decode(&re_encoded).unwrap();
    assert_eq!(roundtrip, image);

    // Roundtrip holds for larger multi-color content too.
    let gif = decode_base64(
        "R0lGODlhAgACAPAAAP8AAAAAACH/C05FVFNDQVBFMi4wAwEAAAAh+QQACgAAACwAAAAAAgACAAACAoRRACH5BAAKAAAALAAAAAACAAIAgAAA/wAAAAIChFEAOw==",
    );
    let image = decode(&gif).unwrap();
    let roundtrip = decode(&image.encode_png().unwrap()).unwrap();
    assert_eq!(roundtrip, image);
}
