use reactive_tui::markdown::MarkdownRenderer;

const SOURCE: &str = "fn main() { let answer = 42; }\n";

/// The text and color of every run the renderer paints for `markdown`.
fn runs(markdown: &str) -> Vec<(String, reactive_tui::core::surface::Rgba)> {
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
}
