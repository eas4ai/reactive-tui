use base64::Engine;
use proptest::prelude::*;
use reactive_tui::core::geometry::Point;
use reactive_tui::core::surface::{Attr, Cell, DiffWriter, Rgba, Surface, TextStyle};
use reactive_tui::core::terminal::Terminal as HostTerminal;
use reactive_tui::platform::parser::EscapeSequenceParser;
use reactive_tui::platform::TerminalEvent as InputEvent;
use reactive_tui::terminal::{utils, Terminal, TerminalConfig, TerminalEvent};
use reactive_tui::ui::paint::PaintStyle;
use std::collections::BTreeSet;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use syn::{
    FnArg, GenericArgument, ImplItem, Item, ItemImpl, ItemStruct, PathArguments, Type,
    TypeParamBound, Visibility,
};

const EXPECTED_STRING_PATHS: &[&str] = &[
    "src/builder/dialog_builders.rs::ProgressDialogBuilder::title",
    "src/builder/dialog_builders.rs::WizardBuilder::title",
    "src/builder/dialog_builders.rs::WizardStep::title",
    "src/builder/specialized.rs::ConfirmationDialogBuilder::title",
    "src/builder/specialized.rs::DialogBuilder::title",
    "src/builder/widgets/chart.rs::ChartBuilder::title",
    "src/builder/widgets/dialog.rs::ModalBuilder::title",
    "src/builder/widgets/menu.rs::MenuBarBuilder::title",
    "src/core/surface.rs::Surface::set_grapheme",
    "src/core/surface.rs::Surface::write_str",
    "src/core/surface.rs::Surface::write_str_at",
    "src/core/surface.rs::Surface::write_str_clipped",
    "src/core/surface.rs::Surface::write_styled_text",
    "src/core/surface.rs::Surface::write_text_enhanced",
    "src/core/surface.rs::Surface::write_text_styled",
    "src/core/surface.rs::Surface::write_text_styled_clipped",
    "src/core/surface.rs::Surface::write_text_wrapped",
    "src/core/surface.rs::SurfaceSubview::write_text",
    "src/core/terminal.rs::Terminal::set_title",
    "src/terminal/ansi.rs::utils::set_title",
    "src/terminal/mod.rs::TerminalConfig::title",
    "src/terminal/terminal_impl.rs::Terminal::set_title",
    "src/widgets/dialog/autocomplete.rs::AutocompleteDialogOptions::title",
    "src/widgets/dialog/confirmation.rs::ConfirmationDialogOptions::title",
    "src/widgets/dialog/input.rs::InputDialogOptions::title",
    "src/widgets/dialog/progress.rs::ProgressDialogOptions::title",
    "src/widgets/dialog/wizard.rs::WizardDialogOptions::title",
    "src/widgets/dialog/wizard.rs::WizardStep::title",
    "src/widgets/display/charts.rs::ChartProps::with_title",
    "src/widgets/display/charts.rs::ChartsBuilder::title",
    "src/widgets/display/charts/plot/axis.rs::Axis::with_title",
    "src/widgets/display/charts/typed.rs::DonutChartBuilder::title",
    "src/widgets/display/modal.rs::Modal::with_title",
    "src/widgets/display/table.rs::TableColumn::title",
    "src/widgets/layout/accordion.rs::AccordionSection::title",
    "src/widgets/menu/dialog.rs::DialogMenuBuilder::title",
    "src/widgets/menu/menubar.rs::MenuBarBuilder::title",
    "src/widgets/terminal.rs::TerminalProps::title",
];

#[derive(Clone, Copy, Debug)]
enum SurfaceWriter {
    Grapheme,
    Str,
    StrAt,
    StrClipped,
    Styled,
    StyledClipped,
    Enhanced,
    Wrapped,
    StyledText,
    Subview,
}

const SURFACE_WRITERS: &[SurfaceWriter] = &[
    SurfaceWriter::Grapheme,
    SurfaceWriter::Str,
    SurfaceWriter::StrAt,
    SurfaceWriter::StrClipped,
    SurfaceWriter::Styled,
    SurfaceWriter::StyledClipped,
    SurfaceWriter::Enhanced,
    SurfaceWriter::Wrapped,
    SurfaceWriter::StyledText,
    SurfaceWriter::Subview,
];

fn write_surface(path: SurfaceWriter, text: &str) -> Surface {
    let mut surface = Surface::new(64, 4);
    let style = TextStyle::new(Rgba::white(), Rgba::black(), Attr::empty());
    let paint_style = PaintStyle::default();
    match path {
        SurfaceWriter::Grapheme => {
            surface.set_grapheme(0, 0, text, Cell::default());
        }
        SurfaceWriter::Str => {
            surface.write_str(0, 0, text, style.fg, style.bg, style.attr);
        }
        SurfaceWriter::StrAt => {
            surface.write_str_at(Point::new(0, 0), text, style.fg, style.bg, style.attr);
        }
        SurfaceWriter::StrClipped => {
            surface.write_str_clipped(0, 0, text, 64, style.fg, style.bg, style.attr);
        }
        SurfaceWriter::Styled => surface.write_text_styled(0, 0, text, &paint_style),
        SurfaceWriter::StyledClipped => {
            surface.write_text_styled_clipped(0, 0, text, 64, &paint_style);
        }
        SurfaceWriter::Enhanced => surface.write_text_enhanced(0, 0, text, 64, style),
        SurfaceWriter::Wrapped => {
            surface.write_text_wrapped(0, 0, text, 64, style);
        }
        SurfaceWriter::StyledText => surface.write_styled_text(0, 0, text, style),
        SurfaceWriter::Subview => {
            surface
                .subview_mut(0, 0, 0, 0, 64, 4)
                .write_text(0, 0, text, style);
        }
    }
    surface
}

/// Report the first way a painted surface leaks host control bytes, or `Ok(())`.
fn surface_host_controls(
    path: SurfaceWriter,
    surface: &Surface,
    attacker_text: &str,
) -> Result<(), String> {
    let blank = Surface::new(surface.dims().0, surface.dims().1);
    for y in 0..surface.dims().1 {
        for x in 0..surface.dims().0 {
            if surface.get(x, y) == blank.get(x, y) {
                continue;
            }
            if surface
                .grapheme(x, y)
                .chars()
                .any(|character| character.is_control())
            {
                return Err(format!(
                    "{path:?} retained a host control from {attacker_text:?}"
                ));
            }
        }
    }

    let mut diff = DiffWriter::new();
    diff.try_diff(&blank, surface, true).unwrap();
    if diff
        .output()
        .windows(attacker_text.len())
        .any(|window| window == attacker_text.as_bytes())
    {
        return Err(format!(
            "{path:?} emitted attacker-controlled bytes from {attacker_text:?}"
        ));
    }
    Ok(())
}

fn contains_paste(events: &[InputEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, InputEvent::Paste(_)))
}

fn control_character() -> impl Strategy<Value = char> {
    prop_oneof![
        (0_u32..=0x1f).prop_map(|value| char::from_u32(value).unwrap()),
        Just('\u{7f}'),
        (0x80_u32..=0x9f).prop_map(|value| char::from_u32(value).unwrap()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn surface_string_writers_do_not_retain_controls(control in control_character()) {
        let attacker_text = format!("left{control}right");
        for path in SURFACE_WRITERS {
            let surface = write_surface(*path, &attacker_text);
            assert_eq!(
                surface_host_controls(*path, &surface, &attacker_text),
                Ok(())
            );
        }
    }

    #[test]
    fn title_paths_reject_or_encode_controls(control in control_character()) {
        let attacker_title = format!("title{control}payload");

        let mut host = ManuallyDrop::new(HostTerminal::new().unwrap());
        assert!(host.set_title(&attacker_title).is_err());

        let mut terminal = Terminal::new(TerminalConfig::default());
        assert!(terminal.set_title(&attacker_title).is_err());

        let sequence = utils::set_title(&attacker_title);
        let payload = sequence.trim_start_matches("\u{1b}]0;").trim_end_matches('\u{7}');
        assert!(payload.chars().all(|character| !character.is_control()));

        let child_sequence = format!("\u{1b}]0;{attacker_title}\u{7}");
        for event in terminal.process_output(child_sequence.as_bytes()) {
            if let TerminalEvent::TitleChanged(title) = event {
                assert!(title.chars().all(|character| !character.is_control()));
            }
        }
        assert!(terminal.title().chars().all(|character| !character.is_control()));
    }

    #[test]
    fn unsolicited_osc_52_never_becomes_paste(
        bytes in prop::collection::vec(any::<u8>(), 0..128),
        use_st in any::<bool>(),
    ) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        let terminator = if use_st { "\u{1b}\\" } else { "\u{7}" };
        let input = format!("\u{1b}]52;c;{encoded}{terminator}");
        let events = EscapeSequenceParser::new().parse(input.as_bytes());
        assert!(!contains_paste(&events), "OSC 52 became a paste: {events:?}");
    }
}

#[test]
fn explicit_osc_and_csi_injection_is_not_retained() {
    for attacker_text in ["left\u{1b}[2Jright", "left\u{1b}]52;c;SGVsbG8=\u{7}right"] {
        for path in SURFACE_WRITERS {
            assert_eq!(
                surface_host_controls(*path, &write_surface(*path, attacker_text), attacker_text),
                Ok(())
            );
        }
    }
}

#[test]
fn ordinary_unicode_remains_renderable() {
    for path in SURFACE_WRITERS {
        let surface = write_surface(*path, "é");
        assert!(
            (0..surface.dims().1)
                .flat_map(|y| (0..surface.dims().0).map(move |x| (x, y)))
                .any(|(x, y)| surface.grapheme(x, y).contains('é')),
            "{path:?} discarded ordinary Unicode"
        );
    }
}

fn type_name(ty: &Type) -> Option<String> {
    if let Type::Path(path) = ty {
        path.path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
    } else {
        None
    }
}

fn path_contains_string(path: &syn::Path) -> bool {
    path.segments.iter().any(|segment| {
        matches!(segment.ident.to_string().as_str(), "str" | "String")
            || match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().any(|argument| {
                    matches!(argument, GenericArgument::Type(ty) if contains_string_type(ty))
                }),
                _ => false,
            }
    })
}

fn contains_string_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => contains_string_type(reference.elem.as_ref()),
        Type::Path(path) => path_contains_string(&path.path),
        Type::ImplTrait(implementation) => implementation.bounds.iter().any(|bound| {
            matches!(bound, TypeParamBound::Trait(bound) if path_contains_string(&bound.path))
        }),
        _ => false,
    }
}

fn accepts_string(arguments: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> bool {
    arguments.iter().any(|argument| match argument {
        FnArg::Receiver(_) => false,
        FnArg::Typed(argument) => contains_string_type(argument.ty.as_ref()),
    })
}

fn collect_rust_sources(directory: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if !path.ends_with("crates/reactive-tui-crossterm")
                && !path.ends_with("crates/reactive-tui-suprtui")
            {
                collect_rust_sources(&path, sources);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

fn inventory_impl(prefix: &str, block: &ItemImpl, paths: &mut BTreeSet<String>) {
    let Some(owner) = type_name(block.self_ty.as_ref()) else {
        return;
    };
    for member in &block.items {
        let ImplItem::Fn(function) = member else {
            continue;
        };
        let name = function.sig.ident.to_string();
        let surface_writer = matches!(owner.as_str(), "Surface" | "SurfaceSubview")
            && (name.starts_with("write") || name == "set_grapheme");
        if matches!(function.vis, Visibility::Public(_))
            && accepts_string(&function.sig.inputs)
            && (surface_writer || name.contains("title"))
        {
            paths.insert(format!("{prefix}::{owner}::{name}"));
        }
    }
}

fn inventory_title_field(prefix: &str, item: &ItemStruct, paths: &mut BTreeSet<String>) {
    for field in &item.fields {
        if field.ident.as_ref().is_some_and(|name| name == "title")
            && matches!(field.vis, Visibility::Public(_))
            && type_name(&field.ty).as_deref() == Some("String")
        {
            paths.insert(format!("{prefix}::{}::title", item.ident));
        }
    }
}

fn inventory_items(prefix: &str, items: &[Item], paths: &mut BTreeSet<String>) {
    for item in items {
        match item {
            Item::Fn(function)
                if matches!(function.vis, Visibility::Public(_))
                    && function.sig.ident.to_string().contains("title")
                    && accepts_string(&function.sig.inputs) =>
            {
                paths.insert(format!("{prefix}::{}", function.sig.ident));
            }
            Item::Impl(block) => inventory_impl(prefix, block, paths),
            Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    inventory_items(&format!("{prefix}::{}", module.ident), items, paths);
                }
            }
            Item::Struct(item) if matches!(item.vis, Visibility::Public(_)) => {
                inventory_title_field(prefix, item, paths);
            }
            _ => {}
        }
    }
}

#[test]
fn public_terminal_string_path_inventory_is_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    collect_rust_sources(&root.join("src"), &mut sources);
    let mut actual = BTreeSet::new();
    for source in sources {
        let prefix = source.strip_prefix(root).unwrap().display().to_string();
        let text = std::fs::read_to_string(source).unwrap();
        let syntax = syn::parse_file(&text).unwrap();
        inventory_items(&prefix, &syntax.items, &mut actual);
    }
    let expected = EXPECTED_STRING_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "public string path inventory changed");
}

#[test]
fn validator_rejects_safe_violating_fixtures() {
    let expected = EXPECTED_STRING_PATHS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let missing = EXPECTED_STRING_PATHS[1..]
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_ne!(
        missing, expected,
        "missing-writer fixture escaped inventory validation"
    );
    assert!("safe\u{1b}[2J".chars().any(char::is_control));
    assert!(contains_paste(&[InputEvent::Paste("unsolicited".into())]));
}
