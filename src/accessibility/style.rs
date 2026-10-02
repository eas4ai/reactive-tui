//! Resolve style semantics before presenting an App candidate.

use crate::{
    component::{bridge::element_style, Element, ElementType},
    error::{ReactiveError, Result},
};
use accesskit::{Live, Role, Toggled};
use std::collections::{BTreeMap, HashMap, HashSet};

type Attributes = BTreeMap<String, Option<String>>;

#[derive(Default)]
struct Label {
    text: String,
    references: Option<String>,
}

/// Validate the entire candidate before publishing any frame or reader update.
pub(crate) fn prepare(element: &mut Element) -> Result<()> {
    let mut labels = HashMap::new();
    collect(element, &mut labels)?;
    apply(element, &labels, false)
}

fn invalid(message: impl Into<String>) -> ReactiveError {
    ReactiveError::invalid_state(format!("Accessibility style: {}", message.into()))
}

fn collect(element: &Element, labels: &mut HashMap<String, Label>) -> Result<()> {
    if let Some(id) = element
        .metadata
        .accessibility_options
        .as_ref()
        .and_then(|o| o.id.as_ref())
    {
        if id.is_empty() || id.chars().any(char::is_whitespace) {
            return Err(invalid(
                "accessibility IDs must be nonempty and contain no whitespace",
            ));
        }
        let attrs = element_style(element)?.accessibility;
        let text = attrs
            .get("aria-label")
            .and_then(|v| v.clone())
            .unwrap_or_else(|| text(element));
        let label = Label {
            text,
            references: attrs.get("aria-labelledby").and_then(|v| v.clone()),
        };
        if labels.insert(id.clone(), label).is_some() {
            return Err(invalid(format!("duplicate accessibility ID {id:?}")));
        }
    }
    for child in &element.children {
        collect(child, labels)?;
    }
    Ok(())
}

fn text(element: &Element) -> String {
    if let Some(label) = element
        .metadata
        .accessibility_options
        .as_ref()
        .and_then(|o| o.label.as_ref())
    {
        return label.clone();
    }
    if let Some(label) = element
        .metadata
        .accessibility
        .as_ref()
        .and_then(|n| n.inner.label())
    {
        return label.to_owned();
    }
    if let ElementType::Text(text) = &element.element_type {
        return text.clone();
    }
    element
        .children
        .iter()
        .map(text)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn references(
    value: &str,
    labels: &HashMap<String, Label>,
    visiting: &mut HashSet<String>,
) -> Result<String> {
    if value.split_whitespace().next().is_none() {
        return Err(invalid(
            "label and description references must name at least one accessibility ID",
        ));
    }
    let mut result = Vec::new();
    for id in value.split_whitespace() {
        let label = labels
            .get(id)
            .ok_or_else(|| invalid(format!("unknown accessibility ID {id:?}")))?;
        if !visiting.insert(id.to_owned()) {
            return Err(invalid(format!("cyclic label reference at {id:?}")));
        }
        result.push(match &label.references {
            Some(ids) => references(ids, labels, visiting)?,
            None => label.text.clone(),
        });
        visiting.remove(id);
    }
    Ok(result.join(" "))
}

fn boolean(attribute: &str, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(invalid(format!(
            "{attribute} expects true or false, received {value:?}"
        ))),
    }
}

fn apply(
    element: &mut Element,
    labels: &HashMap<String, Label>,
    ancestor_disabled: bool,
) -> Result<()> {
    element.metadata.disabled |= ancestor_disabled;
    let attrs: Attributes = element_style(element)?.accessibility;
    for (attribute, value) in &attrs {
        if attribute == "aria-label"
            && value.is_none()
            && (element
                .metadata
                .accessibility_options
                .as_ref()
                .is_some_and(|o| o.label.is_some())
                || element
                    .metadata
                    .accessibility
                    .as_ref()
                    .is_some_and(|n| n.inner.label().is_some()))
        {
            continue;
        }
        let value = value.as_deref().ok_or_else(|| {
            invalid(format!(
                "{attribute} needs a value supplied with apply_aria_attribute"
            ))
        })?;
        match attribute.as_str() {
            "sr-only" => {
                element
                    .metadata
                    .accessibility_options
                    .get_or_insert_default()
                    .screen_reader_only = boolean(attribute, value)?
            }
            "tabindex" => {
                let index = value
                    .parse::<i32>()
                    .map_err(|_| invalid("tabindex expects a signed integer"))?;
                let focus = element.focus.get_or_insert_default();
                focus.focusable = true;
                focus.tab_index = index;
            }
            "keyboard-focusable" | "keyboard-only" => {
                element.focus.get_or_insert_default().focusable = true;
                if attribute == "keyboard-only" {
                    element
                        .metadata
                        .accessibility_options
                        .get_or_insert_default()
                        .keyboard_only = true;
                }
            }
            "reduced-motion" => {}
            "aria-label" => {
                element
                    .metadata
                    .accessibility_options
                    .get_or_insert_default()
                    .label = Some(value.to_owned());
            }
            "aria-labelledby" | "aria-describedby" => {
                let value = references(value, labels, &mut HashSet::new())?;
                if attribute == "aria-labelledby" {
                    element
                        .metadata
                        .accessibility_options
                        .get_or_insert_default()
                        .label = Some(value);
                } else {
                    node(element).set_description(value);
                }
            }
            "role" => node(element).set_role(role(value)),
            "aria-expanded" => node(element).set_expanded(boolean(attribute, value)?),
            "aria-selected" => node(element).set_selected(boolean(attribute, value)?),
            "aria-checked" | "aria-pressed" => {
                let toggled = match value {
                    "mixed" => Toggled::Mixed,
                    _ if boolean(attribute, value)? => Toggled::True,
                    _ => Toggled::False,
                };
                node(element).set_toggled(toggled);
            }
            "aria-hidden" => {
                if boolean(attribute, value)? {
                    node(element).set_hidden();
                } else {
                    node(element).clear_hidden();
                }
            }
            "aria-disabled" => {
                let disabled = boolean(attribute, value)?;
                element.metadata.disabled |= disabled;
                if disabled {
                    node(element).set_disabled();
                } else {
                    node(element).clear_disabled();
                }
            }
            "aria-live" => node(element).set_live(match value {
                "polite" => Live::Polite,
                "assertive" => Live::Assertive,
                "off" => Live::Off,
                _ => return Err(invalid(format!("unknown aria-live level {value:?}"))),
            }),
            _ => return Err(invalid(format!("unsupported attribute {attribute:?}"))),
        }
    }
    let disabled = element.metadata.disabled;
    for child in &mut element.children {
        apply(child, labels, disabled)?;
    }
    Ok(())
}

fn node(element: &mut Element) -> &mut accesskit::Node {
    &mut element
        .metadata
        .accessibility
        .get_or_insert_with(|| super::Node::new(Role::Unknown))
        .inner
}

fn role(value: &str) -> Role {
    match value {
        "button" => Role::Button,
        "link" => Role::Link,
        "menuitem" => Role::MenuItem,
        "tab" => Role::Tab,
        "tabpanel" => Role::TabPanel,
        "option" => Role::ListBoxOption,
        "dialog" => Role::Dialog,
        "menu" => Role::Menu,
        "tablist" => Role::TabList,
        "listbox" => Role::ListBox,
        "grid" => Role::Grid,
        "tree" => Role::Tree,
        "heading" => Role::Heading,
        "article" => Role::Article,
        "main" => Role::Main,
        "navigation" => Role::Navigation,
        "banner" => Role::Banner,
        "contentinfo" => Role::ContentInfo,
        "presentation" | "none" => Role::GenericContainer,
        _ => Role::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{
        css::focus::{apply_aria_attribute, apply_role},
        style::StyleBuilder,
    };

    fn styled(mut element: Element, style: StyleBuilder) -> Element {
        element.metadata.styles = Some(std::sync::Arc::new(style.snapshot()));
        element
    }

    #[test]
    fn style_snapshots_preserve_values_roles_states_and_label_relationships() {
        let style = apply_aria_attribute(StyleBuilder::new(), "labelledby", "title suffix");
        let style = apply_aria_attribute(style, "describedby", "help");
        let style = apply_aria_attribute(style, "checked", "mixed");
        let control = styled(Element::text("Painted"), apply_role(style, "button")).class(
            "aria-labelledby aria-describedby aria-expanded aria-selected-false tabindex--1",
        );
        let mut root = Element::fragment().children(vec![
            Element::text("Accessible").with_accessibility_id("title"),
            Element::text("action").with_accessibility_id("suffix"),
            Element::text("Extra help")
                .with_accessibility_id("help")
                .class("sr-only"),
            control,
        ]);
        prepare(&mut root).unwrap();
        let control = &root.children[3];
        let node = &control.metadata.accessibility.as_ref().unwrap().inner;
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.description(), Some("Extra help"));
        assert_eq!(node.is_expanded(), Some(true));
        assert_eq!(node.is_selected(), Some(false));
        assert_eq!(node.toggled(), Some(Toggled::Mixed));
        assert_eq!(
            control
                .metadata
                .accessibility_options
                .as_ref()
                .unwrap()
                .label
                .as_deref(),
            Some("Accessible action")
        );
        assert_eq!(control.focus.as_ref().unwrap().tab_index, -1);
        assert!(control.focus.as_ref().unwrap().focusable);
    }

    #[test]
    fn label_marker_preserves_explicit_element_labels_and_false_states() {
        let mut element = Element::text("Painted").with_accessibility_label("Reader")
            .class("aria-label aria-hidden aria-hidden-false aria-expanded aria-expanded-false aria-live-assertive");
        prepare(&mut element).unwrap();
        let node = &element.metadata.accessibility.as_ref().unwrap().inner;
        assert!(!node.is_hidden());
        assert_eq!(node.is_expanded(), Some(false));
        assert_eq!(node.live(), Some(Live::Assertive));
        assert_eq!(
            element
                .metadata
                .accessibility_options
                .as_ref()
                .unwrap()
                .label
                .as_deref(),
            Some("Reader")
        );
    }

    #[test]
    fn invalid_values_missing_duplicate_and_cyclic_references_fail_before_presentation() {
        let reference = |id: &str| {
            styled(
                Element::text("Painted"),
                apply_aria_attribute(StyleBuilder::new(), "labelledby", id),
            )
        };
        let cases = [
            (
                Element::text("Painted").class("aria-label"),
                "needs a value",
            ),
            (reference("absent"), "unknown accessibility ID"),
            (reference(""), "at least one"),
            (
                Element::fragment().children(vec![
                    Element::text("a").with_accessibility_id("same"),
                    Element::text("b").with_accessibility_id("same"),
                ]),
                "duplicate",
            ),
            (reference("self").with_accessibility_id("self"), "cyclic"),
            (
                Element::text("a").with_accessibility_id("two words"),
                "no whitespace",
            ),
            (
                styled(
                    Element::text("a"),
                    apply_aria_attribute(StyleBuilder::new(), "expanded", "maybe"),
                ),
                "true or false",
            ),
        ];
        for (mut element, expected) in cases {
            let error = prepare(&mut element).unwrap_err().to_string();
            assert!(
                error.contains(expected),
                "{error:?} should contain {expected:?}"
            );
        }
    }
}
