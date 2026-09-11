//! A separately compiled consumer: only public application/backend interfaces.
use reactive_tui::{
    app::{App, RootComponent, RootUpdate},
    backend::{Backend, DebugBackend},
    component::{Element, ElementType, LayoutType},
    error::{ReactiveError, Result},
    event::types::Event,
    render::{NodeKey, PatchOp, RenderTree},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Default)]
struct Observed {
    patches: Vec<Vec<PatchOp>>,
    screens: Vec<String>,
    staged: Vec<PatchOp>,
    trees: Vec<Element>,
    shutdowns: usize,
}

struct LegacyBackend {
    observed: Arc<Mutex<Observed>>,
    screen: DebugBackend,
    fail: &'static str,
}

impl Backend for LegacyBackend {
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        if self.fail == "patch" {
            return Err(ReactiveError::terminal("entry point patch failure"));
        }
        let mut state = self.observed.lock().unwrap();
        state.staged.extend_from_slice(patches);
        state
            .trees
            .push(tree.root().unwrap().as_element().unwrap().clone());
        self.screen.apply_patches(patches, tree)
    }
    fn clear(&mut self) -> Result<()> {
        self.screen.clear()
    }
    fn present(&mut self) -> Result<()> {
        if self.fail == "present" {
            return Err(ReactiveError::terminal("entry point present failure"));
        }
        self.screen.present()?;
        let mut state = self.observed.lock().unwrap();
        let patches = std::mem::take(&mut state.staged);
        state.patches.push(patches);
        state.screens.push(self.screen.screen_content());
        Ok(())
    }
    fn size(&self) -> (u16, u16) {
        self.screen.size()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        Ok(None)
    }
    fn shutdown(&mut self) -> Result<()> {
        self.observed.lock().unwrap().shutdowns += 1;
        if self.fail == "shutdown" {
            return Err(ReactiveError::terminal("entry point shutdown failure"));
        }
        Ok(())
    }
}

struct Root {
    frames: Vec<Element>,
    index: usize,
    observed: Arc<Mutex<Observed>>,
    deadline: Instant,
    fail: bool,
}
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.frames[self.index].clone()
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn update(&mut self) -> Result<RootUpdate> {
        assert!(
            Instant::now() < self.deadline,
            "App stopped making frame progress"
        );
        if self.fail {
            return Err(ReactiveError::terminal("entry point root failure"));
        }
        let count = self.observed.lock().unwrap().screens.len();
        if count >= self.frames.len() {
            return Ok(RootUpdate::Exit);
        }
        if count > self.index {
            self.index = count;
            return Ok(RootUpdate::Redraw);
        }
        Ok(RootUpdate::Unchanged)
    }
}

fn frame(items: &[(&str, &str)]) -> Element {
    Element::layout(LayoutType::Flex)
        .with_key("root")
        .with_class("flex flex-col w-full h-full")
        .with_children(
            items
                .iter()
                .map(|(key, text)| Element::text(*text).with_key(*key).with_class("w-full h-1"))
                .collect(),
        )
}

fn run(frames: Vec<Element>, fail: &'static str) -> (Result<()>, Arc<Mutex<Observed>>) {
    let observed = Arc::new(Mutex::new(Observed::default()));
    let app = App::builder()
        .screen_reader(false)
        .backend(LegacyBackend {
            observed: observed.clone(),
            screen: DebugBackend::new(32, 8),
            fail,
        })
        .root(Root {
            frames,
            index: 0,
            observed: observed.clone(),
            deadline: Instant::now() + Duration::from_secs(3),
            fail: fail == "root",
        })
        .build()
        .unwrap();
    (app.run(), observed)
}

#[test]
fn initial_frame_uses_the_actual_root_insertion() {
    let (result, observed) = run(vec![frame(&[("a", "Alpha")])], "");
    result.unwrap();
    let state = observed.lock().unwrap();
    assert!(
        matches!(state.patches[0].as_slice(), [PatchOp::Insert {
        parent_key: None, index: 0, node_key
    }] if *node_key == NodeKey::named("root")),
        "fabricated initial patches: {:?}",
        state.patches[0]
    );
    assert!(state.screens[0].contains("Alpha"));
    assert_eq!(state.shutdowns, 1);
}

#[test]
fn unchanged_update_reorder_and_removal_use_the_last_presented_tree() {
    let first = frame(&[("a", "Alpha"), ("b", "Beta")]);
    let (result, observed) = run(
        vec![
            first.clone(),
            first,
            frame(&[("a", "Gamma"), ("b", "Beta")]),
            frame(&[("b", "Beta"), ("a", "Gamma")]),
            frame(&[("b", "Beta")]),
            frame(&[]),
        ],
        "",
    );
    result.unwrap();
    let state = observed.lock().unwrap();
    assert_eq!(state.screens.len(), 6);
    assert!(
        state.patches[1].is_empty(),
        "unchanged frame reapplied patches: {:?}",
        state.patches[1]
    );
    assert!(state.patches[2]
        .iter()
        .any(|p| matches!(p, PatchOp::Update { .. })));
    assert!(state.screens[2].contains("Gamma"));
    assert!(!state.screens[2].contains("Alpha"));
    assert!(state.screens[3].find("Beta").unwrap() < state.screens[3].find("Gamma").unwrap());
    assert!(state.screens[4].contains("Beta"));
    assert!(!state.screens[4].contains("Gamma"));
    assert!(state.screens[5].trim().is_empty());
    assert!(
        state
            .trees
            .iter()
            .any(|tree| tree.children.iter().any(|child| {
                matches!(&child.element_type, ElementType::Text(text) if text == "Gamma")
            })),
        "backend never received the updated tree"
    );
    assert_eq!(state.shutdowns, 1);
}

#[test]
fn patch_present_root_and_shutdown_errors_are_returned() {
    for failure in ["patch", "present", "root", "shutdown"] {
        let (result, observed) = run(vec![frame(&[("a", "Alpha")])], failure);
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains(&format!("entry point {failure} failure")),
            "{error}"
        );
        assert_eq!(observed.lock().unwrap().shutdowns, 1);
        if matches!(failure, "patch" | "present") {
            assert!(observed.lock().unwrap().screens.is_empty());
        }
    }
}
