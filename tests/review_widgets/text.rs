use reactive_tui::core::surface::Rgba;
use reactive_tui::editor::SyntaxEditor;
use reactive_tui::markdown::MarkdownRenderer;
use reactive_tui::syntax::highlighter::MAX_SYNTAX_BYTES;

/// The editor's line-number gutter: four digits and a space.
const GUTTER: usize = 5;

const SOURCE: &str = "fn main() { let answer = 42; }\n";

/// The text and color of every run the renderer paints for `markdown`.
fn runs(markdown: &str) -> Vec<(String, Rgba)> {
    MarkdownRenderer::new()
        .render_to_styled_lines(markdown)
        .iter()
        .flat_map(|line| line.runs.iter())
        .map(|run| (run.text.clone(), run.fg))
        .collect()
}

/// TXT-005: a fence written ```rust highlights as one written ```Rust does.
#[test]
fn txt_005_a_lowercase_fence_highlights_as_its_language() {
    let upper = runs(&format!("```Rust\n{SOURCE}```"));
    assert!(
        upper.windows(2).any(|pair| pair[0].1 != pair[1].1),
        "the fence named by the display name is highlighted: {upper:?}"
    );
    let lower = runs(&format!("```rust\n{SOURCE}```"));
    assert_eq!(
        lower, upper,
        "a lowercase fence highlights as the uppercase one"
    );
    let extension = runs(&format!("```rs\n{SOURCE}```"));
    assert_eq!(
        extension, upper,
        "a fence naming the file extension highlights as the language"
    );
}

/// The foreground of each character of the editor's first visible line,
/// past the gutter and the cursor's cell, which paints inverted.
fn line_colors(editor: &mut SyntaxEditor) -> Vec<Rgba> {
    editor.get_styled_lines()[0]
        .runs
        .iter()
        .flat_map(|run| run.text.chars().map(move |_| run.fg))
        .skip(GUTTER + 1)
        .collect()
}

/// TXT-006: a line the highlighter leaves plain, here one too long to
/// parse, is painted in the editor's foreground, as a line of an editor
/// with no highlighter is, and never in black.
#[test]
fn txt_006_a_plain_fallback_line_takes_the_editors_foreground() {
    let oversized = "x".repeat(MAX_SYNTAX_BYTES + 1);
    let mut highlighted = SyntaxEditor::with_language(&oversized, "Rust").unwrap();
    highlighted.set_size(40, 1);
    let mut unhighlighted = SyntaxEditor::with_language(&oversized, "no such language").unwrap();
    unhighlighted.set_size(40, 1);
    let fallback = line_colors(&mut highlighted);
    let own = line_colors(&mut unhighlighted);
    assert!(!fallback.is_empty() && fallback.len() == own.len());
    assert_eq!(
        fallback[0], own[0],
        "the fallback line is painted in the editor's foreground"
    );
    assert_eq!(fallback, own);
    assert!(fallback.iter().all(|fg| *fg != Rgba::black()));
}
