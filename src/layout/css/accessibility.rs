//! Accessibility CSS utilities
//!
//! This module provides utilities for:
//! - ARIA attributes (aria-hidden, aria-label, etc.)
//! - Role attributes (role-button, role-dialog, etc.)
//! - Tab navigation (tabindex-*)
//! - Accessibility metadata

use crate::layout::style::StyleBuilder;

/// Apply accessibility utilities
pub fn apply_accessibility_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    let mut sb = sb;
    if token.starts_with("aria-") {
        let (attribute, value) = if let Some(attribute) = token.strip_suffix("-false") {
            (attribute, Some("false"))
        } else if let Some(level) = token.strip_prefix("aria-live-") {
            ("aria-live", Some(level))
        } else if matches!(token, "aria-label" | "aria-labelledby" | "aria-describedby") {
            (token, None)
        } else {
            (token, Some("true"))
        };
        match value {
            Some(value) => {
                sb.accessibility
                    .insert(attribute.into(), Some(value.into()));
            }
            None => {
                sb.accessibility.entry(attribute.into()).or_insert(None);
            }
        }
    } else if let Some(role) = token.strip_prefix("role-") {
        sb = super::focus::apply_role(sb, role);
    } else if let Some(index) = token.strip_prefix("tabindex-") {
        sb.accessibility
            .insert("tabindex".into(), Some(index.into()));
    } else if matches!(
        token,
        "keyboard-focusable" | "keyboard-only" | "reduced-motion"
    ) {
        sb.accessibility.insert(token.into(), Some("true".into()));
    }
    // ARIA attributes
    if token.starts_with("aria-") {
        return apply_aria_utility(token, sb);
    }

    // Role attributes
    if token.starts_with("role-") {
        return apply_role_utility(token, sb);
    }

    // Tab navigation
    if token.starts_with("tabindex-") {
        return apply_tabindex_utility(token, sb);
    }

    // Other accessibility utilities
    match token {
        // Screen reader utilities (already handled in focus.rs, but included for completeness)
        "sr-only" => Some(apply_screen_reader_only(sb)),
        "not-sr-only" => Some(apply_not_screen_reader_only(sb)),

        // Keyboard navigation hints
        "keyboard-focusable" => Some(apply_keyboard_focusable(sb)),
        "keyboard-only" => Some(apply_keyboard_only(sb)),

        // High contrast mode utilities
        "high-contrast" => Some(apply_high_contrast(sb)),
        "reduced-motion" => Some(apply_reduced_motion(sb)),

        _ => None,
    }
}

/// Apply ARIA attribute utilities
fn apply_aria_utility(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Common ARIA states
        "aria-hidden" => Some(apply_aria_hidden(sb, true)),
        "aria-hidden-false" => Some(apply_aria_hidden(sb, false)),
        "aria-expanded" => Some(apply_aria_expanded(sb, true)),
        "aria-expanded-false" => Some(apply_aria_expanded(sb, false)),
        "aria-selected" => Some(apply_aria_selected(sb, true)),
        "aria-selected-false" => Some(apply_aria_selected(sb, false)),
        "aria-checked" => Some(apply_aria_checked(sb, true)),
        "aria-checked-false" => Some(apply_aria_checked(sb, false)),
        "aria-disabled" => Some(apply_aria_disabled(sb, true)),
        "aria-disabled-false" => Some(apply_aria_disabled(sb, false)),
        "aria-pressed" => Some(apply_aria_pressed(sb, true)),
        "aria-pressed-false" => Some(apply_aria_pressed(sb, false)),

        // ARIA properties
        "aria-label" => Some(apply_aria_label(sb)),
        "aria-labelledby" => Some(apply_aria_labelledby(sb)),
        "aria-describedby" => Some(apply_aria_describedby(sb)),
        "aria-live-polite" => Some(apply_aria_live(sb, "polite")),
        "aria-live-assertive" => Some(apply_aria_live(sb, "assertive")),
        "aria-live-off" => Some(apply_aria_live(sb, "off")),

        _ => None,
    }
}

/// Apply role attribute utilities
fn apply_role_utility(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    let role = token.strip_prefix("role-")?;

    match role {
        // Interactive roles
        "button" => Some(apply_role_button(sb)),
        "link" => Some(apply_role_link(sb)),
        "menuitem" => Some(apply_role_menuitem(sb)),
        "tab" => Some(apply_role_tab(sb)),
        "tabpanel" => Some(apply_role_tabpanel(sb)),
        "option" => Some(apply_role_option(sb)),

        // Container roles
        "dialog" => Some(apply_role_dialog(sb)),
        "menu" => Some(apply_role_menu(sb)),
        "tablist" => Some(apply_role_tablist(sb)),
        "listbox" => Some(apply_role_listbox(sb)),
        "grid" => Some(apply_role_grid(sb)),
        "tree" => Some(apply_role_tree(sb)),

        // Content roles
        "heading" => Some(apply_role_heading(sb)),
        "article" => Some(apply_role_article(sb)),
        "main" => Some(apply_role_main(sb)),
        "navigation" => Some(apply_role_navigation(sb)),
        "banner" => Some(apply_role_banner(sb)),
        "contentinfo" => Some(apply_role_contentinfo(sb)),

        // Utility roles
        "presentation" => Some(apply_role_presentation(sb)),
        "none" => Some(apply_role_none(sb)),

        _ => Some(apply_generic_role(sb, role)),
    }
}

/// Apply tabindex utilities
fn apply_tabindex_utility(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    let index_str = token.strip_prefix("tabindex-")?;

    // Parse tabindex (both positive and negative values)
    let index = index_str.parse::<i32>().ok()?;

    Some(apply_tabindex(sb, index))
}

// ARIA state implementations
fn apply_aria_hidden(sb: StyleBuilder, hidden: bool) -> StyleBuilder {
    if hidden {
        // Hidden from screen readers, but visually present
        sb.opacity(0.7) // Slightly dimmed to indicate hidden state
    } else {
        sb.opacity(1.0)
    }
}

fn apply_aria_expanded(sb: StyleBuilder, expanded: bool) -> StyleBuilder {
    if expanded {
        // Visual indication of expanded state
        sb.bold(true)
    } else {
        sb
    }
}

fn apply_aria_selected(sb: StyleBuilder, selected: bool) -> StyleBuilder {
    if selected {
        // Visual indication of selected state
        sb.reverse(true)
    } else {
        sb
    }
}

fn apply_aria_checked(sb: StyleBuilder, checked: bool) -> StyleBuilder {
    if checked {
        // Visual indication of checked state
        sb.bold(true).fg_rgba(0.0, 0.8, 0.0, 1.0) // Green for checked
    } else {
        sb
    }
}

fn apply_aria_disabled(sb: StyleBuilder, disabled: bool) -> StyleBuilder {
    if disabled {
        // Visual indication of disabled state
        sb.opacity(0.5)
    } else {
        sb.opacity(1.0)
    }
}

fn apply_aria_pressed(sb: StyleBuilder, pressed: bool) -> StyleBuilder {
    if pressed {
        // Visual indication of pressed state
        sb.reverse(true)
    } else {
        sb
    }
}

// ARIA property implementations (metadata only)
fn apply_aria_label(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_aria_labelledby(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_aria_describedby(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_aria_live(sb: StyleBuilder, _level: &str) -> StyleBuilder {
    sb
}

// Role implementations
fn apply_role_button(sb: StyleBuilder) -> StyleBuilder {
    // Buttons should be visually distinct and focusable
    sb.bold(true)
}

fn apply_role_link(sb: StyleBuilder) -> StyleBuilder {
    // Links are typically underlined
    sb.underline(true)
}

fn apply_role_menuitem(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_tab(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_tabpanel(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_option(sb: StyleBuilder) -> StyleBuilder {
    sb
}

fn apply_role_dialog(sb: StyleBuilder) -> StyleBuilder {
    // Dialogs might have a border or background
    sb.bg_rgba(0.1, 0.1, 0.1, 0.9) // Dark background
}

fn apply_role_menu(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_tablist(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_listbox(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_grid(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_tree(sb: StyleBuilder) -> StyleBuilder {
    sb
}

fn apply_role_heading(sb: StyleBuilder) -> StyleBuilder {
    // Headings are typically bold
    sb.bold(true)
}

fn apply_role_article(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_main(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_navigation(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_banner(sb: StyleBuilder) -> StyleBuilder {
    sb
}
fn apply_role_contentinfo(sb: StyleBuilder) -> StyleBuilder {
    sb
}

fn apply_role_presentation(sb: StyleBuilder) -> StyleBuilder {
    // Presentation role removes semantic meaning
    sb
}

fn apply_role_none(sb: StyleBuilder) -> StyleBuilder {
    // None role is similar to presentation
    sb
}

fn apply_generic_role(sb: StyleBuilder, _role: &str) -> StyleBuilder {
    // Generic role handler for unknown roles
    sb
}

// Other accessibility utilities
fn apply_screen_reader_only(sb: StyleBuilder) -> StyleBuilder {
    super::focus::apply_focus_utilities("sr-only", sb).expect("known accessibility utility")
}

fn apply_not_screen_reader_only(sb: StyleBuilder) -> StyleBuilder {
    super::focus::apply_focus_utilities("not-sr-only", sb).expect("known accessibility utility")
}

fn apply_keyboard_focusable(sb: StyleBuilder) -> StyleBuilder {
    // Indicate that element can receive keyboard focus
    sb
}

fn apply_keyboard_only(sb: StyleBuilder) -> StyleBuilder {
    // Element only accessible via keyboard
    sb
}

fn apply_high_contrast(sb: StyleBuilder) -> StyleBuilder {
    // High contrast mode styling
    sb.bold(true)
        .fg_rgba(1.0, 1.0, 1.0, 1.0)
        .bg_rgba(0.0, 0.0, 0.0, 1.0)
}

fn apply_reduced_motion(sb: StyleBuilder) -> StyleBuilder {
    // Reduced motion preferences (metadata only for TUI)
    sb
}

fn apply_tabindex(sb: StyleBuilder, _index: i32) -> StyleBuilder {
    // Tabindex is metadata that doesn't affect visual styling
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aria_utilities() {
        let sb = StyleBuilder::new();

        // Test ARIA states
        let result = apply_accessibility_utilities("aria-hidden", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("aria-expanded", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("aria-selected", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_role_utilities() {
        let sb = StyleBuilder::new();

        // Test common roles
        let result = apply_accessibility_utilities("role-button", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("role-dialog", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("role-heading", sb.clone());
        assert!(result.is_some());
    }

    #[test]
    fn test_tabindex_utilities() {
        let sb = StyleBuilder::new();

        // Test tabindex values
        let result = apply_accessibility_utilities("tabindex-0", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("tabindex--1", sb.clone());
        assert!(result.is_some());

        let result = apply_accessibility_utilities("tabindex-1", sb.clone());
        assert!(result.is_some());
    }
}
