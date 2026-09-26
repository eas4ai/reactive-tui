use reactive_tui::{
    backend::{direct_tty::DirectTtyBackend, Backend, CrosstermBackend},
    component::Element,
    event::types::{Event, KeyCode},
    render::{tree::ElementNode, NodeKey, PatchOp, RenderTree},
};
use std::io::{self, Write};

fn marker(name: &str) {
    let mut output = io::stdout().lock();
    write!(output, "\x1b]900;{name}\x07").unwrap();
    output.flush().unwrap();
}

fn tree(text: &str) -> RenderTree {
    let mut tree = RenderTree::new();
    tree.set_root(Box::new(ElementNode::new(
        Element::text(text)
            .with_key("manual")
            .with_class("w-full h-1"),
    )));
    tree
}

fn patches(backend: &mut impl Backend) {
    let key = NodeKey::named("manual");
    backend
        .apply_patches(
            &[PatchOp::Insert {
                parent_key: None,
                index: 0,
                node_key: key.clone(),
            }],
            &tree("FIRST"),
        )
        .unwrap();
    backend.present().unwrap();
    marker("FIRST");
    backend.present().unwrap();
    marker("UNCHANGED");
    backend
        .apply_patches(
            &[PatchOp::Update {
                node_key: key.clone(),
            }],
            &tree("SECOND"),
        )
        .unwrap();
    backend.present().unwrap();
    marker("SECOND");
    backend
        .apply_patches(&[PatchOp::Remove { node_key: key }], &RenderTree::new())
        .unwrap();
    backend.present().unwrap();
    marker("REMOVED");
}

fn crossterm() {
    let mut backend = CrosstermBackend::new().unwrap();
    patches(&mut backend);
    backend
        .render_full(&Element::text("STABLE").with_class("w-full h-1"))
        .unwrap();
    backend.present().unwrap();
    marker("BASE");
    backend.present().unwrap();
    marker("INCREMENTAL");
    backend.set_use_render_ops(false);
    backend.present().unwrap();
    marker("FULL");
    backend.set_use_render_ops(true);
    backend.present().unwrap();
    marker("INCREMENTAL_AGAIN");
    backend.set_debug_overlay(true);
    backend.present().unwrap();
    marker("DEBUG");
    backend.set_debug_overlay(false);
    backend.present().unwrap();
    marker("NO_DEBUG");
    backend.shutdown().unwrap();
    backend.shutdown().unwrap();
    assert!(backend.present().is_err(), "closed backend accepted output");
}

fn direct() {
    let mut backend = DirectTtyBackend::new().unwrap();
    patches(&mut backend);
    marker("READY");
    for expected in ['x', 'y', 'z'] {
        assert!(
            matches!(backend.poll_event(Some(1000)).unwrap(), Some(Event::Key(key))
            if key.code == KeyCode::Char(expected)),
            "native batch lost {expected}"
        );
    }
    marker("BATCH");
    backend.shutdown().unwrap();
    backend.shutdown().unwrap();
    assert!(backend.poll_event(Some(0)).is_err());
}

#[cfg(unix)]
fn unix_handles() {
    use reactive_tui::platform::{unix::UnixTty, PlatformTty};
    use std::time::Duration;
    for index in 0..2 {
        let original = UnixTty::init().unwrap();
        let retained = original.clone();
        drop(original);
        assert_eq!(retained.size().unwrap(), (32, 8));
        assert_eq!(retained.write_raw(b" ").unwrap(), 1);
        marker(&format!("RAW{index}"));
        let mut byte = [0];
        assert_eq!(
            retained
                .read(&mut byte, Some(Duration::from_secs(2)))
                .unwrap(),
            1
        );
        assert_eq!(byte[0], b'x');
        retained.restore().unwrap();
        retained.restore().unwrap();
        drop(retained);
        marker(&format!("CLOSED{index}"));
        // Wait for the host to inspect canonical mode before the next owner.
        io::stdin().read_line(&mut String::new()).unwrap();
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("crossterm") => crossterm(),
        Some("direct") => direct(),
        #[cfg(unix)]
        Some("unix") => unix_handles(),
        _ => panic!("unknown manual entry-point fixture"),
    }
    println!("ENTRY_POINT_CLEAN_EXIT");
}
