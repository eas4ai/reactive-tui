//! Part of tests/review_facade.rs: what the generated C header documents.

/// The generated C header.
pub fn native_header() -> &'static str {
    include_str!("../../include/reactive_tui/native.h")
}

/// The documentation block right above the declaration of `function` in the
/// header: the `/** ... */` comment whose last line precedes the line that
/// holds `function(`, or an empty string when the declaration has none.
pub fn doc_of(function: &str) -> String {
    let header = native_header();
    let needle = format!("{function}(");
    let Some(at) = header
        .lines()
        .position(|line| line.contains(&needle) && !line.trim_start().starts_with('*'))
    else {
        panic!("{function} is not declared in include/reactive_tui/native.h");
    };
    let lines: Vec<&str> = header.lines().collect();
    let mut index = at;
    // A declaration may continue a previous line's parameter list; find its
    // first line.
    while index > 0
        && !lines[index - 1].trim().is_empty()
        && !lines[index - 1].trim_end().ends_with(';')
        && !lines[index - 1].trim_end().ends_with("*/")
        && !lines[index - 1].trim_end().ends_with('}')
    {
        index -= 1;
    }
    if index == 0 || !lines[index - 1].trim_end().ends_with("*/") {
        return String::new();
    }
    let mut block = Vec::new();
    let mut cursor = index - 1;
    loop {
        block.push(lines[cursor]);
        if lines[cursor].trim_start().starts_with("/**")
            || lines[cursor].trim_start().starts_with("/*")
        {
            break;
        }
        if cursor == 0 {
            break;
        }
        cursor -= 1;
    }
    block.reverse();
    block.join("\n")
}

/// Whether `function` has a documentation block in the header.
pub fn documented(function: &str) -> bool {
    !doc_of(function).trim().is_empty()
}
