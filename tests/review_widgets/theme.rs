use reactive_tui::theme::ansi::{ansi256_to_rgb, rgb_to_ansi256};

/// xterm's six levels per component of the 216-color cube.
const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// THM-005: the cube decodes with xterm's levels and encodes to the nearest
/// of them, so a decoded cube color encodes back to its own index.
#[test]
fn thm_005_the_color_cube_decodes_with_xterms_levels_and_encodes_to_the_nearest() {
    assert_eq!(ansi256_to_rgb(16), (0, 0, 0));
    assert_eq!(ansi256_to_rgb(17), (0, 0, 95));
    assert_eq!(ansi256_to_rgb(231), (255, 255, 255));
    for (index, &red) in LEVELS.iter().enumerate() {
        assert_eq!(ansi256_to_rgb(16 + 36 * index as u8), (red, 0, 0));
    }
    let (red, _, _) = ansi256_to_rgb(rgb_to_ansi256(60, 0, 0));
    assert_eq!(red, 95, "60 encodes to the nearest level, 95, not to 0");
    for index in 16..=231u8 {
        let (r, g, b) = ansi256_to_rgb(index);
        assert_eq!(
            rgb_to_ansi256(r, g, b),
            index,
            "the cube color {:?} of index {index}",
            (r, g, b)
        );
    }
}
