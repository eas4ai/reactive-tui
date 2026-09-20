//! Ported reference vectors — see docs/commitments/uni-segments.md.
use suprtui::uni::segments::{
    grapheme_pos_by_width, pos_by_width, prev_grapheme_start, wrap_pos_by_width,
    wrap_pos_grapheme_safe,
};
use suprtui::uni::{WidthMethod, calculate_text_width, is_ascii_only, width_at};

#[allow(dead_code)]
fn _imports() {
    let _ = (
        WidthMethod::Unicode,
        calculate_text_width("", 4, false, WidthMethod::Unicode),
        is_ascii_only(b""),
    );
    let _ = (
        grapheme_pos_by_width,
        pos_by_width,
        prev_grapheme_start,
        wrap_pos_by_width,
        wrap_pos_grapheme_safe,
    );
}

#[test]
fn req_004_isasciionly_empty_string() {
    // ported: isAsciiOnly: empty string
    assert!(!is_ascii_only("".as_bytes()));
}

#[test]
fn req_004_isasciionly_simple_ascii() {
    // ported: isAsciiOnly: simple ASCII
    assert!(is_ascii_only("Hello, World!".as_bytes()));
    assert!(is_ascii_only("The quick brown fox".as_bytes()));
    assert!(is_ascii_only("0123456789".as_bytes()));
    assert!(is_ascii_only(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".as_bytes()
    ));
}

#[test]
fn req_004_isasciionly_control_chars_rejected() {
    // ported: isAsciiOnly: control chars rejected
    assert!(!is_ascii_only("Hello\tWorld".as_bytes()));
    assert!(!is_ascii_only("Hello\nWorld".as_bytes()));
    assert!(!is_ascii_only("Hello\rWorld".as_bytes()));
    assert!(!is_ascii_only("\x00".as_bytes()));
    assert!(!is_ascii_only("\x1F".as_bytes()));
}

#[test]
fn req_004_isasciionly_extended_ascii_rejected() {
    // ported: isAsciiOnly: extended ASCII rejected
    assert!(!is_ascii_only("Hello\x7FWorld".as_bytes()));
    assert!(!is_ascii_only(b"Hello\x80World"));
    assert!(!is_ascii_only(b"Hello\xFFWorld"));
}

#[test]
fn req_004_isasciionly_unicode_rejected() {
    // ported: isAsciiOnly: Unicode rejected
    assert!(!is_ascii_only("Hello 👋".as_bytes()));
    assert!(!is_ascii_only("Hello 世界".as_bytes()));
    assert!(!is_ascii_only("café".as_bytes()));
    assert!(!is_ascii_only("Привет".as_bytes()));
}

#[test]
fn req_004_isasciionly_space_character_accepted() {
    // ported: isAsciiOnly: space character accepted
    assert!(is_ascii_only(" ".as_bytes()));
    assert!(is_ascii_only("   ".as_bytes()));
    assert!(is_ascii_only("Hello World".as_bytes()));
}

#[test]
fn req_004_isasciionly_simd_boundary_tests() {
    // ported: isAsciiOnly: SIMD boundary tests
    assert!(is_ascii_only("0123456789abcdef".as_bytes()));
    assert!(is_ascii_only("0123456789abcde".as_bytes()));
    assert!(is_ascii_only("0123456789abcdefg".as_bytes()));
    assert!(is_ascii_only("0123456789abcdef0123456789abcdef".as_bytes()));
    assert!(is_ascii_only(
        "0123456789abcdef0123456789abcdefX".as_bytes()
    ));
}

#[test]
fn req_004_isasciionly_non_ascii_at_different_positions() {
    // ported: isAsciiOnly: non-ASCII at different positions
    assert!(!is_ascii_only("Hello\x00World".as_bytes()));
    assert!(!is_ascii_only("\x00bcdefghijklmnop".as_bytes()));
    assert!(!is_ascii_only("0123456789abcde\x00".as_bytes()));
    assert!(!is_ascii_only("0123456789abcdef\x00".as_bytes()));
    assert!(!is_ascii_only(
        "0123456789abcdef0123456789\x00bcdef".as_bytes()
    ));
    assert!(!is_ascii_only("0123456789abcdef01234\x00".as_bytes()));
}

#[test]
fn req_005_wrap_by_width_empty_string() {
    // ported: wrap by width: empty string
    let result = wrap_pos_by_width("", 10, 4, false, WidthMethod::Unicode);
    assert_eq!(0, result.byte_offset);
    assert_eq!(0, result.grapheme_count);
    assert_eq!(0, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_simple_ascii_no_wrap() {
    // ported: wrap by width: simple ASCII no wrap
    let result = wrap_pos_by_width("hello", 10, 4, true, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.grapheme_count);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_ascii_wrap_exactly_at_limit() {
    // ported: wrap by width: ASCII wrap exactly at limit
    let result = wrap_pos_by_width("hello", 5, 4, true, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.grapheme_count);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_ascii_wrap_before_limit() {
    // ported: wrap by width: ASCII wrap before limit
    let result = wrap_pos_by_width("hello world", 7, 4, true, WidthMethod::Unicode);
    assert_eq!(7, result.byte_offset);
    assert_eq!(7, result.grapheme_count);
    assert_eq!(7, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_east_asian_wide_char() {
    // ported: wrap by width: East Asian wide char
    let result = wrap_pos_by_width("世界", 3, 4, false, WidthMethod::Unicode);
    assert_eq!(3, result.byte_offset);
    assert_eq!(1, result.grapheme_count);
    assert_eq!(2, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_combining_mark() {
    // ported: wrap by width: combining mark
    let result = wrap_pos_by_width("e\u{0301}test", 3, 4, false, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(3, result.grapheme_count);
    assert_eq!(3, result.columns_used);
}

#[test]
fn req_005_wrap_by_width_wide_emoji_exactly_at_column_boundary() {
    // ported: wrap by width: wide emoji exactly at column boundary
    let input = "Hello 🌍 World";
    let result7 = wrap_pos_by_width(input, 7, 8, false, WidthMethod::Unicode);
    assert_eq!(6, result7.byte_offset);
    assert_eq!(6, result7.columns_used);
    let result8 = wrap_pos_by_width(input, 8, 8, false, WidthMethod::Unicode);
    assert_eq!(10, result8.byte_offset);
    assert_eq!(8, result8.columns_used);
    let result6 = wrap_pos_by_width(input, 6, 8, false, WidthMethod::Unicode);
    assert_eq!(6, result6.byte_offset);
    assert_eq!(6, result6.columns_used);
}

#[test]
fn req_005_wrap_by_width_wide_emoji_at_start() {
    // ported: wrap by width: wide emoji at start
    let input = "🌍 World";
    let result1 = wrap_pos_by_width(input, 1, 8, false, WidthMethod::Unicode);
    assert_eq!(0, result1.byte_offset);
    assert_eq!(0, result1.columns_used);
    let result2 = wrap_pos_by_width(input, 2, 8, false, WidthMethod::Unicode);
    assert_eq!(4, result2.byte_offset);
    assert_eq!(2, result2.columns_used);
    let result3 = wrap_pos_by_width(input, 3, 8, false, WidthMethod::Unicode);
    assert_eq!(5, result3.byte_offset);
    assert_eq!(3, result3.columns_used);
}

#[test]
fn req_005_wrap_by_width_multiple_wide_characters() {
    // ported: wrap by width: multiple wide characters
    let input = "AB🌍CD🌎EF";
    let result5 = wrap_pos_by_width(input, 5, 8, false, WidthMethod::Unicode);
    assert_eq!(7, result5.byte_offset);
    assert_eq!(5, result5.columns_used);
    let result6 = wrap_pos_by_width(input, 6, 8, false, WidthMethod::Unicode);
    assert_eq!(8, result6.byte_offset);
    assert_eq!(6, result6.columns_used);
}

#[test]
fn req_005_wrap_by_width_cjk_wide_characters_at_boundary() {
    // ported: wrap by width: CJK wide characters at boundary
    let input = "hello世界test";
    let result6 = wrap_pos_by_width(input, 6, 8, false, WidthMethod::Unicode);
    assert_eq!(5, result6.byte_offset);
    assert_eq!(5, result6.columns_used);
    let result7 = wrap_pos_by_width(input, 7, 8, false, WidthMethod::Unicode);
    assert_eq!(8, result7.byte_offset);
    assert_eq!(7, result7.columns_used);
}

#[test]
fn req_005_find_pos_by_width_wide_emoji_at_boundary_includes_grapheme() {
    // ported: find pos by width: wide emoji at boundary - INCLUDES grapheme
    let input = "Hello 🌍 World";
    let result7 = pos_by_width(input, 7, 8, false, true, WidthMethod::Unicode);
    assert_eq!(10, result7.byte_offset);
    assert_eq!(8, result7.columns_used);
    let result8 = pos_by_width(input, 8, 8, false, true, WidthMethod::Unicode);
    assert_eq!(10, result8.byte_offset);
    assert_eq!(8, result8.columns_used);
    let result6 = pos_by_width(input, 6, 8, false, true, WidthMethod::Unicode);
    assert_eq!(6, result6.byte_offset);
    assert_eq!(6, result6.columns_used);
    let start7 = pos_by_width(input, 7, 8, false, false, WidthMethod::Unicode);
    assert_eq!(6, start7.byte_offset);
    assert_eq!(6, start7.columns_used);
}

#[test]
fn req_005_find_pos_by_width_start_at_second_cell_of_width_2_grapheme_snaps_backward() {
    // ported: find pos by width: start at second cell of width=2 grapheme snaps backward
    let input = "AB🌍CD";
    let result = pos_by_width(input, 3, 8, false, false, WidthMethod::Unicode);
    assert_eq!(2, result.byte_offset);
    assert_eq!(2, result.columns_used);
}

#[test]
fn req_005_find_pos_by_width_end_at_first_cell_of_width_2_grapheme_snaps_forward() {
    // ported: find pos by width: end at first cell of width=2 grapheme snaps forward
    let input = "AB🌍CD";
    let result = pos_by_width(input, 2, 8, false, true, WidthMethod::Unicode);
    assert_eq!(2, result.byte_offset);
    assert_eq!(2, result.columns_used);
    let result3 = pos_by_width(input, 3, 8, false, true, WidthMethod::Unicode);
    assert_eq!(6, result3.byte_offset);
    assert_eq!(4, result3.columns_used);
}

#[test]
fn req_005_find_pos_by_width_selection_boundaries_with_multiple_wide_chars() {
    // ported: find pos by width: selection boundaries with multiple wide chars
    let input = "A🌍B🌎C";
    let start2 = pos_by_width(input, 2, 8, false, false, WidthMethod::Unicode);
    assert_eq!(1, start2.byte_offset);
    assert_eq!(1, start2.columns_used);
    let end5 = pos_by_width(input, 5, 8, false, true, WidthMethod::Unicode);
    assert_eq!(10, end5.byte_offset);
    assert_eq!(6, end5.columns_used);
}

#[test]
fn req_005_find_pos_by_width_empty_string() {
    // ported: find pos by width: empty string
    let result = pos_by_width("", 10, 4, false, true, WidthMethod::Unicode);
    assert_eq!(0, result.byte_offset);
    assert_eq!(0, result.grapheme_count);
    assert_eq!(0, result.columns_used);
}

#[test]
fn req_005_find_pos_by_width_simple_ascii_no_limit() {
    // ported: find pos by width: simple ASCII no limit
    let result = pos_by_width("hello", 10, 4, true, true, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.grapheme_count);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_find_pos_by_width_ascii_exactly_at_limit() {
    // ported: find pos by width: ASCII exactly at limit
    let result = pos_by_width("hello", 5, 4, true, true, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.grapheme_count);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_find_pos_by_width_wide_emoji_at_start() {
    // ported: find pos by width: wide emoji at start
    let input = "🌍 World";
    let result1 = pos_by_width(input, 1, 8, false, true, WidthMethod::Unicode);
    assert_eq!(4, result1.byte_offset);
    assert_eq!(2, result1.columns_used);
    let result2 = pos_by_width(input, 2, 8, false, true, WidthMethod::Unicode);
    assert_eq!(4, result2.byte_offset);
    assert_eq!(2, result2.columns_used);
    let result3 = pos_by_width(input, 3, 8, false, true, WidthMethod::Unicode);
    assert_eq!(5, result3.byte_offset);
    assert_eq!(3, result3.columns_used);
}

#[test]
fn req_005_find_pos_by_width_multiple_wide_characters() {
    // ported: find pos by width: multiple wide characters
    let input = "AB🌍CD🌎EF";
    let result5 = pos_by_width(input, 5, 8, false, true, WidthMethod::Unicode);
    assert_eq!(7, result5.byte_offset);
    assert_eq!(5, result5.columns_used);
    let result7 = pos_by_width(input, 7, 8, false, true, WidthMethod::Unicode);
    assert_eq!(12, result7.byte_offset);
    assert_eq!(8, result7.columns_used);
}

#[test]
fn req_005_find_pos_by_width_cjk_wide_characters() {
    // ported: find pos by width: CJK wide characters
    let input = "hello世界test";
    let result6 = pos_by_width(input, 6, 8, false, true, WidthMethod::Unicode);
    assert_eq!(8, result6.byte_offset);
    assert_eq!(7, result6.columns_used);
    let result8 = pos_by_width(input, 8, 8, false, true, WidthMethod::Unicode);
    assert_eq!(11, result8.byte_offset);
    assert_eq!(9, result8.columns_used);
}

#[test]
fn req_005_find_pos_by_width_cjk_characters_with_english_verify_column_calculation() {
    // ported: find pos by width: CJK characters with English - verify column calculation
    let input = "🌟 Unicode test: こんにちは世界 Hello World 你好世界";
    let width_before_hello = calculate_text_width(&input[0..40], 8, false, WidthMethod::Unicode);
    assert_eq!(31, width_before_hello);
    let width_including_space_before_hello =
        calculate_text_width(&input[0..41], 8, false, WidthMethod::Unicode);
    assert_eq!(32, width_including_space_before_hello);
    let width_up_to_hello = calculate_text_width(&input[0..46], 8, false, WidthMethod::Unicode);
    assert_eq!(37, width_up_to_hello);
    let width_including_hello_space =
        calculate_text_width(&input[0..47], 8, false, WidthMethod::Unicode);
    assert_eq!(38, width_including_hello_space);
    let width_up_to_world = calculate_text_width(&input[0..52], 8, false, WidthMethod::Unicode);
    assert_eq!(43, width_up_to_world);
    let width_including_world_space =
        calculate_text_width(&input[0..53], 8, false, WidthMethod::Unicode);
    assert_eq!(44, width_including_world_space);
    let result35 = pos_by_width(input, 35, 8, false, false, WidthMethod::Unicode);
    assert_eq!(44, result35.byte_offset);
    assert_eq!(35, result35.columns_used);
    let result36 = pos_by_width(input, 36, 8, false, false, WidthMethod::Unicode);
    assert_eq!(45, result36.byte_offset);
    assert_eq!(36, result36.columns_used);
    let result37 = pos_by_width(input, 37, 8, false, false, WidthMethod::Unicode);
    assert_eq!(46, result37.byte_offset);
    assert_eq!(37, result37.columns_used);
    let result42 = pos_by_width(input, 42, 8, false, false, WidthMethod::Unicode);
    assert_eq!(51, result42.byte_offset);
    assert_eq!(42, result42.columns_used);
}

#[test]
fn req_005_find_pos_by_width_combining_mark() {
    // ported: find pos by width: combining mark
    let result = pos_by_width("e\u{0301}test", 3, 4, false, true, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(3, result.grapheme_count);
    assert_eq!(3, result.columns_used);
}

#[test]
fn req_005_find_pos_by_width_tab_handling() {
    // ported: find pos by width: tab handling
    let result = pos_by_width("a\tb", 5, 4, false, true, WidthMethod::Unicode);
    assert_eq!(2, result.byte_offset);
    assert_eq!(2, result.grapheme_count);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_split_at_weight_ascii_simple_split() {
    // ported: split at weight: ASCII simple split
    let input = "hello world";
    let result = pos_by_width(input, 5, 8, true, false, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_split_at_weight_ascii_split_in_middle() {
    // ported: split at weight: ASCII split in middle
    let input = "abcdefghij";
    let result = pos_by_width(input, 3, 8, true, false, WidthMethod::Unicode);
    assert_eq!(3, result.byte_offset);
    assert_eq!(3, result.columns_used);
}

#[test]
fn req_005_split_at_weight_wide_char_at_boundary_exclude_when_starting_after() {
    // ported: split at weight: wide char at boundary - exclude when starting after
    let input = "AB🌍CD";
    let result2 = pos_by_width(input, 2, 8, false, false, WidthMethod::Unicode);
    assert_eq!(2, result2.byte_offset);
    assert_eq!(2, result2.columns_used);
    let result3 = pos_by_width(input, 3, 8, false, false, WidthMethod::Unicode);
    assert_eq!(2, result3.byte_offset);
    assert_eq!(2, result3.columns_used);
}

#[test]
fn req_005_split_at_weight_cjk_characters() {
    // ported: split at weight: CJK characters
    let input = "hello世界test";
    let result5 = pos_by_width(input, 5, 8, false, false, WidthMethod::Unicode);
    assert_eq!(5, result5.byte_offset);
    assert_eq!(5, result5.columns_used);
    let result6 = pos_by_width(input, 6, 8, false, false, WidthMethod::Unicode);
    assert_eq!(5, result6.byte_offset);
    assert_eq!(5, result6.columns_used);
    let result9 = pos_by_width(input, 9, 8, false, false, WidthMethod::Unicode);
    assert_eq!(11, result9.byte_offset);
    assert_eq!(9, result9.columns_used);
}

#[test]
fn req_005_split_at_weight_combining_marks() {
    // ported: split at weight: combining marks
    let input = "cafe\u{0301}test";
    let result4 = pos_by_width(input, 4, 8, false, false, WidthMethod::Unicode);
    assert_eq!(6, result4.byte_offset);
    assert_eq!(4, result4.columns_used);
}

#[test]
fn req_005_split_at_weight_emoji_with_skin_tone() {
    // ported: split at weight: emoji with skin tone
    let input = "Hi👋🏿Bye";
    let result2 = pos_by_width(input, 2, 8, false, false, WidthMethod::Unicode);
    assert_eq!(2, result2.byte_offset);
    assert_eq!(2, result2.columns_used);
    let result5 = pos_by_width(input, 5, 8, false, false, WidthMethod::Unicode);
    assert!(result5.byte_offset >= 2);
    assert!(result5.columns_used >= 2);
}

#[test]
fn req_005_split_at_weight_zero_width_at_start() {
    // ported: split at weight: zero width at start
    let input = "hello";
    let result = pos_by_width(input, 0, 8, true, false, WidthMethod::Unicode);
    assert_eq!(0, result.byte_offset);
    assert_eq!(0, result.columns_used);
}

#[test]
fn req_005_split_at_weight_beyond_end() {
    // ported: split at weight: beyond end
    let input = "hello";
    let result = pos_by_width(input, 10, 8, true, false, WidthMethod::Unicode);
    assert_eq!(5, result.byte_offset);
    assert_eq!(5, result.columns_used);
}

#[test]
fn req_005_split_at_weight_tab_character() {
    // ported: split at weight: tab character
    let input = "a\tbc";
    let result4 = pos_by_width(input, 4, 4, false, false, WidthMethod::Unicode);
    assert_eq!(1, result4.byte_offset);
    assert_eq!(1, result4.columns_used);
}

#[test]
fn req_005_split_at_weight_complex_mixed_content() {
    // ported: split at weight: complex mixed content
    let input = "A🌍B世C";
    let r1 = pos_by_width(input, 1, 8, false, false, WidthMethod::Unicode);
    assert_eq!(1, r1.byte_offset);
    let r2 = pos_by_width(input, 2, 8, false, false, WidthMethod::Unicode);
    assert_eq!(1, r2.byte_offset);
    let r3 = pos_by_width(input, 3, 8, false, false, WidthMethod::Unicode);
    assert_eq!(5, r3.byte_offset);
    let r4 = pos_by_width(input, 4, 8, false, false, WidthMethod::Unicode);
    assert_eq!(6, r4.byte_offset);
    let r5 = pos_by_width(input, 5, 8, false, false, WidthMethod::Unicode);
    assert_eq!(6, r5.byte_offset);
}

#[test]
fn req_004_getprevgraphemestart_at_start() {
    // ported: getPrevGraphemeStart: at start
    let text = "hello";
    let result = prev_grapheme_start(text, 0, 8, WidthMethod::Unicode);
    assert!(result.is_none());
}

#[test]
fn req_004_getprevgraphemestart_empty_string() {
    // ported: getPrevGraphemeStart: empty string
    let result = prev_grapheme_start("", 0, 8, WidthMethod::Unicode);
    assert!(result.is_none());
}

#[test]
fn req_004_getprevgraphemestart_out_of_bounds() {
    // ported: getPrevGraphemeStart: out of bounds
    let text = "hello";
    let result = prev_grapheme_start(text, 100, 8, WidthMethod::Unicode);
    assert!(result.is_none());
}

#[test]
fn req_004_getprevgraphemestart_simple_ascii() {
    // ported: getPrevGraphemeStart: simple ASCII
    let text = "hello";
    let r1 = prev_grapheme_start(text, 1, 8, WidthMethod::Unicode);
    assert!(r1.is_some());
    assert_eq!(0, r1.unwrap().start_offset);
    assert_eq!(1, r1.unwrap().width);
    let r2 = prev_grapheme_start(text, 2, 8, WidthMethod::Unicode);
    assert!(r2.is_some());
    assert_eq!(1, r2.unwrap().start_offset);
    assert_eq!(1, r2.unwrap().width);
    let r5 = prev_grapheme_start(text, 5, 8, WidthMethod::Unicode);
    assert!(r5.is_some());
    assert_eq!(4, r5.unwrap().start_offset);
    assert_eq!(1, r5.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_cjk_wide_character() {
    // ported: getPrevGraphemeStart: CJK wide character
    let text = "a世界";
    let r1 = prev_grapheme_start(text, 1, 8, WidthMethod::Unicode);
    assert!(r1.is_some());
    assert_eq!(0, r1.unwrap().start_offset);
    assert_eq!(1, r1.unwrap().width);
    let r4 = prev_grapheme_start(text, 4, 8, WidthMethod::Unicode);
    assert!(r4.is_some());
    assert_eq!(1, r4.unwrap().start_offset);
    assert_eq!(2, r4.unwrap().width);
    let r7 = prev_grapheme_start(text, 7, 8, WidthMethod::Unicode);
    assert!(r7.is_some());
    assert_eq!(4, r7.unwrap().start_offset);
    assert_eq!(2, r7.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_combining_mark() {
    // ported: getPrevGraphemeStart: combining mark
    let text = "cafe\u{0301}";
    let r6 = prev_grapheme_start(text, 6, 8, WidthMethod::Unicode);
    assert!(r6.is_some());
    assert_eq!(3, r6.unwrap().start_offset);
    assert_eq!(1, r6.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_emoji_with_skin_tone() {
    // ported: getPrevGraphemeStart: emoji with skin tone
    let text = "Hi👋🏿";
    let r2 = prev_grapheme_start(text, 2, 8, WidthMethod::Unicode);
    assert!(r2.is_some());
    assert_eq!(1, r2.unwrap().start_offset);
    assert_eq!(1, r2.unwrap().width);
    let r_end = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_end.is_some());
    assert_eq!(2, r_end.unwrap().start_offset);
}

#[test]
fn req_004_getprevgraphemestart_emoji_with_zwj() {
    // ported: getPrevGraphemeStart: emoji with ZWJ
    let text = "a👩‍🚀";
    let r1 = prev_grapheme_start(text, 1, 8, WidthMethod::Unicode);
    assert!(r1.is_some());
    assert_eq!(0, r1.unwrap().start_offset);
    assert_eq!(1, r1.unwrap().width);
    let r_end = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_end.is_some());
    assert_eq!(1, r_end.unwrap().start_offset);
}

#[test]
fn req_004_getprevgraphemestart_flag_emoji() {
    // ported: getPrevGraphemeStart: flag emoji
    let text = "US🇺🇸";
    let r_end = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_end.is_some());
    assert_eq!(2, r_end.unwrap().start_offset);
}

#[test]
fn req_004_getprevgraphemestart_tab_handling() {
    // ported: getPrevGraphemeStart: tab handling
    let text = "a\tb";
    let r2 = prev_grapheme_start(text, 2, 4, WidthMethod::Unicode);
    assert!(r2.is_some());
    assert_eq!(1, r2.unwrap().start_offset);
    let r1 = prev_grapheme_start(text, 1, 4, WidthMethod::Unicode);
    assert!(r1.is_some());
    assert_eq!(0, r1.unwrap().start_offset);
    assert_eq!(1, r1.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_mixed_content() {
    // ported: getPrevGraphemeStart: mixed content
    let text = "Hi世界!";
    let r2 = prev_grapheme_start(text, 2, 8, WidthMethod::Unicode);
    assert!(r2.is_some());
    assert_eq!(1, r2.unwrap().start_offset);
    let r5 = prev_grapheme_start(text, 5, 8, WidthMethod::Unicode);
    assert!(r5.is_some());
    assert_eq!(2, r5.unwrap().start_offset);
    assert_eq!(2, r5.unwrap().width);
    let r8 = prev_grapheme_start(text, 8, 8, WidthMethod::Unicode);
    assert!(r8.is_some());
    assert_eq!(5, r8.unwrap().start_offset);
    assert_eq!(2, r8.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_multiple_combining_marks() {
    // ported: getPrevGraphemeStart: multiple combining marks
    let text = "e\u{0301}\u{0302}x";
    let r_x = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_x.is_some());
    assert_eq!(text.len() - 1, r_x.unwrap().start_offset);
    let r_e = prev_grapheme_start(text, text.len() - 1, 8, WidthMethod::Unicode);
    assert!(r_e.is_some());
    assert_eq!(0, r_e.unwrap().start_offset);
    assert_eq!(1, r_e.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_hiragana() {
    // ported: getPrevGraphemeStart: hiragana
    let text = "こんにちは";
    let r_last = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_last.is_some());
    assert_eq!(12, r_last.unwrap().start_offset);
    assert_eq!(2, r_last.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_realistic_scenario() {
    // ported: getPrevGraphemeStart: realistic scenario
    let text = "Hello 世界! 👋";
    let r_end = prev_grapheme_start(text, text.len(), 8, WidthMethod::Unicode);
    assert!(r_end.is_some());
    assert_eq!(14, r_end.unwrap().start_offset);
    let r_space = prev_grapheme_start(text, 14, 8, WidthMethod::Unicode);
    assert!(r_space.is_some());
    assert_eq!(13, r_space.unwrap().start_offset);
    assert_eq!(1, r_space.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_consecutive_wide_chars() {
    // ported: getPrevGraphemeStart: consecutive wide chars
    let text = "世界中";
    let r9 = prev_grapheme_start(text, 9, 8, WidthMethod::Unicode);
    assert!(r9.is_some());
    assert_eq!(6, r9.unwrap().start_offset);
    assert_eq!(2, r9.unwrap().width);
    let r6 = prev_grapheme_start(text, 6, 8, WidthMethod::Unicode);
    assert!(r6.is_some());
    assert_eq!(3, r6.unwrap().start_offset);
    assert_eq!(2, r6.unwrap().width);
    let r3 = prev_grapheme_start(text, 3, 8, WidthMethod::Unicode);
    assert!(r3.is_some());
    assert_eq!(0, r3.unwrap().start_offset);
    assert_eq!(2, r3.unwrap().width);
}

#[test]
fn req_005_thai_wrap_by_width_respects_combining_marks() {
    // ported: Thai: wrap by width respects combining marks
    let text = "คือ";
    let result1 = wrap_pos_by_width(text, 1, 4, false, WidthMethod::Unicode);
    assert_eq!(6, result1.byte_offset);
    assert_eq!(1, result1.columns_used);
    let result2 = wrap_pos_by_width(text, 2, 4, false, WidthMethod::Unicode);
    assert_eq!(9, result2.byte_offset);
    assert_eq!(2, result2.columns_used);
}

#[test]
fn req_005_thai_wrap_by_width_with_tone_marks() {
    // ported: Thai: wrap by width with tone marks
    let text = "ก่อน";
    let result2 = wrap_pos_by_width(text, 2, 4, false, WidthMethod::Unicode);
    assert_eq!(2, result2.columns_used);
    let result3 = wrap_pos_by_width(text, 3, 4, false, WidthMethod::Unicode);
    assert_eq!(3, result3.columns_used);
}

#[test]
fn req_004_no_zwj_findwrapposbywidth_with_zwj_sequences() {
    // ported: no_zwj: findWrapPosByWidth with ZWJ sequences
    let text = "AB👩‍🚀CD";
    let result_unicode = wrap_pos_by_width(text, 4, 4, false, WidthMethod::Unicode);
    let result_no_zwj = wrap_pos_by_width(text, 4, 4, false, WidthMethod::NoZwj);
    assert_eq!(4, result_unicode.columns_used);
    assert_eq!(4, result_no_zwj.columns_used);
}

#[test]
fn req_004_no_zwj_findposbywidth_with_zwj_sequences() {
    // ported: no_zwj: findPosByWidth with ZWJ sequences
    let text = "AB👩‍🚀CD";
    let start4_unicode = pos_by_width(text, 4, 4, false, false, WidthMethod::Unicode);
    let start4_no_zwj = pos_by_width(text, 4, 4, false, false, WidthMethod::NoZwj);
    assert_eq!(13, start4_unicode.byte_offset);
    assert_eq!(4, start4_unicode.columns_used);
    assert_eq!(9, start4_no_zwj.byte_offset);
    assert_eq!(4, start4_no_zwj.columns_used);
    let end4_unicode = pos_by_width(text, 4, 4, false, true, WidthMethod::Unicode);
    let end4_no_zwj = pos_by_width(text, 4, 4, false, true, WidthMethod::NoZwj);
    assert_eq!(4, end4_unicode.columns_used);
    assert_eq!(4, end4_no_zwj.columns_used);
}

#[test]
fn req_004_no_zwj_getwidthat_with_zwj_sequence() {
    // ported: no_zwj: getWidthAt with ZWJ sequence
    let text = "👩‍🚀";
    let width_woman_unicode = width_at(text, 0, 4, WidthMethod::Unicode);
    let width_woman_no_zwj = width_at(text, 0, 4, WidthMethod::NoZwj);
    assert_eq!(2, width_woman_unicode);
    assert_eq!(2, width_woman_no_zwj);
    let width_zwj_no_zwj = width_at(text, 4, 4, WidthMethod::NoZwj);
    assert_eq!(0, width_zwj_no_zwj);
}

#[test]
fn req_004_no_zwj_getprevgraphemestart_with_zwj_sequence() {
    // ported: no_zwj: getPrevGraphemeStart with ZWJ sequence
    let text = "AB👩‍🚀CD";
    let r1_unicode = prev_grapheme_start(text, text.len(), 4, WidthMethod::Unicode);
    let r1_no_zwj = prev_grapheme_start(text, text.len(), 4, WidthMethod::NoZwj);
    assert!(r1_unicode.is_some());
    assert!(r1_no_zwj.is_some());
    assert_eq!(1, r1_unicode.unwrap().width);
    assert_eq!(1, r1_no_zwj.unwrap().width);
}

#[test]
fn req_004_getprevgraphemestart_wcwidth_each_codepoint_separate() {
    // ported: getPrevGraphemeStart wcwidth: each codepoint separate
    let text = "Hi👋🏿";
    let r_end = prev_grapheme_start(text, text.len(), 4, WidthMethod::Wcwidth);
    assert!(r_end.is_some());
    assert_eq!(6, r_end.unwrap().start_offset);
    assert_eq!(2, r_end.unwrap().width);
    let r_wave = prev_grapheme_start(text, 6, 4, WidthMethod::Wcwidth);
    assert!(r_wave.is_some());
    assert_eq!(2, r_wave.unwrap().start_offset);
    assert_eq!(2, r_wave.unwrap().width);
}

#[test]
fn req_004_wcwidth_findwrapposbywidth_with_zwj_sequences() {
    // ported: wcwidth: findWrapPosByWidth with ZWJ sequences
    let text = "AB👩‍🚀CD";
    let result = wrap_pos_by_width(text, 4, 4, false, WidthMethod::Wcwidth);
    assert_eq!(6, result.byte_offset);
    assert_eq!(4, result.columns_used);
}

#[test]
fn req_004_wcwidth_findposbywidth_with_skin_tone_modifier() {
    // ported: wcwidth: findPosByWidth with skin tone modifier
    let text = "AB👋🏻CD";
    let start4 = pos_by_width(text, 4, 4, false, false, WidthMethod::Wcwidth);
    assert_eq!(6, start4.byte_offset);
    assert_eq!(4, start4.columns_used);
    let end4 = pos_by_width(text, 4, 4, false, true, WidthMethod::Wcwidth);
    assert_eq!(6, end4.byte_offset);
    assert_eq!(4, end4.columns_used);
}

// ---- hand-ported: goldens, builders, deterministic randoms, corpus ----

use suprtui::uni::segments::{
    DecodeError, LayoutWrapBreakKind, LineBreakKind, TextWidthCursor, chunk_layout_info,
    decode_utf8_at, is_word_codepoint, line_and_word_chunk_layout_info, line_breaks, tab_stops,
    word_chunk_layout_info,
};

include!("golden_tables.inc");

#[test]
fn req_006_line_break_goldens() {
    // ported: line breaks: golden tests
    for (input, expected) in LINE_BREAK_GOLDEN {
        let got: Vec<usize> = line_breaks(input).iter().map(|b| b.pos).collect();
        assert_eq!(*expected, got.as_slice(), "input {input:?}");
    }
}

#[test]
fn req_006_line_break_kinds() {
    // kinds implied by the golden positions, asserted directly
    let cases: &[(&str, &[(usize, LineBreakKind)])] = &[
        ("a\nb", &[(1, LineBreakKind::Lf)]),
        ("a\rb", &[(1, LineBreakKind::Cr)]),
        ("a\r\nb", &[(2, LineBreakKind::Crlf)]),
        (
            "\n\r\n\r",
            &[
                (0, LineBreakKind::Lf),
                (2, LineBreakKind::Crlf),
                (3, LineBreakKind::Cr),
            ],
        ),
        (
            "\r\r\n",
            &[(0, LineBreakKind::Cr), (2, LineBreakKind::Crlf)],
        ),
        (
            "unix\nmac\rwin\r\n",
            &[
                (4, LineBreakKind::Lf),
                (8, LineBreakKind::Cr),
                (13, LineBreakKind::Crlf),
            ],
        ),
    ];
    for (input, expected) in cases {
        let got: Vec<(usize, LineBreakKind)> =
            line_breaks(input).iter().map(|b| (b.pos, b.kind)).collect();
        assert_eq!(*expected, got.as_slice(), "input {input:?}");
    }
}

#[test]
fn req_006_line_break_builders() {
    // ported: CRLF at SIMD16 edge, multiple breaks, multibyte adjacency,
    // realistic text (byte-poke builders, SIMD-width independent)
    let mut buf = vec![b'x'; 32];
    buf[15] = b'\r';
    buf[16] = b'\n';
    let got: Vec<usize> = line_breaks(std::str::from_utf8(&buf).unwrap())
        .iter()
        .map(|b| b.pos)
        .collect();
    assert_eq!(&[16], got.as_slice());

    let mut buf = vec![b'x'; 32];
    buf[14] = b'\n';
    buf[15] = b'\r';
    buf[16] = b'\n';
    buf[17] = b'\n';
    let got: Vec<usize> = line_breaks(std::str::from_utf8(&buf).unwrap())
        .iter()
        .map(|b| b.pos)
        .collect();
    assert_eq!(&[14, 16, 17], got.as_slice());

    for (input, expected) in [
        ("日本語\ntext", vec![9]),
        ("日本語\r\ntext", vec![10]),
        ("line1\nline2\r\nline3\rlast", vec![5, 12, 18]),
    ] {
        let got: Vec<usize> = line_breaks(input).iter().map(|b| b.pos).collect();
        assert_eq!(expected, got, "input {input:?}");
    }
}

#[test]
fn req_005_tab_stop_goldens() {
    // ported: tab stops: golden tests
    for (input, expected) in TAB_STOP_GOLDEN {
        assert_eq!(*expected, tab_stops(input).as_slice(), "input {input:?}");
    }
}

#[test]
fn req_005_tab_stop_builders() {
    // ported: SIMD-edge, multibyte-adjacent, realistic, TSV, makefile,
    // multi-chunk, reuse-equivalent, lanes, periodic, 16/17-byte cases
    let mut buf = vec![b'x'; 32];
    buf[15] = b'\t';
    assert_eq!(
        &[15],
        tab_stops(std::str::from_utf8(&buf).unwrap()).as_slice()
    );
    let mut buf = vec![b'x'; 32];
    buf[16] = b'\t';
    assert_eq!(
        &[16],
        tab_stops(std::str::from_utf8(&buf).unwrap()).as_slice()
    );

    for (input, expected) in [
        ("a\t世\tb", vec![1, 5]),
        ("日本語\ttext", vec![9]),
        ("col1\tcol2\tcol3", vec![4, 9]),
        ("a\tb\nc\td", vec![1, 5]),
        ("\t\t\t", vec![0, 1, 2]),
    ] {
        assert_eq!(expected, tab_stops(input), "input {input:?}");
    }

    let mut big = vec![b'a'; 64];
    for i in [0, 16, 32, 48, 63] {
        big[i] = b'\t';
    }
    assert_eq!(
        &[0, 16, 32, 48, 63],
        tab_stops(std::str::from_utf8(&big).unwrap()).as_slice()
    );

    let sixteen = "123456789012345\ttail";
    assert_eq!(&[15], tab_stops(sixteen).as_slice());
    let seventeen = "1234567890123456\ttail";
    assert_eq!(&[16], tab_stops(seventeen).as_slice());

    // makefile style + TSV data ("target:" is bytes 0..7, '\n' at 7, '\t' at 8)
    assert_eq!(&[8], tab_stops("target:\n\tcommand").as_slice());
    assert_eq!(&[1, 3], tab_stops("a\tb\tc").as_slice());
}

#[test]
fn req_005_chunk_layout_goldens() {
    // ported: chunk layout scanner: golden break offsets
    for (input, expected) in LAYOUT_BREAK_GOLDEN {
        let ascii = input.bytes().all(|b| b < 0x80);
        let (breaks, _) = chunk_layout_info(input, 4, ascii, WidthMethod::Unicode);
        let got: Vec<usize> = breaks.iter().map(|b| b.byte_start as usize).collect();
        assert_eq!(*expected, got.as_slice(), "input {input:?}");
    }
}

#[test]
fn req_005_chunk_layout_kinds_and_edges() {
    // kinds implied by the goldens plus edge metadata, asserted directly
    let (breaks, edges) = chunk_layout_info("Hello, world!", 4, true, WidthMethod::Unicode);
    // ',' at 5, ' ' at 6, '!' at 12
    assert_eq!(
        vec![5, 6, 12],
        breaks.iter().map(|b| b.byte_start).collect::<Vec<_>>()
    );
    assert!(
        breaks
            .iter()
            .all(|b| b.kind == LayoutWrapBreakKind::Punctuation
                || b.kind == LayoutWrapBreakKind::Whitespace)
    );
    assert_eq!(suprtui::uni::segments::WordClass::AsciiWord, edges.first);
    // trailing '!' classifies .other per reference classifyWordClass
    assert_eq!(suprtui::uni::segments::WordClass::Other, edges.last);

    let (breaks, edges) = chunk_layout_info("你好世界", 4, false, WidthMethod::Unicode);
    assert_eq!(
        vec![0, 3, 6],
        breaks.iter().map(|b| b.byte_start).collect::<Vec<_>>()
    );
    assert!(
        breaks
            .iter()
            .all(|b| b.kind == LayoutWrapBreakKind::CjkIntercharacter)
    );
    assert!(edges.has_cjk_breaks);

    // word-only mode drops CJK intercharacter breaks but keeps the flag
    let (words, edges) = word_chunk_layout_info("你好世界", 4, false, WidthMethod::Unicode);
    assert!(words.is_empty());
    assert!(edges.has_cjk_breaks);

    // line+word split on mixed content
    let (lines, words, _) =
        line_and_word_chunk_layout_info("a b你好", 4, false, WidthMethod::Unicode);
    assert!(
        words
            .iter()
            .any(|b| b.kind == LayoutWrapBreakKind::Whitespace)
    );
    assert!(
        lines
            .iter()
            .any(|b| b.kind == LayoutWrapBreakKind::CjkIntercharacter)
    );

    // word codespot checks
    assert!(is_word_codepoint('a' as u32));
    assert!(is_word_codepoint(0x4E2D));
    assert!(!is_word_codepoint(' ' as u32));
}

#[test]
fn req_005_chunk_layout_simd_edges() {
    // ported: wrap breaks at SIMD16 edges and script transitions
    let mut buf = vec![b'x'; 32];
    buf[15] = b' ';
    buf[16] = b'y';
    let (breaks, _) = chunk_layout_info(
        std::str::from_utf8(&buf).unwrap(),
        4,
        true,
        WidthMethod::Unicode,
    );
    assert_eq!(
        &[15],
        breaks
            .iter()
            .map(|b| b.byte_start as usize)
            .collect::<Vec<_>>()
            .as_slice()
    );

    let text = format!("{}a\u{00A0}b", "x".repeat(14));
    let (breaks, _) = chunk_layout_info(&text, 4, false, WidthMethod::Unicode);
    assert!(breaks.iter().any(|b| b.byte_start as usize == 15));
}

#[test]
fn req_004_isasciionly_builders() {
    // ported: all printable ASCII, large ASCII text
    let all: Vec<u8> = (32..=126).collect();
    assert!(suprtui::uni::is_ascii_only(&all));
    assert!(suprtui::uni::is_ascii_only(&vec![b'a'; 10000]));
    let mut edge = vec![b'a'; 32];
    edge[15] = 0x7F;
    assert!(!suprtui::uni::is_ascii_only(&edge));
    edge[15] = b'a';
    edge[16] = 0x80;
    assert!(!suprtui::uni::is_ascii_only(&edge));
}

#[test]
fn req_005_wrap_consistency() {
    // ported: wrap by width consistency suites (tables the transpiler skips)
    let sample = "The quick brown fox jumps over the lazy dog. Lorem ipsum dolor sit amet. File paths: /usr/local/bin.";
    for w in [10, 20, 40, 80, 120] {
        for m in [
            WidthMethod::Unicode,
            WidthMethod::Wcwidth,
            WidthMethod::NoZwj,
            WidthMethod::UnicodeWide,
        ] {
            let r = wrap_pos_by_width(sample, w, 4, true, m);
            assert!(r.byte_offset <= sample.len() as u32);
            assert!(r.columns_used <= w);
        }
    }
    let uni = "世界 こんにちは test 你好 CJK-mixed";
    for w in [5, 10, 15, 20, 30] {
        for m in [
            WidthMethod::Unicode,
            WidthMethod::Wcwidth,
            WidthMethod::NoZwj,
            WidthMethod::UnicodeWide,
        ] {
            let r = wrap_pos_by_width(uni, w, 4, false, m);
            assert!(r.columns_used <= w, "w={w} m={m:?} got {r:?}");
            assert!(uni.is_char_boundary(r.byte_offset as usize));
        }
    }
    for (text, ascii) in [
        ("", false),
        (" ", true),
        ("a", true),
        ("abc", true),
        ("   ", true),
        ("a b c d e", true),
        ("no-spaces-here", true),
        ("/usr/local/bin", true),
        ("世界", false),
        ("\t\t\t", false),
    ] {
        for w in [1, 5, 10, 20] {
            for m in [
                WidthMethod::Unicode,
                WidthMethod::Wcwidth,
                WidthMethod::NoZwj,
                WidthMethod::UnicodeWide,
            ] {
                let r = wrap_pos_by_width(text, w, 4, ascii, m);
                assert!(r.columns_used <= w, "{text:?} w={w} m={m:?} got {r:?}");
            }
        }
    }
    // tab handling case the transpiler skips
    let r = wrap_pos_by_width("a\tb", 5, 4, false, WidthMethod::Unicode);
    assert_eq!(2, r.byte_offset);
    assert_eq!(2, r.grapheme_count);
    assert_eq!(5, r.columns_used);
}

#[test]
fn req_005_wrap_simd_edges() {
    // ported: wrap boundary SIMD16 tests (builders)
    let mut buf = vec![b'a'; 40];
    buf[16] = b' ';
    let text = std::str::from_utf8(&buf).unwrap();
    let r = wrap_pos_by_width(text, 20, 4, true, WidthMethod::Unicode);
    assert_eq!(20, r.byte_offset);
    let wide: String = "x".repeat(14) + "世界" + &"y".repeat(14);
    let r = wrap_pos_by_width(&wide, 16, 4, false, WidthMethod::Unicode);
    // 14 ASCII cols + U+4E16 (2 cols) = 16 <= 16 commits; U+754C starts at byte 17
    assert_eq!(17, r.byte_offset);
    assert_eq!(16, r.columns_used);
}

#[test]
fn req_005_deterministic_randoms() {
    // ported shape of the fixed-seed random suites (xorshift64, seed 42):
    // crash-freedom plus wrap/pos invariants over random ASCII
    fn next(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }
    let mut st = 42u64;
    for _ in 0..50 {
        let size = 16 + (next(&mut st) % 1024) as usize;
        let mut buf = vec![0u8; size];
        for b in buf.iter_mut() {
            let r = (next(&mut st) % 100) as u8;
            *b = if r < 5 {
                b'\n'
            } else if r < 10 {
                b'\r'
            } else {
                b'a' + (next(&mut st) % 26) as u8
            };
        }
        let text = std::str::from_utf8(&buf).unwrap();
        let _ = line_breaks(text);
        let _ = tab_stops(text);
        let _ = wrap_pos_by_width(text, 40, 4, false, WidthMethod::Unicode);
        let _ = pos_by_width(text, 40, 4, false, true, WidthMethod::Unicode);
    }
    let mut st = 42u64;
    for _ in 0..50 {
        let size = 16 + (next(&mut st) % 256) as usize;
        let buf: Vec<u8> = (0..size)
            .map(|_| b'a' + (next(&mut st) % 26) as u8)
            .collect();
        let text = std::str::from_utf8(&buf).unwrap();
        let width = 10 + (next(&mut st) % 70) as u32;
        let r = wrap_pos_by_width(text, width, 4, true, WidthMethod::Unicode);
        assert!(r.columns_used <= width);
    }
}

#[test]
fn req_007_decode_corpus() {
    // valid sequences decode exactly (mirrors the unchecked oracle)
    assert_eq!(('A', 1), decode_utf8_at(b"A", 0).unwrap());
    assert_eq!(('¢', 2), decode_utf8_at("¢".as_bytes(), 0).unwrap());
    assert_eq!(('日', 3), decode_utf8_at("日".as_bytes(), 0).unwrap());
    assert_eq!(('𐀀', 4), decode_utf8_at("𐀀".as_bytes(), 0).unwrap());
    assert_eq!(
        ('\u{10FFFF}', 4),
        decode_utf8_at("\u{10FFFF}".as_bytes(), 0).unwrap()
    );
    // every scalar round-trips (reference loops the same range)
    for value in 0..0x110000u32 {
        if (0xD800..0xE000).contains(&value) {
            continue;
        }
        let ch = char::from_u32(value).unwrap();
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        assert_eq!(
            (ch, s.len() as u8),
            decode_utf8_at(s.as_bytes(), 0).unwrap(),
            "U+{value:04X}"
        );
    }
    // truncated inputs
    assert_eq!(Err(DecodeError::OutOfBounds), decode_utf8_at(b"", 0));
    assert_eq!(Err(DecodeError::OutOfBounds), decode_utf8_at(b"a", 1));
    assert_eq!(Err(DecodeError::UnexpectedEnd), decode_utf8_at(b"\xC2", 0));
    // Offset into the middle of U+65E5 lands on continuation byte 0x97:
    // no lead-byte branch matches, so InvalidStart (iterator still advances 1).
    assert_eq!(
        Err(DecodeError::InvalidStart),
        decode_utf8_at("日".as_bytes(), 1)
    );
    // overlongs
    assert_eq!(Err(DecodeError::Overlong), decode_utf8_at(b"\xC0\xAF", 0));
    assert_eq!(
        Err(DecodeError::Overlong),
        decode_utf8_at(b"\xE0\x80\xAF", 0)
    );
    assert_eq!(
        Err(DecodeError::Overlong),
        decode_utf8_at(b"\xF0\x80\x80\xAF", 0)
    );
    // surrogates
    assert_eq!(
        Err(DecodeError::Surrogate),
        decode_utf8_at(b"\xED\xA0\x80", 0)
    );
    // too large
    assert_eq!(
        Err(DecodeError::TooLarge),
        decode_utf8_at(b"\xF4\x90\x80\x80", 0)
    );
    assert_eq!(
        Err(DecodeError::TooLarge),
        decode_utf8_at(b"\xF5\x80\x80\x80", 0)
    );
    // bad starts and continuations
    assert_eq!(Err(DecodeError::InvalidStart), decode_utf8_at(b"\x80", 0));
    assert_eq!(Err(DecodeError::InvalidStart), decode_utf8_at(b"\xFF", 0));
    assert_eq!(Err(DecodeError::InvalidStart), decode_utf8_at(b"\xFE", 0));
    assert_eq!(
        Err(DecodeError::InvalidContinuation),
        decode_utf8_at(b"\xC2\x41", 0)
    );
    assert_eq!(
        Err(DecodeError::InvalidContinuation),
        decode_utf8_at(b"\xE2\x28\xA1", 0)
    );
    // malformed first-byte matrix (reference shape): progress stays bounded
    for first in 0..256u32 {
        let bytes = [first as u8, 0xBF, 0xBF, 0xBF];
        for len in 1..5usize {
            if decode_utf8_at(&bytes[..len], 0).is_ok() {
                let (ch, n) = decode_utf8_at(&bytes[..len], 0).unwrap();
                assert!(n as usize <= len, "first={first:#X} len={len}");
                assert!(ch.len_utf8() == n as usize);
            }
        }
    }
}

#[test]
fn req_009_packing_vectors() {
    use suprtui::uni::segments::*;
    // flags and masks
    assert_eq!(0x4000_0000, CHAR_FLAG_IMAGE);
    assert_eq!(0x8000_0000, CHAR_FLAG_GRAPHEME);
    assert_eq!(0xC000_0000, CHAR_FLAG_CONTINUATION);
    // plain scalars select none of the flags
    for cp in ['A' as u32, '世' as u32, 0, 0x10FFFF] {
        assert!(!is_image_char(cp));
        assert!(!is_grapheme_char(cp));
        assert!(!is_continuation_char(cp));
        assert!(!is_cluster_char(cp));
        assert_eq!(1, encoded_char_width(cp));
    }
    // image round-trip mirrors the reference asserts
    let img = pack_image_cell(0x1234, 0xA);
    assert!(is_image_char(img));
    assert!(!is_grapheme_char(img));
    assert_eq!(0x1234, image_id_from_char(img));
    assert_eq!(0xA, image_fallback_from_char(img));
    let img_max = pack_image_cell(IMAGE_ID_MASK, 0xF);
    assert_eq!(IMAGE_ID_MASK, image_id_from_char(img_max));
    assert_eq!(0xF, image_fallback_from_char(img_max));
    // grapheme start round-trip with capped extents
    let g = pack_grapheme_start(0x42, 2);
    assert!(is_grapheme_char(g));
    assert!(is_cluster_char(g));
    assert!(!is_continuation_char(g));
    assert_eq!(0x42, grapheme_id_from_char(g));
    assert_eq!(1, char_right_extent(g));
    assert_eq!(0, char_left_extent(g));
    assert_eq!(2, encoded_char_width(g));
    // wide logical widths cap at the four-cell encoded span
    let g5 = pack_grapheme_start(0x42, 9);
    assert_eq!(3, char_right_extent(g5));
    assert_eq!(4, encoded_char_width(g5));
    // continuation round-trip
    let c = pack_continuation(2, 1, 0x99);
    assert!(is_continuation_char(c));
    assert!(is_cluster_char(c));
    assert_eq!(0x99, grapheme_id_from_char(c));
    assert_eq!(2, char_left_extent(c));
    assert_eq!(1, char_right_extent(c));
    assert_eq!(4, encoded_char_width(c));
    // ids never alias the flag bits (reference comptime asserts)
    assert_eq!(0, CHAR_FLAG_GRAPHEME & GRAPHEME_ID_MASK);
    assert_eq!(0, CHAR_FLAG_CONTINUATION & GRAPHEME_ID_MASK);
    assert_ne!(CHAR_FLAG_GRAPHEME, CHAR_FLAG_CONTINUATION);
}

#[test]
fn req_005_cursor_equivalence() {
    // advance_to the whole text equals calculate_text_width, and every
    // split point composes to the same total, for every method
    let texts = [
        "hello world",
        "a\tb\tc",
        "世界こんにちは",
        "👩‍👩‍👧‍👦 family",
        "e\u{301} café",
        "A👋🏿B世C",
        " Élève naïve ",
    ];
    for text in texts {
        for m in [
            WidthMethod::Unicode,
            WidthMethod::Wcwidth,
            WidthMethod::NoZwj,
            WidthMethod::UnicodeWide,
        ] {
            let full = calculate_text_width(text, 4, false, m);
            let mut cur = TextWidthCursor::new(text, 4, m);
            assert_eq!(full, cur.advance_to(text.len()), "{text:?} {m:?}");
            // every split composes
            let bounds: Vec<usize> = {
                let mut v: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
                v.push(text.len());
                v
            };
            for a in &bounds {
                for b in &bounds {
                    if a <= b {
                        let mut cur = TextWidthCursor::new(text, 4, m);
                        cur.advance_to(*a);
                        let got = cur.advance_to(*b);
                        let mut whole = TextWidthCursor::new(text, 4, m);
                        let want = whole.advance_to(*b);
                        assert_eq!(want, got, "{text:?} {m:?} split {a}/{b}");
                    }
                }
            }
            // clamping past the end and idempotent re-advance
            let mut cur = TextWidthCursor::new(text, 4, m);
            assert_eq!(full, cur.advance_to(text.len() + 100));
            assert_eq!(full, cur.advance_to(text.len()));
        }
    }
}

#[test]
fn req_005_grapheme_safe_equivalence() {
    // wrap_pos_grapheme_safe matches the plain call except that
    // wcwidth uses cluster boundaries with summed widths
    let texts = ["hello world", "A👋🏿B", "👩‍🚀 test", "e\u{301}x", "a\tb"];
    for text in texts {
        for w in [1, 2, 3, 5, 10, 40] {
            for m in [
                WidthMethod::Unicode,
                WidthMethod::NoZwj,
                WidthMethod::UnicodeWide,
            ] {
                assert_eq!(
                    wrap_pos_by_width(text, w, 4, false, m),
                    wrap_pos_grapheme_safe(text, w, 4, false, m),
                    "{text:?} w={w} m={m:?}"
                );
            }
            let r = wrap_pos_grapheme_safe(text, w, 4, false, WidthMethod::Wcwidth);
            assert!(r.columns_used <= w, "{text:?} w={w} got {r:?}");
            assert!(text.is_char_boundary(r.byte_offset as usize));
        }
    }
    // grapheme_pos_by_width equals pos_by_width for non-wcwidth methods
    for text in texts {
        for w in [1, 2, 3, 5, 10, 40] {
            for inc in [false, true] {
                for m in [
                    WidthMethod::Unicode,
                    WidthMethod::NoZwj,
                    WidthMethod::UnicodeWide,
                ] {
                    assert_eq!(
                        pos_by_width(text, w, 4, false, inc, m),
                        grapheme_pos_by_width(text, w, 4, false, inc, m),
                        "{text:?} w={w} inc={inc} m={m:?}"
                    );
                }
            }
        }
    }
}
