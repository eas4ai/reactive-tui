# Text editing, Markdown and syntax

Prefix: TXT

The editor (src/editor: `TextEditor`, `SyntaxEditor` with its `LineCache`),
the syntax highlighter (src/syntax: `SyntaxHighlighter` over tree-sitter
languages and themes, with a byte limit `MAX_SYNTAX_BYTES` on its checked
entry points) and the Markdown renderer (src/markdown: `MarkdownRenderer`
over comrak's tree, with `MAX_MARKDOWN_BYTES` on its checked entry points)
turn text into styled lines for a display element or a surface. The manual
chapter manual/text-editing-markdown-and-syntax.md describes them.

Read on 2026-10-04 from the developer's production code review of 65e618ec
(findings N12, N13 and N21) and checked against the code on 2026-10-05:
`SyntaxEditor` highlights the whole document on every change and discards
the result, then paints each visible line from a second cache that parses
the line alone, so a line inside a block comment or a multiline string is
painted as code (also the developer's review of 2026-10-01, finding 7, the
backlog item syntax-editor-loses-multiline-context); `highlight_lines`
parses the whole document on every call although its comment and the
manual promise cached reuse, and the byte limits guard only the checked
entry points; and a Markdown table's header is followed by a bare `├`
because the walker never reads the table's columns.

## Observed

(none yet)

## Draft

[TXT-001] `SyntaxEditor` MUST paint each visible line with the highlighting the whole document gives it: a line inside a block comment or a multiline string MUST be painted as that comment or string, and after an edit anywhere in the document, above or below the visible lines, the painted lines MUST be what the edited document gives them.
Falsifier: For the Rust document `/*\nfn main() {}\n*/` the editor's second line has foreground colors that differ from those a fresh `SyntaxHighlighter` gives that line of the whole document; or after `/* ` is inserted on a line above a visible `let x = 5;`, that line keeps its keyword colors.
Mechanism: review-native
Rationale: `rehighlight_visible` highlighted the whole document and discarded the result, and painting re-parsed each line alone through a cache keyed by the line's own text, which cannot see a change of context above it (the developer's code review of 2026-10-04, N12; the developer's review of 2026-10-01, finding 7).
Status: Agreed 2026-10-05

[TXT-002] `SyntaxHighlighter::highlight_lines` MUST parse the document once per change of its text or theme and serve further calls for the same text from its cache; `highlight_lines` and `rehighlight_line` MUST apply `MAX_SYNTAX_BYTES` as `try_highlight_text` does, returning plain lines for a larger text; `MarkdownRenderer::render_with_sourcepos` MUST apply `MAX_MARKDOWN_BYTES` as `render_to_styled_lines` does, returning its error line and no positions; and the manual's words on incremental highlighting MUST match what the highlighter does.
Falsifier: Two `highlight_lines` calls with the same text and theme parse the document twice; a text of `MAX_SYNTAX_BYTES + 1` bytes given to `highlight_lines` or `rehighlight_line` is parsed rather than returned plain; `render_with_sourcepos` renders a source of `MAX_MARKDOWN_BYTES + 1` bytes; or manual/text-editing-markdown-and-syntax.md promises reuse the highlighter does not do.
Mechanism: review-native
Rationale: `highlight_lines` built a new highlighter and parsed the whole text on every call despite the comment beside it and the manual, and the byte limits guarded only the checked entry points (N13).
Status: Agreed 2026-10-05

[TXT-003] The Markdown renderer MUST draw a table from its columns: the header separator MUST have one segment per column, joined by `┼` and closed by `┤`, each as wide as its column, and every cell MUST be padded to its column's width by the column's declared alignment, left, right or center.
Falsifier: `MarkdownRenderer::new().render_to_styled_lines("| Name | Qty |\n| :--- | ---: |\n| ab | 1 |")` has, after the header row `│ Name │ Qty │`, a line other than `├──────┼─────┤`, or a body row other than `│ ab   │   1 │`.
Mechanism: review-native
Rationale: The walker never read the table's column count or alignment, so a header row was followed by a bare `├` (N21).
Status: Agreed 2026-10-05
