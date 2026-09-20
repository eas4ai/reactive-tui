//! Ported reference vectors — see docs/commitments/uni-width.md.
use suprtui::uni::{
    WidthMethod, calculate_text_width, cell_width, is_ascii_only, render_clusters, width_at,
};

#[allow(dead_code)]
fn _imports() {
    let _ = (
        WidthMethod::Unicode,
        is_ascii_only(b""),
        width_at("", 0, 4, WidthMethod::Unicode),
    );
}

#[test]
fn req_002_eastasianwidth_verify_all_characters_in_test_string_have_correct_width() {
    // ported: eastAsianWidth: verify all characters in test string have correct width
    assert_eq!(2, cell_width(0x3053, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x3093, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x306B, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x3061, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x306F, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x4E16, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x754C, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x1F31F, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x1F680, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x4F60, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0x597D, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0xC548, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0xB155, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0xD558, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0xC138, WidthMethod::Unicode));
    assert_eq!(2, cell_width(0xC694, WidthMethod::Unicode));
    assert_eq!(1, cell_width('H' as u32, WidthMethod::Unicode));
    assert_eq!(1, cell_width('e' as u32, WidthMethod::Unicode));
    assert_eq!(1, cell_width(' ' as u32, WidthMethod::Unicode));
    assert_eq!(1, cell_width(':' as u32, WidthMethod::Unicode));
}

#[test]
fn req_002_calculatetextwidth_verify_cjk_string_widths_character_by_character() {
    // ported: calculateTextWidth: verify CJK string widths character by character
    assert_eq!(
        2,
        calculate_text_width("こ", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("ん", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("に", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("ち", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("は", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("世", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("界", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        4,
        calculate_text_width("こん", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        6,
        calculate_text_width("こんに", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        14,
        calculate_text_width("こんにちは世界", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        5,
        calculate_text_width("Hello", 8, true, WidthMethod::Unicode)
    );
    assert_eq!(
        6,
        calculate_text_width("Hello ", 8, true, WidthMethod::Unicode)
    );
    assert_eq!(
        8,
        calculate_text_width("Hello 世", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        10,
        calculate_text_width("Hello 世界", 8, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_step_by_step_for_emoji_cjk_test_string() {
    // ported: calculateTextWidth: step by step for emoji CJK test string
    assert_eq!(
        2,
        calculate_text_width("🌟", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        3,
        calculate_text_width("🌟 ", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        10,
        calculate_text_width("🌟 Unicode", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        11,
        calculate_text_width("🌟 Unicode ", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        15,
        calculate_text_width("🌟 Unicode test", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        16,
        calculate_text_width("🌟 Unicode test:", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        17,
        calculate_text_width("🌟 Unicode test: ", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        19,
        calculate_text_width("🌟 Unicode test: こ", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        21,
        calculate_text_width("🌟 Unicode test: こん", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        23,
        calculate_text_width("🌟 Unicode test: こんに", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        25,
        calculate_text_width("🌟 Unicode test: こんにち", 8, false, WidthMethod::Unicode)
    );
    assert_eq!(
        27,
        calculate_text_width(
            "🌟 Unicode test: こんにちは",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        29,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        31,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        32,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界 ",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        33,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界 H",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        37,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界 Hello",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        38,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界 Hello ",
            8,
            false,
            WidthMethod::Unicode
        )
    );
    assert_eq!(
        43,
        calculate_text_width(
            "🌟 Unicode test: こんにちは世界 Hello World",
            8,
            false,
            WidthMethod::Unicode
        )
    );
}

#[test]
fn req_002_getwidthat_empty_string() {
    // ported: getWidthAt: empty string
    let result = width_at("", 0, 8, WidthMethod::Unicode);
    assert_eq!(0, result);
}

#[test]
fn req_002_getwidthat_out_of_bounds() {
    // ported: getWidthAt: out of bounds
    let result = width_at("hello", 10, 8, WidthMethod::Unicode);
    assert_eq!(0, result);
}

#[test]
fn req_002_getwidthat_simple_ascii() {
    // ported: getWidthAt: simple ASCII
    let text = "hello";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 1, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 4, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_tab_character() {
    // ported: getWidthAt: tab character
    let text = "a\tb";
    assert_eq!(1, width_at(text, 0, 4, WidthMethod::Unicode));
    assert_eq!(4, width_at(text, 1, 4, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 2, 4, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_tab_at_different_columns() {
    // ported: getWidthAt: tab at different columns
    let text = "\t";
    assert_eq!(4, width_at(text, 0, 4, WidthMethod::Unicode));
    assert_eq!(4, width_at(text, 0, 4, WidthMethod::Unicode));
    assert_eq!(4, width_at(text, 0, 4, WidthMethod::Unicode));
    assert_eq!(4, width_at(text, 0, 4, WidthMethod::Unicode));
    assert_eq!(4, width_at(text, 0, 4, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_cjk_wide_character() {
    // ported: getWidthAt: CJK wide character
    let text = "世界";
    assert_eq!(2, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 3, 8, WidthMethod::Unicode));
}

#[test]
fn req_003_getwidthat_emoji_single_width() {
    // ported: getWidthAt: emoji single width
    let text = "🌍";
    assert_eq!(2, width_at(text, 0, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_combining_mark_grapheme() {
    // ported: getWidthAt: combining mark grapheme
    let text = "cafe\u{0301}";
    let width = width_at(text, 3, 8, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_003_getwidthat_emoji_with_skin_tone() {
    // ported: getWidthAt: emoji with skin tone
    let text = "👋🏿";
    let width = width_at(text, 0, 8, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_getwidthat_emoji_with_zwj() {
    // ported: getWidthAt: emoji with ZWJ
    let text = "👩‍🚀";
    let width = width_at(text, 0, 8, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_getwidthat_flag_emoji() {
    // ported: getWidthAt: flag emoji
    let text = "🇺🇸";
    let width = width_at(text, 0, 8, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_002_getwidthat_mixed_ascii_and_cjk() {
    // ported: getWidthAt: mixed ASCII and CJK
    let text = "Hello世界";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 1, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 5, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 8, 8, WidthMethod::Unicode));
}

#[test]
fn req_003_getwidthat_emoji_with_vs16_selector() {
    // ported: getWidthAt: emoji with VS16 selector
    let text = "❤️";
    let width = width_at(text, 0, 8, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_002_getwidthat_hiragana() {
    // ported: getWidthAt: hiragana
    let text = "こんにちは";
    assert_eq!(2, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 3, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_katakana() {
    // ported: getWidthAt: katakana
    let text = "カタカナ";
    assert_eq!(2, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 3, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_fullwidth_forms() {
    // ported: getWidthAt: fullwidth forms
    let text = "ＡＢＣ";
    assert_eq!(2, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 3, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_zero_width_at_start_of_string() {
    // ported: getWidthAt: zero width at start of string
    let text = "a\u{0301}bc";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 3, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_control_characters() {
    // ported: getWidthAt: control characters
    let text = "a\x00b";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(0, width_at(text, 1, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 2, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_multiple_combining_marks() {
    // ported: getWidthAt: multiple combining marks
    let text = "e\u{0301}\u{0302}";
    let width = width_at(text, 0, 8, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_002_getwidthat_at_exact_end_boundary() {
    // ported: getWidthAt: at exact end boundary
    let text = "hello";
    let width = width_at(text, 5, 8, WidthMethod::Unicode);
    assert_eq!(0, width);
}

#[test]
fn req_002_getwidthat_realistic_mixed_content() {
    // ported: getWidthAt: realistic mixed content
    let text = "Hello 世界! 👋";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 5, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 6, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 9, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 12, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 13, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 14, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_getwidthat_random_positions_in_realistic_text() {
    // ported: getWidthAt: random positions in realistic text
    let text = "The quick brown 🦊 jumps over the lazy 犬";
    assert_eq!(1, width_at(text, 0, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 10, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 16, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 41, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_calculatetextwidth_empty_string() {
    // ported: calculateTextWidth: empty string
    let result = calculate_text_width("", 4, false, WidthMethod::Unicode);
    assert_eq!(0, result);
}

#[test]
fn req_002_calculatetextwidth_simple_ascii() {
    // ported: calculateTextWidth: simple ASCII
    let result = calculate_text_width("hello", 4, true, WidthMethod::Unicode);
    assert_eq!(5, result);
}

#[test]
fn req_002_calculatetextwidth_single_tab() {
    // ported: calculateTextWidth: single tab
    let result = calculate_text_width("\t", 4, false, WidthMethod::Unicode);
    assert_eq!(4, result);
}

#[test]
fn req_002_calculatetextwidth_tab_with_different_widths() {
    // ported: calculateTextWidth: tab with different widths
    assert_eq!(
        2,
        calculate_text_width("\t", 2, false, WidthMethod::Unicode)
    );
    assert_eq!(
        4,
        calculate_text_width("\t", 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        8,
        calculate_text_width("\t", 8, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_multiple_tabs() {
    // ported: calculateTextWidth: multiple tabs
    let result = calculate_text_width("\t\t\t", 4, false, WidthMethod::Unicode);
    assert_eq!(12, result);
}

#[test]
fn req_002_calculatetextwidth_text_with_tabs() {
    // ported: calculateTextWidth: text with tabs
    let result = calculate_text_width("a\tb", 4, false, WidthMethod::Unicode);
    assert_eq!(6, result);
}

#[test]
fn req_002_calculatetextwidth_multiple_tabs_between_text() {
    // ported: calculateTextWidth: multiple tabs between text
    let result = calculate_text_width("a\t\tb", 2, false, WidthMethod::Unicode);
    assert_eq!(6, result);
}

#[test]
fn req_002_calculatetextwidth_tab_at_start() {
    // ported: calculateTextWidth: tab at start
    let result = calculate_text_width("\tabc", 4, false, WidthMethod::Unicode);
    assert_eq!(7, result);
}

#[test]
fn req_002_calculatetextwidth_tab_at_end() {
    // ported: calculateTextWidth: tab at end
    let result = calculate_text_width("abc\t", 4, false, WidthMethod::Unicode);
    assert_eq!(7, result);
}

#[test]
fn req_002_calculatetextwidth_cjk_with_tabs() {
    // ported: calculateTextWidth: CJK with tabs
    let result = calculate_text_width("世\t界", 4, false, WidthMethod::Unicode);
    assert_eq!(8, result);
}

#[test]
fn req_003_calculatetextwidth_emoji_with_tab() {
    // ported: calculateTextWidth: emoji with tab
    let result = calculate_text_width("🌍\t", 4, false, WidthMethod::Unicode);
    assert_eq!(6, result);
}

#[test]
fn req_002_calculatetextwidth_mixed_ascii_and_unicode_with_tabs() {
    // ported: calculateTextWidth: mixed ASCII and Unicode with tabs
    let result = calculate_text_width("hello\t世界", 4, false, WidthMethod::Unicode);
    assert_eq!(13, result);
}

#[test]
fn req_002_calculatetextwidth_realistic_code_with_tabs() {
    // ported: calculateTextWidth: realistic code with tabs
    let text = "\tif (x > 5) {\n\t\treturn true;\n\t}";
    let result = calculate_text_width(text, 2, false, WidthMethod::Unicode);
    assert_eq!(33, result);
}

#[test]
fn req_002_calculatetextwidth_only_spaces() {
    // ported: calculateTextWidth: only spaces
    let result = calculate_text_width("     ", 4, true, WidthMethod::Unicode);
    assert_eq!(5, result);
}

#[test]
fn req_002_calculatetextwidth_tabs_and_spaces_mixed() {
    // ported: calculateTextWidth: tabs and spaces mixed
    let result = calculate_text_width("  \t  \t  ", 4, false, WidthMethod::Unicode);
    assert_eq!(14, result);
}

#[test]
fn req_002_calculatetextwidth_control_characters() {
    // ported: calculateTextWidth: control characters
    let result = calculate_text_width("a\x00b\x1Fc", 4, false, WidthMethod::Unicode);
    assert_eq!(3, result);
}

#[test]
fn req_002_calculatetextwidth_combining_marks() {
    // ported: calculateTextWidth: combining marks
    let result = calculate_text_width("cafe\u{0301}", 4, false, WidthMethod::Unicode);
    assert_eq!(4, result);
}

#[test]
fn req_003_calculatetextwidth_scroll_book_and_writing_emojis_width_2() {
    // ported: calculateTextWidth: scroll book and writing emojis width 2
    assert_eq!(
        2,
        calculate_text_width("📜", 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_devanagari_width_4() {
    // ported: calculateTextWidth: Devanagari नमस्ते width 4
    let result = calculate_text_width("नमस्ते", 4, false, WidthMethod::Unicode);
    assert_eq!(4, result);
}

#[test]
fn req_002_calculatetextwidth_u_26a0_warning_sign_should_be_width_1() {
    // ported: calculateTextWidth: U+26A0 warning sign should be width 1
    let result = calculate_text_width("⚠", 4, false, WidthMethod::Unicode);
    assert_eq!(1, result);
}

#[test]
fn req_002_calculatetextwidth_u_2049_exclamation_question_mark_should_be_width_2() {
    // ported: calculateTextWidth: U+2049 exclamation question mark should be width 2
    let result = calculate_text_width("⁉", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_u_203c_double_exclamation_mark_should_be_width_2() {
    // ported: calculateTextWidth: U+203C double exclamation mark should be width 2
    let result = calculate_text_width("‼", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_u_26d1_rescue_worker_helmet_should_be_width_2() {
    // ported: calculateTextWidth: U+26D1 rescue worker helmet should be width 2
    let result = calculate_text_width("⛑", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_u_2622_radioactive_sign_should_be_width_2() {
    // ported: calculateTextWidth: U+2622 radioactive sign should be width 2
    let result = calculate_text_width("☢", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_u_2623_biohazard_sign_should_be_width_2() {
    // ported: calculateTextWidth: U+2623 biohazard sign should be width 2
    let result = calculate_text_width("☣", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_u_269b_atom_symbol_should_be_width_2() {
    // ported: calculateTextWidth: U+269B atom symbol should be width 2
    let result = calculate_text_width("⚛", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_findrenderclusterinfo_empty_string() {
    // ported: findRenderClusterInfo: empty string
    let result = render_clusters("", 4, false, WidthMethod::Unicode);
    assert_eq!(0, result.len());
}

#[test]
fn req_002_findrenderclusterinfo_ascii_only_returns_empty() {
    // ported: findRenderClusterInfo: ASCII-only returns empty
    let result = render_clusters("hello world", 4, true, WidthMethod::Unicode);
    assert_eq!(0, result.len());
}

#[test]
fn req_002_findrenderclusterinfo_ascii_with_tab() {
    // ported: findRenderClusterInfo: ASCII with tab
    let result = render_clusters("hello\tworld", 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(5, result[0].byte_start);
    assert_eq!(1, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
    assert_eq!(5, result[0].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_multiple_tabs() {
    // ported: findRenderClusterInfo: multiple tabs
    let result = render_clusters("a\tb\tc", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result.len());
    assert_eq!(1, result[0].byte_start);
    assert_eq!(1, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
    assert_eq!(1, result[0].col_start);
    assert_eq!(3, result[1].byte_start);
    assert_eq!(1, result[1].byte_len);
    assert_eq!(4, result[1].width_cols);
    assert_eq!(6, result[1].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_cjk_characters() {
    // ported: findRenderClusterInfo: CJK characters
    let text = "hello世界";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(2, result.len());
    assert_eq!(5, result[0].byte_start);
    assert_eq!(3, result[0].byte_len);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(5, result[0].col_start);
    assert_eq!(8, result[1].byte_start);
    assert_eq!(3, result[1].byte_len);
    assert_eq!(2, result[1].width_cols);
    assert_eq!(7, result[1].col_start);
}

#[test]
fn req_003_findrenderclusterinfo_emoji_with_skin_tone() {
    // ported: findRenderClusterInfo: emoji with skin tone
    let text = "Hi👋🏿Bye";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(2, result[0].byte_start);
    assert_eq!(8, result[0].byte_len);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(2, result[0].col_start);
}

#[test]
fn req_003_findrenderclusterinfo_emoji_with_zwj() {
    // ported: findRenderClusterInfo: emoji with ZWJ
    let text = "a👩‍🚀b";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(1, result[0].byte_start);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(1, result[0].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_combining_mark() {
    // ported: findRenderClusterInfo: combining mark
    let text = "cafe\u{0301}";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(3, result[0].byte_start);
    assert_eq!(3, result[0].byte_len);
    assert_eq!(1, result[0].width_cols);
    assert_eq!(3, result[0].col_start);
}

#[test]
fn req_003_findrenderclusterinfo_flag_emoji() {
    // ported: findRenderClusterInfo: flag emoji
    let text = "US🇺🇸";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(2, result[0].byte_start);
    assert_eq!(8, result[0].byte_len);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(2, result[0].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_mixed_content() {
    // ported: findRenderClusterInfo: mixed content
    let text = "Hi\t世界!";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(3, result.len());
    assert_eq!(2, result[0].byte_start);
    assert_eq!(1, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
    assert_eq!(2, result[0].col_start);
    assert_eq!(3, result[1].byte_start);
    assert_eq!(3, result[1].byte_len);
    assert_eq!(2, result[1].width_cols);
    assert_eq!(6, result[1].col_start);
    assert_eq!(6, result[2].byte_start);
    assert_eq!(3, result[2].byte_len);
    assert_eq!(2, result[2].width_cols);
    assert_eq!(8, result[2].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_only_ascii_letters_no_cache() {
    // ported: findRenderClusterInfo: only ASCII letters no cache
    let result = render_clusters("abcdefghij", 4, false, WidthMethod::Unicode);
    assert_eq!(0, result.len());
}

#[test]
fn req_003_findrenderclusterinfo_emoji_with_vs16() {
    // ported: findRenderClusterInfo: emoji with VS16
    let text = "I ❤️ U";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(2, result[0].byte_start);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(2, result[0].col_start);
}

#[test]
fn req_002_findrenderclusterinfo_realistic_text() {
    // ported: findRenderClusterInfo: realistic text
    let text = "function test() {\n\tconst 世界 = 10;\n}";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(3, result.len());
}

#[test]
fn req_002_findrenderclusterinfo_hiragana() {
    // ported: findRenderClusterInfo: hiragana
    let text = "こんにちは";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(5, result.len());
    assert_eq!(0, result[0].byte_start);
    assert_eq!(3, result[0].byte_len);
    assert_eq!(2, result[0].width_cols);
}

#[test]
fn req_003_calculatetextwidth_book_and_writing_hand_emojis_width_2() {
    // ported: calculateTextWidth: book and writing hand emojis width 2
    assert_eq!(
        2,
        calculate_text_width("📖", 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("✍️", 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_devanagari_script() {
    // ported: calculateTextWidth: Devanagari script
    let result = calculate_text_width("देवनागरी", 4, false, WidthMethod::Unicode);
    assert_eq!(5, result);
    assert_eq!(
        3,
        calculate_text_width("प्रथम", 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_checkmark_symbol() {
    // ported: calculateTextWidth: checkmark symbol
    let result = calculate_text_width("✓", 4, false, WidthMethod::Unicode);
    assert_eq!(1, result);
}

#[test]
fn req_003_calculatetextwidth_emoji_with_skin_tone() {
    // ported: calculateTextWidth: emoji with skin tone
    let result = calculate_text_width("👋🏿", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_003_calculatetextwidth_emoji_with_zwj() {
    // ported: calculateTextWidth: emoji with ZWJ
    let result = calculate_text_width("👩‍🚀", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_003_calculatetextwidth_emoji_with_vs16_selector() {
    // ported: calculateTextWidth: emoji with VS16 selector
    let result = calculate_text_width("❤️", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_003_calculatetextwidth_flag_emoji() {
    // ported: calculateTextWidth: flag emoji
    let result = calculate_text_width("🇺🇸", 4, false, WidthMethod::Unicode);
    assert_eq!(2, result);
}

#[test]
fn req_002_calculatetextwidth_hiragana_with_tab() {
    // ported: calculateTextWidth: hiragana with tab
    let result = calculate_text_width("こん\tにちは", 4, false, WidthMethod::Unicode);
    assert_eq!(14, result);
}

#[test]
fn req_002_calculatetextwidth_fullwidth_forms_with_tab() {
    // ported: calculateTextWidth: fullwidth forms with tab
    let result = calculate_text_width("ＡＢ\tＣ", 4, false, WidthMethod::Unicode);
    assert_eq!(10, result);
}

#[test]
fn req_002_calculatetextwidth_ascii_fast_path_consistency() {
    // ported: calculateTextWidth: ASCII fast path consistency
    let text_ascii = "hello world";
    let result_fast = calculate_text_width(text_ascii, 4, true, WidthMethod::Unicode);
    let result_slow = calculate_text_width(text_ascii, 4, false, WidthMethod::Unicode);
    assert_eq!(result_fast, result_slow);
}

#[test]
fn req_003_calculatetextwidth_checkmark_grapheme() {
    // ported: calculateTextWidth: checkmark grapheme ✅
    let checkmark = "✅";
    let width = calculate_text_width(checkmark, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_002_calculatetextwidth_sanskrit_text_with_combining_marks() {
    // ported: calculateTextWidth: Sanskrit text with combining marks
    let result = calculate_text_width("संस्कृति", 4, false, WidthMethod::Unicode);
    assert_eq!(4, result);
}

#[test]
fn req_003_calculatetextwidth_checkmark_in_text() {
    // ported: calculateTextWidth: checkmark in text
    let text = "Done ✅";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(7, width);
}

#[test]
fn req_003_calculatetextwidth_complex_graphemes_with_zwj() {
    // ported: calculateTextWidth: complex graphemes with ZWJ
    let woman_astronaut = "👩‍🚀";
    let width = calculate_text_width(woman_astronaut, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_flag_emoji_grapheme() {
    // ported: calculateTextWidth: flag emoji grapheme
    let us_flag = "🇺🇸";
    let width = calculate_text_width(us_flag, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_skin_tone_modifier_grapheme() {
    // ported: calculateTextWidth: skin tone modifier grapheme
    let wave_dark = "👋🏿";
    let width = calculate_text_width(wave_dark, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_presentation_with_vs15_text() {
    // ported: calculateTextWidth: emoji presentation with VS15 (text)
    let heart_text = "❤\u{FE0E}";
    let width = calculate_text_width(heart_text, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_presentation_with_vs16_emoji() {
    // ported: calculateTextWidth: emoji presentation with VS16 (emoji)
    let heart_emoji = "❤️";
    let width = calculate_text_width(heart_emoji, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_keycap_sequences() {
    // ported: calculateTextWidth: keycap sequences
    let keycap_1 = "1️⃣";
    let keycap_hash = "#️⃣";
    assert_eq!(
        2,
        calculate_text_width(keycap_1, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(keycap_hash, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_family_zwj_sequences() {
    // ported: calculateTextWidth: family ZWJ sequences
    let family = "👨‍👩‍👧‍👦";
    let width = calculate_text_width(family, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_profession_zwj_sequences() {
    // ported: calculateTextWidth: profession ZWJ sequences
    let health_worker = "👩‍⚕️";
    let firefighter = "👨‍🚒";
    let teacher = "👩‍🏫";
    assert_eq!(
        2,
        calculate_text_width(health_worker, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(firefighter, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(teacher, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_couple_zwj_sequences() {
    // ported: calculateTextWidth: couple ZWJ sequences
    let kiss = "💏";
    let couple_with_heart = "👩‍❤️‍👨";
    assert_eq!(
        2,
        calculate_text_width(kiss, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(couple_with_heart, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_all_skin_tone_modifiers() {
    // ported: calculateTextWidth: all skin tone modifiers
    let wave_light = "👋🏻";
    let wave_medium_light = "👋🏼";
    let wave_medium = "👋🏽";
    let wave_medium_dark = "👋🏾";
    let wave_dark = "👋🏿";
    assert_eq!(
        2,
        calculate_text_width(wave_light, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(wave_medium_light, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(wave_medium, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(wave_medium_dark, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(wave_dark, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_skin_tone_with_zwj() {
    // ported: calculateTextWidth: skin tone with ZWJ
    let family_skin_tones = "👨🏿‍👩🏻‍👶";
    let width = calculate_text_width(family_skin_tones, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_various_flag_emojis() {
    // ported: calculateTextWidth: various flag emojis
    let flag_us = "🇺🇸";
    let flag_uk = "🇬🇧";
    let flag_jp = "🇯🇵";
    let flag_de = "🇩🇪";
    let flag_fr = "🇫🇷";
    assert_eq!(
        2,
        calculate_text_width(flag_us, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(flag_uk, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(flag_jp, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(flag_de, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(flag_fr, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_multiple_flags_in_text() {
    // ported: calculateTextWidth: multiple flags in text
    let text = "Flags: 🇺🇸 🇬🇧 🇯🇵";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(15, width);
}

#[test]
fn req_002_calculatetextwidth_devanagari_basic_characters() {
    // ported: calculateTextWidth: Devanagari basic characters
    let namaste = "नमस्ते";
    let width = calculate_text_width(namaste, 4, false, WidthMethod::Unicode);
    assert!(width > 0);
}

#[test]
fn req_002_calculatetextwidth_devanagari_with_combining_marks() {
    // ported: calculateTextWidth: Devanagari with combining marks
    let ka = "क";
    let ki = "कि";
    let kii = "की";
    assert_eq!(1, calculate_text_width(ka, 4, false, WidthMethod::Unicode));
    assert_eq!(1, calculate_text_width(ki, 4, false, WidthMethod::Unicode));
    assert_eq!(1, calculate_text_width(kii, 4, false, WidthMethod::Unicode));
}

#[test]
fn req_002_calculatetextwidth_devanagari_conjuncts() {
    // ported: calculateTextWidth: Devanagari conjuncts
    let kta = "क्त";
    let jna = "ज्ञ";
    let ksha = "क्‍ष";
    assert_eq!(2, calculate_text_width(kta, 4, false, WidthMethod::Unicode));
    assert_eq!(2, calculate_text_width(jna, 4, false, WidthMethod::Unicode));
    assert_eq!(
        2,
        calculate_text_width(ksha, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_bengali_script() {
    // ported: calculateTextWidth: Bengali script
    let bangla = "বাংলা";
    let width = calculate_text_width(bangla, 4, false, WidthMethod::Unicode);
    assert!(width > 0);
}

#[test]
fn req_002_calculatetextwidth_tamil_script() {
    // ported: calculateTextWidth: Tamil script
    let tamil = "தமிழ்";
    let width = calculate_text_width(tamil, 4, false, WidthMethod::Unicode);
    assert!(width > 0);
}

#[test]
fn req_002_calculatetextwidth_telugu_script() {
    // ported: calculateTextWidth: Telugu script
    let telugu = "తెలుగు";
    let width = calculate_text_width(telugu, 4, false, WidthMethod::Unicode);
    assert!(width > 0);
}

#[test]
fn req_002_calculatetextwidth_arabic_basic_text() {
    // ported: calculateTextWidth: Arabic basic text
    let arabic = "مرحبا";
    let width = calculate_text_width(arabic, 4, false, WidthMethod::Unicode);
    assert!(width >= 5);
}

#[test]
fn req_002_calculatetextwidth_arabic_with_diacritics() {
    // ported: calculateTextWidth: Arabic with diacritics
    let with_diacritics = "مَرْحَبًا";
    let width = calculate_text_width(with_diacritics, 4, false, WidthMethod::Unicode);
    assert!(width >= 5);
}

#[test]
fn req_002_calculatetextwidth_hebrew_text() {
    // ported: calculateTextWidth: Hebrew text
    let hebrew = "שלום";
    let width = calculate_text_width(hebrew, 4, false, WidthMethod::Unicode);
    assert!(width >= 4);
}

#[test]
fn req_002_calculatetextwidth_chinese_traditional_characters() {
    // ported: calculateTextWidth: Chinese traditional characters
    let traditional = "繁體中文";
    let width = calculate_text_width(traditional, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_002_calculatetextwidth_chinese_simplified_characters() {
    // ported: calculateTextWidth: Chinese simplified characters
    let simplified = "简体中文";
    let width = calculate_text_width(simplified, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_002_calculatetextwidth_japanese_mixed_scripts() {
    // ported: calculateTextWidth: Japanese mixed scripts
    let mixed = "ひらがな漢字カタカナ";
    let width = calculate_text_width(mixed, 4, false, WidthMethod::Unicode);
    assert_eq!(20, width);
}

#[test]
fn req_002_calculatetextwidth_korean_hangul_syllables() {
    // ported: calculateTextWidth: Korean Hangul syllables
    let korean = "한글";
    let width = calculate_text_width(korean, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_cjk_with_ascii() {
    // ported: calculateTextWidth: CJK with ASCII
    let mixed = "Hello世界World";
    let width = calculate_text_width(mixed, 4, false, WidthMethod::Unicode);
    assert_eq!(14, width);
}

#[test]
fn req_002_calculatetextwidth_multiple_combining_marks_on_one_base() {
    // ported: calculateTextWidth: multiple combining marks on one base
    let multiple = "e\u{0301}\u{0302}\u{0304}";
    let width = calculate_text_width(multiple, 4, false, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_002_calculatetextwidth_combining_enclosing_marks() {
    // ported: calculateTextWidth: combining enclosing marks
    let enclosed = "a\u{20E0}";
    let width = calculate_text_width(enclosed, 4, false, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_002_calculatetextwidth_vietnamese_with_multiple_diacritics() {
    // ported: calculateTextWidth: Vietnamese with multiple diacritics
    let vietnamese = "Tiếng Việt";
    let width = calculate_text_width(vietnamese, 4, false, WidthMethod::Unicode);
    assert_eq!(10, width);
}

#[test]
fn req_003_calculatetextwidth_zero_width_joiner_zwj() {
    // ported: calculateTextWidth: zero width joiner (ZWJ)
    let zwj = "\u{200D}";
    let width = calculate_text_width(zwj, 4, false, WidthMethod::Unicode);
    assert_eq!(0, width);
}

#[test]
fn req_002_calculatetextwidth_zero_width_non_joiner_zwnj() {
    // ported: calculateTextWidth: zero width non-joiner (ZWNJ)
    let zwnj = "ab\u{200C}cd";
    let width = calculate_text_width(zwnj, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_zero_width_space() {
    // ported: calculateTextWidth: zero width space
    let zwsp = "a\u{200B}b\u{200B}c";
    let width = calculate_text_width(zwsp, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_word_joiner() {
    // ported: calculateTextWidth: word joiner
    let word_joiner = "word\u{2060}joiner";
    let width = calculate_text_width(word_joiner, 4, false, WidthMethod::Unicode);
    assert_eq!(10, width);
}

#[test]
fn req_002_calculatetextwidth_various_unicode_spaces() {
    // ported: calculateTextWidth: various Unicode spaces
    let en_space = "a\u{2002}b";
    let em_space = "a\u{2003}b";
    let thin_space = "a\u{2009}b";
    let hair_space = "a\u{200A}b";
    let ideo_space = "a\u{3000}b";
    assert_eq!(
        3,
        calculate_text_width(en_space, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        3,
        calculate_text_width(em_space, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        3,
        calculate_text_width(thin_space, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        3,
        calculate_text_width(hair_space, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        4,
        calculate_text_width(ideo_space, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_non_breaking_spaces() {
    // ported: calculateTextWidth: non-breaking spaces
    let nbsp = "a\u{00A0}b";
    let narrow_nbsp = "a\u{202F}b";
    assert_eq!(
        3,
        calculate_text_width(nbsp, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        3,
        calculate_text_width(narrow_nbsp, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_calculatetextwidth_emoji_with_multiple_modifiers() {
    // ported: calculateTextWidth: emoji with multiple modifiers
    let rainbow_flag = "🏴‍🌈";
    let width = calculate_text_width(rainbow_flag, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_tag_sequences_subdivision_flags() {
    // ported: calculateTextWidth: emoji tag sequences (subdivision flags)
    let black_flag = "🏴";
    assert_eq!(
        2,
        calculate_text_width(black_flag, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_hair_style_variations() {
    // ported: calculateTextWidth: hair style variations
    let red_hair = "👩‍🦰";
    let curly_hair = "👨‍🦱";
    let white_hair = "👩‍🦳";
    let bald = "👨‍🦲";
    assert_eq!(
        2,
        calculate_text_width(red_hair, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(curly_hair, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(white_hair, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width(bald, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_multilingual_sentence() {
    // ported: calculateTextWidth: multilingual sentence
    let text = "Hello 世界! مرحبا 👋";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert!(width >= 18);
}

#[test]
fn req_003_calculatetextwidth_code_with_emoji_comments() {
    // ported: calculateTextWidth: code with emoji comments
    let code = "const x = 42; // ✅ works";
    let width = calculate_text_width(code, 4, false, WidthMethod::Unicode);
    assert_eq!(25, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_sentence() {
    // ported: calculateTextWidth: emoji sentence
    let text = "I ❤️ 🍕 and 🍣!";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(15, width);
}

#[test]
fn req_002_calculatetextwidth_social_media_style_text() {
    // ported: calculateTextWidth: social media style text
    let text = "#OpenTUI 🚀 is #awesome 💯!";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(27, width);
}

#[test]
fn req_002_calculatetextwidth_surrogate_pair_edge_cases() {
    // ported: calculateTextWidth: surrogate pair edge cases
    let emoji = "𝕳𝖊𝖑𝖑𝖔";
    let width = calculate_text_width(emoji, 4, false, WidthMethod::Unicode);
    assert_eq!(5, width);
}

#[test]
fn req_003_calculatetextwidth_all_emoji_skin_tones_in_sequence() {
    // ported: calculateTextWidth: all emoji skin tones in sequence
    let text = "👋🏻👋🏼👋🏽👋🏾👋🏿";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(10, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_zodiac_signs() {
    // ported: calculateTextWidth: emoji zodiac signs
    let zodiac = "♈♉♊♋♌♍♎♏♐♑♒♓";
    let width = calculate_text_width(zodiac, 4, false, WidthMethod::Unicode);
    assert_eq!(24, width);
}

#[test]
fn req_002_calculatetextwidth_mathematical_symbols() {
    // ported: calculateTextWidth: mathematical symbols
    let math = "∀∃∈∉∋∑∏∫∂∇≠≤≥";
    let width = calculate_text_width(math, 4, false, WidthMethod::Unicode);
    assert!(width >= 13);
}

#[test]
fn req_002_calculatetextwidth_box_drawing_characters() {
    // ported: calculateTextWidth: box drawing characters
    let box_v = "┌─┐│└─┘";
    let width = calculate_text_width(box_v, 4, false, WidthMethod::Unicode);
    assert_eq!(7, width);
}

#[test]
fn req_002_calculatetextwidth_braille_patterns() {
    // ported: calculateTextWidth: braille patterns
    let braille = "⠀⠁⠂⠃⠄⠅⠆⠇";
    let width = calculate_text_width(braille, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_002_calculatetextwidth_musical_symbols() {
    // ported: calculateTextWidth: musical symbols
    let music = "𝄞𝄢𝅘𝅥𝅮";
    let width = calculate_text_width(music, 4, false, WidthMethod::Unicode);
    let _ = width; // reference asserts crash-freedom only
}

#[test]
fn req_003_calculatetextwidth_weather_and_nature_emoji() {
    // ported: calculateTextWidth: weather and nature emoji
    let weather = "☀️🌤️⛅🌦️🌧️⛈️";
    let width = calculate_text_width(weather, 4, false, WidthMethod::Unicode);
    assert_eq!(12, width);
}

#[test]
fn req_003_calculatetextwidth_food_emoji_collection() {
    // ported: calculateTextWidth: food emoji collection
    let food = "🍎🍌🍇🍓🥕🥦🍞🧀";
    let width = calculate_text_width(food, 4, false, WidthMethod::Unicode);
    assert_eq!(16, width);
}

#[test]
fn req_003_calculatetextwidth_animal_emoji() {
    // ported: calculateTextWidth: animal emoji
    let animals = "🐶🐱🐭🐹🐰🦊🐻🐼";
    let width = calculate_text_width(animals, 4, false, WidthMethod::Unicode);
    assert_eq!(16, width);
}

#[test]
fn req_002_calculatetextwidth_realistic_chat_message() {
    // ported: calculateTextWidth: realistic chat message
    let message =
        "Hey! 👋 Can you review my PR? 🙏 It fixes the bug 🐛 we discussed earlier. Thanks! 😊";
    let width = calculate_text_width(message, 4, false, WidthMethod::Unicode);
    assert!(width > 70);
}

#[test]
fn req_002_calculatetextwidth_empty_string_with_tabs() {
    // ported: calculateTextWidth: empty string with tabs
    let text = "";
    assert_eq!(
        0,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        0,
        calculate_text_width(text, 8, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_calculatetextwidth_only_combining_marks_invalid_but_should_not_crash() {
    // ported: calculateTextWidth: only combining marks (invalid but should not crash)
    let text = "\u{0301}\u{0302}\u{0303}";
    let width = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let _ = width; // reference asserts crash-freedom only
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_celestial_and_symbols() {
    // ported: calculateTextWidth: emoji collection - celestial and symbols
    let celestial = "🌟🔮✨";
    let width = calculate_text_width(celestial, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_religious_and_gestures() {
    // ported: calculateTextWidth: emoji collection - religious and gestures
    let religious = "🙏";
    let width = calculate_text_width(religious, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_zwj_sequences_astronauts() {
    // ported: calculateTextWidth: emoji collection - ZWJ sequences astronauts
    let astronauts = "🧑‍🚀👨‍🚀👩‍🚀";
    let width = calculate_text_width(astronauts, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_rainbow_and_magical_creatures() {
    // ported: calculateTextWidth: emoji collection - rainbow and magical creatures
    let magical = "🌈🦄🧚‍♀️";
    let width = calculate_text_width(magical, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_books_and_writing() {
    // ported: calculateTextWidth: emoji collection - books and writing
    let writing = "📜📖✍️";
    let width = calculate_text_width(writing, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_japanese_culture() {
    // ported: calculateTextWidth: emoji collection - Japanese culture
    let japanese = "🏯🎋🌸";
    let width = calculate_text_width(japanese, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_traditional_japanese_items() {
    // ported: calculateTextWidth: emoji collection - traditional Japanese items
    let traditional = "📯🎴🎎";
    let width = calculate_text_width(traditional, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_hearts_and_peace() {
    // ported: calculateTextWidth: emoji collection - hearts and peace
    let peace = "💝🕊️☮️";
    let width = calculate_text_width(peace, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_meditation_and_nature() {
    // ported: calculateTextWidth: emoji collection - meditation and nature
    let meditation = "🧘‍♂️🌳";
    let width = calculate_text_width(meditation, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_food_and_drink() {
    // ported: calculateTextWidth: emoji collection - food and drink
    let food = "🍵🥟";
    let width = calculate_text_width(food, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_exotic_animals() {
    // ported: calculateTextWidth: emoji collection - exotic animals
    let animals = "🦥🦦🦧🦨🦩🦚🦜🦝🦞🦟";
    let width = calculate_text_width(animals, 4, false, WidthMethod::Unicode);
    assert_eq!(20, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_communication() {
    // ported: calculateTextWidth: emoji collection - communication
    let communication = "🤫🗣️💬";
    let width = calculate_text_width(communication, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_water_and_nature() {
    // ported: calculateTextWidth: emoji collection - water and nature
    let nature = "🌊📝🎭";
    let width = calculate_text_width(nature, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_landscape() {
    // ported: calculateTextWidth: emoji collection - landscape
    let landscape = "🏞️🌊💧";
    let width = calculate_text_width(landscape, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_circus_and_art() {
    // ported: calculateTextWidth: emoji collection - circus and art
    let circus = "🤹‍♂️🎪🎨";
    let width = calculate_text_width(circus, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_shopping_and_food_items() {
    // ported: calculateTextWidth: emoji collection - shopping and food items
    let shopping = "🏪🛒💰🌶️🧄🧅";
    let width = calculate_text_width(shopping, 4, false, WidthMethod::Unicode);
    assert_eq!(12, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_textiles_and_art() {
    // ported: calculateTextWidth: emoji collection - textiles and art
    let textiles = "🧵👘🎨🖼️";
    let width = calculate_text_width(textiles, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_prehistoric_creatures() {
    // ported: calculateTextWidth: emoji collection - prehistoric creatures
    let prehistoric = "🦖🦕🐉🐲";
    let width = calculate_text_width(prehistoric, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_hand_gestures() {
    // ported: calculateTextWidth: emoji collection - hand gestures
    let hands = "🤝🤲👐";
    let width = calculate_text_width(hands, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_lanterns_and_lights() {
    // ported: calculateTextWidth: emoji collection - lanterns and lights
    let lanterns = "🏮🎆🎇🕯️💡";
    let width = calculate_text_width(lanterns, 4, false, WidthMethod::Unicode);
    assert_eq!(10, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_dancers() {
    // ported: calculateTextWidth: emoji collection - dancers
    let dancers = "💃🕺🩰";
    let width = calculate_text_width(dancers, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_musical_instruments() {
    // ported: calculateTextWidth: emoji collection - musical instruments
    let instruments = "🎻🎺🎷🎸🪕🪘";
    let width = calculate_text_width(instruments, 4, false, WidthMethod::Unicode);
    assert_eq!(12, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_bells_and_shrine() {
    // ported: calculateTextWidth: emoji collection - bells and shrine
    let bells = "🔔⛩️";
    let width = calculate_text_width(bells, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_shocked_and_amazed() {
    // ported: calculateTextWidth: emoji collection - shocked and amazed
    let shocked = "😵‍💫🤯✨";
    let width = calculate_text_width(shocked, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_sweets_and_bubble_tea() {
    // ported: calculateTextWidth: emoji collection - sweets and bubble tea
    let sweets = "🧋🍬🍭🧁";
    let width = calculate_text_width(sweets, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_machinery_and_robots() {
    // ported: calculateTextWidth: emoji collection - machinery and robots
    let machinery = "⚙️🤖🦾🦿";
    let width = calculate_text_width(machinery, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_vehicles() {
    // ported: calculateTextWidth: emoji collection - vehicles
    let vehicles = "🚗🚕🚙🚌🚎";
    let width = calculate_text_width(vehicles, 4, false, WidthMethod::Unicode);
    assert_eq!(10, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_space_travel() {
    // ported: calculateTextWidth: emoji collection - space travel
    let space = "🚀🛸🛰️";
    let width = calculate_text_width(space, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_technology() {
    // ported: calculateTextWidth: emoji collection - technology
    let tech = "🐍💻⌨️";
    let width = calculate_text_width(tech, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_education_and_brain() {
    // ported: calculateTextWidth: emoji collection - education and brain
    let education = "🧠📚🎓";
    let width = calculate_text_width(education, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_professional_zwj_sequences() {
    // ported: calculateTextWidth: emoji collection - professional ZWJ sequences
    let professionals = "👨‍💼👩‍💼👨‍🔬👩‍🔬";
    let width = calculate_text_width(professionals, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_earth_globes() {
    // ported: calculateTextWidth: emoji collection - earth globes
    let globes = "🌍🌎🌏";
    let width = calculate_text_width(globes, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_family_zwj_sequence() {
    // ported: calculateTextWidth: emoji collection - family ZWJ sequence
    let family = "👨‍👩‍👧‍👦";
    let width = calculate_text_width(family, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_elderly_people() {
    // ported: calculateTextWidth: emoji collection - elderly people
    let elderly = "👴👵";
    let width = calculate_text_width(elderly, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_sunrise_and_sunset() {
    // ported: calculateTextWidth: emoji collection - sunrise and sunset
    let sunrise = "🌅🌄🌠";
    let width = calculate_text_width(sunrise, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_mountains() {
    // ported: calculateTextWidth: emoji collection - mountains
    let mountains = "🏔️⛰️🗻";
    let width = calculate_text_width(mountains, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_thoughts_and_dreams() {
    // ported: calculateTextWidth: emoji collection - thoughts and dreams
    let dreams = "💭💤🌌";
    let width = calculate_text_width(dreams, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_campfire() {
    // ported: calculateTextWidth: emoji collection - campfire
    let campfire = "🔥🏕️";
    let width = calculate_text_width(campfire, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_cooking() {
    // ported: calculateTextWidth: emoji collection - cooking
    let cooking = "🍛🍲🥘";
    let width = calculate_text_width(cooking, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_love_hearts() {
    // ported: calculateTextWidth: emoji collection - love hearts
    let hearts = "❤️💕💖";
    let width = calculate_text_width(hearts, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_media() {
    // ported: calculateTextWidth: emoji collection - media
    let media = "📸🎞️📹";
    let width = calculate_text_width(media, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_global_and_handshake() {
    // ported: calculateTextWidth: emoji collection - global and handshake
    let global = "🌐🤝🌈";
    let width = calculate_text_width(global, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_special_symbols() {
    // ported: calculateTextWidth: emoji collection - special symbols
    let special = "🦩🧿🪬🫀🫁🧠";
    let width = calculate_text_width(special, 4, false, WidthMethod::Unicode);
    assert_eq!(12, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_strength() {
    // ported: calculateTextWidth: emoji collection - strength
    let strength = "💪✊🙌";
    let width = calculate_text_width(strength, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_003_calculatetextwidth_emoji_collection_entertainment() {
    // ported: calculateTextWidth: emoji collection - entertainment
    let entertainment = "🎬🎭🎪✨🌟⭐";
    let width = calculate_text_width(entertainment, 4, false, WidthMethod::Unicode);
    assert_eq!(12, width);
}

#[test]
fn req_002_calculatetextwidth_devanagari_sanskrit_word() {
    // ported: calculateTextWidth: Devanagari - Sanskrit word
    let sanskrit = "संस्कृति";
    let width = calculate_text_width(sanskrit, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_devanagari_namaste() {
    // ported: calculateTextWidth: Devanagari - namaste
    let namaste = "नमस्ते";
    let width = calculate_text_width(namaste, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_devanagari_om_symbol() {
    // ported: calculateTextWidth: Devanagari - Om symbol
    let om = "ॐ";
    let width = calculate_text_width(om, 4, false, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_002_calculatetextwidth_devanagari_mixed_with_ascii() {
    // ported: calculateTextWidth: Devanagari - mixed with ASCII
    let mixed = "Hello नमस्ते World";
    let width = calculate_text_width(mixed, 4, false, WidthMethod::Unicode);
    assert_eq!(16, width);
}

#[test]
fn req_002_calculatetextwidth_chinese_characters_kanji() {
    // ported: calculateTextWidth: Chinese characters - kanji
    let kanji = "漢字";
    let width = calculate_text_width(kanji, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_hiragana() {
    // ported: calculateTextWidth: Hiragana
    let hiragana = "ひらがな";
    let width = calculate_text_width(hiragana, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_002_calculatetextwidth_katakana() {
    // ported: calculateTextWidth: Katakana
    let katakana = "カタカナ";
    let width = calculate_text_width(katakana, 4, false, WidthMethod::Unicode);
    assert_eq!(8, width);
}

#[test]
fn req_002_calculatetextwidth_korean_hangul() {
    // ported: calculateTextWidth: Korean Hangul
    let hangul = "한글";
    let width = calculate_text_width(hangul, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_korean_words_love_and_peace() {
    // ported: calculateTextWidth: Korean words - love and peace
    let korean = "사랑 평화";
    let width = calculate_text_width(korean, 4, false, WidthMethod::Unicode);
    assert_eq!(9, width);
}

#[test]
fn req_002_calculatetextwidth_tibetan_script() {
    // ported: calculateTextWidth: Tibetan script
    let tibetan = "རྒྱ་མཚོ";
    let width = calculate_text_width(tibetan, 4, false, WidthMethod::Unicode);
    assert!((3..=width).contains(&width));
}

#[test]
fn req_002_calculatetextwidth_gujarati_script() {
    // ported: calculateTextWidth: Gujarati script
    let gujarati = "ગુજરાતી";
    let width = calculate_text_width(gujarati, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_tamil_script_word() {
    // ported: calculateTextWidth: Tamil script word
    let tamil = "தமிழ்";
    let width = calculate_text_width(tamil, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_punjabi_script_word() {
    // ported: calculateTextWidth: Punjabi script word
    let punjabi = "ਪੰਜਾਬੀ";
    let width = calculate_text_width(punjabi, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_telugu_script_word() {
    // ported: calculateTextWidth: Telugu script word
    let telugu = "తెలుగు";
    let width = calculate_text_width(telugu, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_bengali_script_word() {
    // ported: calculateTextWidth: Bengali script word
    let bengali = "বাংলা";
    let width = calculate_text_width(bengali, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_002_calculatetextwidth_kannada_script() {
    // ported: calculateTextWidth: Kannada script
    let kannada = "ಕನ್ನಡ";
    let width = calculate_text_width(kannada, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_malayalam_script() {
    // ported: calculateTextWidth: Malayalam script
    let malayalam = "മലയാളം";
    let width = calculate_text_width(malayalam, 4, false, WidthMethod::Unicode);
    assert!((4..=width).contains(&width));
}

#[test]
fn req_002_calculatetextwidth_malayalam_report_matches_ghostty_grapheme_widths() {
    // ported: calculateTextWidth: Malayalam report matches Ghostty grapheme widths
    let report = "OpenCode search configuration പരിശോധിക്കൽ";
    assert_eq!(
        36,
        calculate_text_width(report, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("രി", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ശോ", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ധി", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ക്ക", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        40,
        calculate_text_width(report, 4, false, WidthMethod::UnicodeWide)
    );
}

#[test]
fn req_002_calculatetextwidth_oriya_script() {
    // ported: calculateTextWidth: Oriya script
    let oriya = "ଓଡ଼ିଆ";
    let width = calculate_text_width(oriya, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_thai_script() {
    // ported: calculateTextWidth: Thai script
    let thai = "ภาษา";
    let width = calculate_text_width(thai, 4, false, WidthMethod::Unicode);
    assert!((3..=width).contains(&width));
}

#[test]
fn req_002_calculatetextwidth_thai_numerals() {
    // ported: calculateTextWidth: Thai numerals
    let thai_num = "๑๐๐";
    let width = calculate_text_width(thai_num, 4, false, WidthMethod::Unicode);
    assert_eq!(3, width);
}

#[test]
fn req_002_calculatetextwidth_lao_script() {
    // ported: calculateTextWidth: Lao script
    let lao = "ໂຫຍ່າກເຈົ້າ";
    let width = calculate_text_width(lao, 4, false, WidthMethod::Unicode);
    assert!((5..=width).contains(&width));
}

#[test]
fn req_002_calculatetextwidth_arabic_character() {
    // ported: calculateTextWidth: Arabic character
    let arabic = "ا";
    let width = calculate_text_width(arabic, 4, false, WidthMethod::Unicode);
    assert_eq!(1, width);
}

#[test]
fn req_002_calculatetextwidth_sinhala_script() {
    // ported: calculateTextWidth: Sinhala script
    let sinhala = "ආහාර";
    let width = calculate_text_width(sinhala, 4, false, WidthMethod::Unicode);
    assert!((3..=width).contains(&width));
}

#[test]
fn req_002_calculatetextwidth_chinese_text() {
    // ported: calculateTextWidth: Chinese text
    let chinese = "中文";
    let width = calculate_text_width(chinese, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width);
}

#[test]
fn req_002_calculatetextwidth_hangul_jamo() {
    // ported: calculateTextWidth: Hangul Jamo
    let jamo = "ㄱ";
    let width = calculate_text_width(jamo, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width);
}

#[test]
fn req_002_calculatetextwidth_realistic_multilingual_sentence() {
    // ported: calculateTextWidth: realistic multilingual sentence
    let multilingual = "Hello 世界! नमस्ते 🙏";
    let width = calculate_text_width(multilingual, 4, false, WidthMethod::Unicode);
    assert_eq!(19, width);
}

#[test]
fn req_002_calculatetextwidth_all_ending_words_from_text() {
    // ported: calculateTextWidth: all ending words from text
    let endings = "समाप्त끝จบముగింపుಅಂತ್ಯઅંત";
    let width = calculate_text_width(endings, 4, false, WidthMethod::Unicode);
    assert!(width > 10);
}

#[test]
fn req_003_calculatetextwidth_complex_text_with_emojis_and_multiple_scripts() {
    // ported: calculateTextWidth: complex text with emojis and multiple scripts
    let complex = "The 🌟 journey: संस्कृति meets 漢字 🎋";
    let width = calculate_text_width(complex, 4, false, WidthMethod::Unicode);
    assert!((30..=width).contains(&width));
}

#[test]
fn req_002_thai_base_consonants_have_width_1() {
    // ported: Thai: base consonants have width 1
    let consonants = "กขคงจฉชซญฎฏฐดตถทธนบปผฝพฟภมยรลวศษสหอฮ";
    let width = calculate_text_width(consonants, 4, false, WidthMethod::Unicode);
    assert_eq!(36, width);
}

#[test]
fn req_002_thai_spacing_vowels_have_width_1() {
    // ported: Thai: spacing vowels have width 1
    let spacing_vowels = "าะแโใไ";
    let width = calculate_text_width(spacing_vowels, 4, false, WidthMethod::Unicode);
    assert_eq!(6, width);
}

#[test]
fn req_002_thai_combining_vowels_above_have_width_0() {
    // ported: Thai: combining vowels above have width 0
    let base = "ก";
    let with_sara_i = "กิ";
    let with_sara_ii = "กี";
    let with_sara_ue = "กึ";
    let with_sara_uee = "กื";
    let with_mai_han_akat = "กั";
    assert_eq!(
        1,
        calculate_text_width(base, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_sara_i, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_sara_ii, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_sara_ue, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_sara_uee, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_mai_han_akat, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_combining_vowels_below_have_width_0() {
    // ported: Thai: combining vowels below have width 0
    let with_sara_u = "กุ";
    let with_sara_uu = "กู";
    assert_eq!(
        1,
        calculate_text_width(with_sara_u, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_sara_uu, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_tone_marks_have_width_0() {
    // ported: Thai: tone marks have width 0
    let with_mai_ek = "ก่";
    let with_mai_tho = "ก้";
    let with_mai_tri = "ก๊";
    let with_mai_chattawa = "ก๋";
    assert_eq!(
        1,
        calculate_text_width(with_mai_ek, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_mai_tho, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_mai_tri, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_mai_chattawa, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_other_diacritics_have_width_0() {
    // ported: Thai: other diacritics have width 0
    let with_maitaikhu = "ก็";
    let with_thanthakhat = "ก์";
    let with_nikhahit = "กํ";
    assert_eq!(
        1,
        calculate_text_width(with_maitaikhu, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_thanthakhat, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        1,
        calculate_text_width(with_nikhahit, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_combined_vowel_and_tone_mark() {
    // ported: Thai: combined vowel and tone mark
    let text = "กี่";
    assert_eq!(
        1,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
    let text2 = "คือ";
    assert_eq!(
        2,
        calculate_text_width(text2, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_word_thai_language() {
    // ported: Thai: word 'ภาษาไทย' (Thai language)
    let text = "ภาษาไทย";
    assert_eq!(
        7,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_word_with_tone_mark() {
    // ported: Thai: word 'อย่าง' with tone mark
    let text = "อย่าง";
    assert_eq!(
        4,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_word_with_vowel_above() {
    // ported: Thai: word 'อธิบาย' with vowel above
    let text = "อธิบาย";
    assert_eq!(
        5,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_full_sentence_with_spaces() {
    // ported: Thai: full sentence with spaces
    let text = "ภาษาไทย คืออะไร อธิบายมาอย่างละเอียด";
    assert_eq!(
        32,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_render_cluster_info_for_combining_marks() {
    // ported: Thai: render-cluster info for combining marks
    let text = "กี่";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(1, result[0].width_cols);
}

#[test]
fn req_002_thai_render_cluster_info_for_word_with_combining_marks() {
    // ported: Thai: render-cluster info for word with combining marks
    let text = "คือ";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(2, result.len());
    assert_eq!(1, result[0].width_cols);
    assert_eq!(1, result[1].width_cols);
}

#[test]
fn req_002_thai_mixed_thai_and_ascii() {
    // ported: Thai: mixed Thai and ASCII
    let text = "Hello ภาษาไทย World";
    assert_eq!(
        19,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_thai_mixed_thai_and_emoji() {
    // ported: Thai: mixed Thai and emoji
    let text = "ภาษา 🇹🇭 ไทย";
    assert_eq!(
        11,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_width_should_be_4() {
    // ported: Thai: คำว่า width should be 4
    let text = "คำว่า";
    assert_eq!(
        4,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_width_should_be_2() {
    // ported: Thai: น้ำ width should be 2
    let text = "น้ำ";
    assert_eq!(
        2,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_thai_width_should_be_1() {
    // ported: Thai: ว่ width should be 1
    let text = "ว่";
    assert_eq!(
        1,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_001_thai_wcwidth_vs_unicode_mode_comparison() {
    // ported: Thai: ว่ wcwidth vs unicode mode comparison
    let text = "ว่";
    let wcwidth_result = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    let unicode_result = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, wcwidth_result);
    assert_eq!(1, unicode_result);
}

#[test]
fn req_002_thai_is_a_single_grapheme_cluster() {
    // ported: Thai: ว่ is a single grapheme cluster
    let text = "ว่";
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result.len());
    assert_eq!(1, result[0].width_cols);
}

#[test]
fn req_001_no_zwj_basic_emoji_zwj_sequence_split() {
    // ported: no_zwj: basic emoji ZWJ sequence split
    let text = "👩‍🚀";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width_unicode);
    assert_eq!(4, width_no_zwj);
    assert_eq!(4, width_wcwidth);
}

#[test]
fn req_001_no_zwj_family_emoji_split() {
    // ported: no_zwj: family emoji split
    let text = "👨‍👩‍👧";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(2, width_unicode);
    assert_eq!(6, width_no_zwj);
}

#[test]
fn req_001_no_zwj_combining_marks_still_combined() {
    // ported: no_zwj: combining marks still combined
    let text = "é";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(1, width_unicode);
    assert_eq!(1, width_no_zwj);
}

#[test]
fn req_001_no_zwj_skin_tone_modifiers_still_combined() {
    // ported: no_zwj: skin tone modifiers still combined
    let text = "👋🏿";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width_unicode);
    assert_eq!(2, width_no_zwj);
    assert_eq!(4, width_wcwidth);
}

#[test]
fn req_001_no_zwj_flag_emoji_stays_combined() {
    // ported: no_zwj: flag emoji stays combined
    let text = "🇺🇸";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width_unicode);
    assert_eq!(2, width_no_zwj);
    assert_eq!(2, width_wcwidth);
}

#[test]
fn req_001_no_zwj_mixed_text_with_zwj_emoji() {
    // ported: no_zwj: mixed text with ZWJ emoji
    let text = "Hello👩‍🚀World";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(12, width_unicode);
    assert_eq!(14, width_no_zwj);
}

#[test]
fn req_001_no_zwj_findrenderclusterinfo_splits_zwj_sequences() {
    // ported: no_zwj: findRenderClusterInfo splits ZWJ sequences
    let text = "Hi👩‍🚀Bye";
    let result_unicode = render_clusters(text, 4, false, WidthMethod::Unicode);
    let result_no_zwj = render_clusters(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(1, result_unicode.len());
    assert_eq!(2, result_unicode[0].width_cols);
    assert_eq!(2, result_no_zwj.len());
    assert_eq!(2, result_no_zwj[0].width_cols);
    assert_eq!(2, result_no_zwj[1].width_cols);
}

#[test]
fn req_001_no_zwj_getwidthat_with_zwj_sequence() {
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
fn req_001_no_zwj_multiple_zwj_sequences() {
    // ported: no_zwj: multiple ZWJ sequences
    let text = "👨‍👩‍👧👨‍👩‍👦";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(4, width_unicode);
    assert_eq!(12, width_no_zwj);
}

#[test]
fn req_001_no_zwj_zwj_with_skin_tones() {
    // ported: no_zwj: ZWJ with skin tones
    let text = "👨🏿‍❤️‍👨🏻";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(2, width_unicode);
    assert_eq!(6, width_no_zwj);
}

#[test]
fn req_001_no_zwj_keycap_sequences_without_zwj() {
    // ported: no_zwj: keycap sequences without ZWJ
    let text = "1️⃣";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width_unicode);
    assert_eq!(2, width_no_zwj);
    assert_eq!(1, width_wcwidth);
}

#[test]
fn req_001_no_zwj_rainbow_flag_without_zwj() {
    // ported: no_zwj: rainbow flag without ZWJ
    let text = "🏳️‍🌈";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    assert_eq!(2, width_unicode);
    assert_eq!(4, width_no_zwj);
}

#[test]
fn req_001_no_zwj_devanagari_conjuncts_still_work() {
    // ported: no_zwj: Devanagari conjuncts still work
    let text = "क्ष";
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    let width_no_zwj = calculate_text_width(text, 4, false, WidthMethod::NoZwj);
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width_unicode);
    assert_eq!(2, width_no_zwj);
    assert_eq!(2, width_wcwidth);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_empty_string() {
    // ported: findRenderClusterInfo wcwidth: empty string
    let result = render_clusters("", 4, false, WidthMethod::Wcwidth);
    assert_eq!(0, result.len());
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_ascii_only_returns_empty() {
    // ported: findRenderClusterInfo wcwidth: ASCII-only returns empty
    let result = render_clusters("hello world", 4, true, WidthMethod::Wcwidth);
    assert_eq!(0, result.len());
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_ascii_with_tab() {
    // ported: findRenderClusterInfo wcwidth: ASCII with tab
    let result = render_clusters("hello\tworld", 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, result.len());
    assert_eq!(5, result[0].byte_start);
    assert_eq!(1, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
    assert_eq!(5, result[0].col_start);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_cjk_characters() {
    // ported: findRenderClusterInfo wcwidth: CJK characters
    let text = "hello世界";
    let result = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, result.len());
    assert_eq!(5, result[0].byte_start);
    assert_eq!(3, result[0].byte_len);
    assert_eq!(2, result[0].width_cols);
    assert_eq!(5, result[0].col_start);
    assert_eq!(8, result[1].byte_start);
    assert_eq!(3, result[1].byte_len);
    assert_eq!(2, result[1].width_cols);
    assert_eq!(7, result[1].col_start);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_emoji_with_skin_tone_single_grapheme_cluster() {
    // ported: findRenderClusterInfo wcwidth: emoji with skin tone - single grapheme cluster
    let text = "👋🏿";
    let result = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, result.len());
    assert_eq!(0, result[0].byte_start);
    assert_eq!(8, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_emoji_with_zwj_single_grapheme_cluster() {
    // ported: findRenderClusterInfo wcwidth: emoji with ZWJ - single grapheme cluster
    let text = "👩‍🚀";
    let result = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, result.len());
    assert_eq!(11, result[0].byte_len);
    assert_eq!(4, result[0].width_cols);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_combining_mark_part_of_base_grapheme() {
    // ported: findRenderClusterInfo wcwidth: combining mark - part of base grapheme
    let text = "e\u{0301}test";
    let result = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, result.len());
    assert_eq!(0, result[0].byte_start);
    assert_eq!(3, result[0].byte_len);
    assert_eq!(1, result[0].width_cols);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_vs_unicode_emoji_with_skin_tone() {
    // ported: findRenderClusterInfo wcwidth vs unicode: emoji with skin tone
    let text = "Hi👋🏿Bye";
    let result_wcwidth = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    let result_unicode = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result_wcwidth.len());
    assert_eq!(1, result_unicode.len());
    assert_eq!(2, result_wcwidth[0].byte_start);
    assert_eq!(8, result_wcwidth[0].byte_len);
    assert_eq!(2, result_unicode[0].byte_start);
    assert_eq!(8, result_unicode[0].byte_len);
    assert_eq!(4, result_wcwidth[0].width_cols);
    assert_eq!(2, result_unicode[0].width_cols);
}

#[test]
fn req_001_findrenderclusterinfo_wcwidth_vs_unicode_flag_emoji() {
    // ported: findRenderClusterInfo wcwidth vs unicode: flag emoji
    let text = "🇺🇸";
    let result_wcwidth = render_clusters(text, 4, false, WidthMethod::Wcwidth);
    let result_unicode = render_clusters(text, 4, false, WidthMethod::Unicode);
    assert_eq!(1, result_wcwidth.len());
    assert_eq!(1, result_unicode.len());
    assert_eq!(2, result_wcwidth[0].width_cols);
    assert_eq!(2, result_unicode[0].width_cols);
}

#[test]
fn req_001_getwidthat_wcwidth_combining_mark_has_zero_width() {
    // ported: getWidthAt wcwidth: combining mark has zero width
    let text = "e\u{0301}";
    let width_e = width_at(text, 0, 8, WidthMethod::Wcwidth);
    assert_eq!(1, width_e);
    let width_combining = width_at(text, 1, 8, WidthMethod::Wcwidth);
    assert_eq!(0, width_combining);
}

#[test]
fn req_001_calculatetextwidth_wcwidth_emoji_with_skin_tone_counts_both_codepoints() {
    // ported: calculateTextWidth wcwidth: emoji with skin tone counts both codepoints
    let text = "👋🏿";
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(4, width_wcwidth);
    assert_eq!(2, width_unicode);
}

#[test]
fn req_001_calculatetextwidth_wcwidth_flag_emoji_counts_both_ris() {
    // ported: calculateTextWidth wcwidth: flag emoji counts both RIs
    let text = "🇺🇸";
    let width_wcwidth = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    let width_unicode = calculate_text_width(text, 4, false, WidthMethod::Unicode);
    assert_eq!(2, width_wcwidth);
    assert_eq!(2, width_unicode);
}

#[test]
fn req_001_wcwidth_zero_width_characters_are_handled_correctly() {
    // ported: wcwidth: zero-width characters are handled correctly
    let text_zwj = "\u{200D}";
    let width_zwj = calculate_text_width(text_zwj, 4, false, WidthMethod::Wcwidth);
    assert_eq!(0, width_zwj);
    let text_combining = "e\u{0301}";
    let width = calculate_text_width(text_combining, 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, width);
}

#[test]
fn req_001_wcwidth_variation_selectors() {
    // ported: wcwidth: variation selectors
    let text_vs16 = "☺\u{FE0F}";
    let width_vs16 = calculate_text_width(text_vs16, 4, false, WidthMethod::Wcwidth);
    assert_eq!(1, width_vs16);
}

#[test]
fn req_001_wcwidth_regional_indicators_counted_separately() {
    // ported: wcwidth: regional indicators counted separately
    let text = "🇺🇸";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(2, width);
}

#[test]
fn req_001_wcwidth_emoji_zwj_sequences_split() {
    // ported: wcwidth: emoji ZWJ sequences split
    let text = "👩‍🚀";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(4, width);
}

#[test]
fn req_001_wcwidth_family_emoji_split_into_components() {
    // ported: wcwidth: family emoji split into components
    let text = "👨‍👩‍👧";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(6, width);
}

#[test]
fn req_001_wcwidth_skin_tone_modifiers_counted_separately() {
    // ported: wcwidth: skin tone modifiers counted separately
    let text = "👋🏻";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(4, width);
}

#[test]
fn req_001_wcwidth_cjk_characters_have_width_2() {
    // ported: wcwidth: CJK characters have width 2
    let text = "你好世界";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(8, width);
}

#[test]
fn req_001_wcwidth_mixed_ascii_and_emoji() {
    // ported: wcwidth: mixed ASCII and emoji
    let text = "Hello👋World";
    let width = calculate_text_width(text, 4, false, WidthMethod::Wcwidth);
    assert_eq!(12, width);
}

#[test]
fn req_001_wcwidth_getwidthat_with_combining_marks() {
    // ported: wcwidth: getWidthAt with combining marks
    let text = "e\u{0301}test";
    let width_e = width_at(text, 0, 4, WidthMethod::Wcwidth);
    assert_eq!(1, width_e);
    let width_combining = width_at(text, 1, 4, WidthMethod::Wcwidth);
    assert_eq!(0, width_combining);
}

// ---- hand-ported: builders, tables, and fixtures the transpiler skips ----

include!("width_map_runs.inc");

#[test]
fn req_002_width_map_runs() {
    // ported: calculateTextWidth: validate against unicode-width-map.zon
    for (start, end, want) in WIDTH_MAP_RUNS {
        for cp in *start..=*end {
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            assert_eq!(
                *want,
                calculate_text_width(s, 4, false, WidthMethod::Unicode),
                "U+{cp:04X}",
            );
        }
    }
}

#[test]
fn req_002_getwidthat_simd_boundary() {
    // ported: getWidthAt: grapheme at SIMD boundary
    let mut buf = vec![b'x'; 32];
    buf[14..17].copy_from_slice("世".as_bytes());
    let text = std::str::from_utf8(&buf).unwrap();
    assert_eq!(1, width_at(text, 13, 8, WidthMethod::Unicode));
    assert_eq!(2, width_at(text, 14, 8, WidthMethod::Unicode));
    assert_eq!(1, width_at(text, 17, 8, WidthMethod::Unicode));
}

#[test]
fn req_002_clusters_simd_boundary() {
    // ported: findRenderClusterInfo: at SIMD boundary
    let mut buf = vec![b'x'; 32];
    buf[14..17].copy_from_slice("世".as_bytes());
    let text = std::str::from_utf8(&buf).unwrap();
    let result = render_clusters(text, 4, false, WidthMethod::Unicode);
    let found = result
        .iter()
        .find(|g| g.byte_start == 14)
        .expect("CJK cluster at 14");
    assert_eq!(3, found.byte_len);
    assert_eq!(2, found.width_cols);
}

#[test]
fn req_001_long_cluster_chain() {
    // ported: findRenderClusterInfo: long render-cluster metadata exceeds u8 ranges
    let mut text = String::new();
    for i in 0..130 {
        if i > 0 {
            text.push('\u{200D}');
        }
        text.push('👩');
    }
    assert!(text.len() > u8::MAX as usize);
    for (method, width) in [(WidthMethod::Unicode, 2), (WidthMethod::Wcwidth, 260)] {
        let result = render_clusters(&text, 4, false, method);
        assert_eq!(1, result.len());
        assert_eq!(0, result[0].byte_start);
        assert_eq!(text.len() as u32, result[0].byte_len);
        assert_eq!(width, result[0].width_cols);
    }
}

#[test]
fn req_002_large_tabs() {
    // ported: calculateTextWidth: large text with many tabs
    let mut buf = vec![0u8; 1000];
    let mut expected = 0u32;
    for (i, b) in buf.iter_mut().enumerate() {
        if i % 10 == 0 {
            *b = b'\t';
            expected += 4;
        } else {
            *b = b'a';
            expected += 1;
        }
    }
    let text = std::str::from_utf8(&buf).unwrap();
    assert_eq!(
        expected,
        calculate_text_width(text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_002_manual_calculation() {
    // ported: calculateTextWidth: comparison with manual calculation
    let cases = [
        ("\t", 2, 2),
        ("\t\t", 2, 4),
        ("a\t", 2, 3),
        ("\ta", 2, 3),
        ("a\tb", 2, 4),
        ("ab\tcd", 4, 8),
        ("\t\tx", 2, 5),
        ("世\t界", 2, 6),
    ];
    for (text, tab, expected) in cases {
        assert_eq!(
            expected,
            calculate_text_width(text, tab, false, WidthMethod::Unicode)
        );
    }
}

#[test]
fn req_003_emoji_graphemes_table() {
    // ported: calculateTextWidth: various emoji graphemes
    for text in ["✅", "❤️", "🎉", "🔥", "💯", "🚀", "⭐", "👍"] {
        assert_eq!(
            2,
            calculate_text_width(text, 4, false, WidthMethod::Unicode)
        );
    }
}

#[test]
fn req_003_long_combining_chain() {
    // ported: calculateTextWidth: long grapheme cluster chain
    let mut text = String::from("e");
    for _ in 0..10 {
        text.push('\u{301}');
    }
    assert_eq!(
        1,
        calculate_text_width(&text, 4, false, WidthMethod::Unicode)
    );
}

#[test]
fn req_003_malayalam_report() {
    // ported: calculateTextWidth: Malayalam report matches Ghostty grapheme widths
    let report = "OpenCode search configuration പരിശോധിക്കൽ";
    assert_eq!(
        36,
        calculate_text_width(report, 4, false, WidthMethod::Unicode)
    );
    assert_eq!(
        2,
        calculate_text_width("രി", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ശോ", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ധി", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        2,
        calculate_text_width("ക്ക", 4, false, WidthMethod::UnicodeWide)
    );
    assert_eq!(
        40,
        calculate_text_width(report, 4, false, WidthMethod::UnicodeWide)
    );
}

#[test]
fn req_003_multilingual_consistency() {
    // ported: findRenderClusterInfo: comprehensive multilingual text
    const TEXT: &str = include_str!("fixtures/multilingual.txt");
    let result = render_clusters(TEXT, 4, false, WidthMethod::Unicode);
    assert!(!result.is_empty());
    let mut prev_end = 0u32;
    for g in &result {
        assert!(g.byte_start >= prev_end);
        assert_eq!(
            calculate_text_width(
                &TEXT[..g.byte_start as usize],
                4,
                false,
                WidthMethod::Unicode
            ),
            g.col_start,
        );
        prev_end = g.byte_start + g.byte_len;
    }
}
