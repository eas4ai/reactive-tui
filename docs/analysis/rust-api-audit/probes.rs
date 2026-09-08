use reactive_tui::{component, event};
use reactive_tui::component::{Component, ComponentInstance, Element, ElementType, FocusProps, LayoutType};
use reactive_tui::component::props::EmptyProps;
use reactive_tui::reactive::{Hooks, use_signal, use_memo};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize,Ordering}};
use std::time::Duration;
#[path = "__REPO_ROOT__/src/app/focus_manager.rs"]
mod app_focus;

#[reactive_tui::component]
fn AuditCounter(hooks: &Hooks) -> Element {
 let signal = use_signal(hooks, 0_i32);
 signal.set(signal.get()+1);
 Element::text(signal.get().to_string())
}
#[test]
fn macro_hook_slots_do_not_reset_between_renders() {
 let instance=ComponentInstance::<AuditCounter>::new(EmptyProps);
 assert_eq!(instance.render().element_type,ElementType::Text("1".into()));
 assert_eq!(instance.render().element_type,ElementType::Text("1".into()));
 let hooks=Hooks::new();let s=use_signal(&hooks,0);s.set(5);hooks.reset();
 assert_eq!(use_signal(&hooks,0).get(),5);
 println!("REPRODUCED: macro renders 1,1; explicit hook reset preserves state=5");
}
#[test]
fn memo_returns_stale_value_after_recompute() {
 let hooks=Hooks::new();let input=Arc::new(AtomicUsize::new(1));
 let a=input.clone();assert_eq!(use_memo(&hooks,move||a.load(Ordering::SeqCst)).get(),1);
 hooks.reset();input.store(2,Ordering::SeqCst);
 let a=input.clone();assert_eq!(use_memo(&hooks,move||a.load(Ordering::SeqCst)).get(),1);
 println!("REPRODUCED: memo input becomes 2 but returned signal remains 1");
}
#[test]
fn focus_identity_changes_and_trapped_autofocus_fails() {
 let child=Element::text("button").with_key("stable").with_focus(FocusProps{auto_focus:true,..Default::default()});
 let mut focus=app_focus::FocusManager::new();
 focus.process_element_tree(&child,event::router::NodeId::new());let first=*focus.current_focus().unwrap();
 focus.process_element_tree(&child,event::router::NodeId::new());assert_ne!(first,*focus.current_focus().unwrap());
 let root=Element::layout(LayoutType::Flex).with_focus(FocusProps{trap_focus:true,..Default::default()}).child(child);
 let mut trapped=app_focus::FocusManager::new();trapped.process_element_tree(&root,event::router::NodeId::new());
 assert!(trapped.current_focus().is_none());
 println!("REPRODUCED: keyed focus ID changes; trapped child autofocus is absent");
}
#[test]
fn builder_drops_callback_before_element_is_dropped() {
 let token=Arc::new(());let captured=token.clone();
 let element=reactive_tui::builder::button().on_click(move||{let _=&captured;}).build();
 assert_eq!(Arc::strong_count(&token),1);assert!(element.class.unwrap().contains("interactive"));
 println!("REPRODUCED: on_click discards captured handler and retains only class");
}
#[test]
fn typed_keyframes_step_instead_of_interpolating() {
 use reactive_tui::animation::keyframes::{KeyframeAnimation,TypedKeyframe};
 let a=KeyframeAnimation::from_typed(vec![TypedKeyframe{offset:0.,value:0_f32,easing:None},TypedKeyframe{offset:1.,value:10.,easing:None}],Duration::from_secs(1));
 assert_eq!(a.get_value_at_time(0.5),Some(0.));assert_eq!(a.get_value_at_time(1.),Some(10.));
 println!("REPRODUCED: 0 to 10 midpoint is 0 rather than interpolated 5");
}
#[derive(Clone,Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Capture {
 fn write(&mut self,b:&[u8])->std::io::Result<usize>{self.0.lock().unwrap().extend_from_slice(b);Ok(b.len())}
 fn flush(&mut self)->std::io::Result<()>{Ok(())}
}
static RENDERS:AtomicUsize=AtomicUsize::new(0);
struct Registered;
impl Component for Registered {
 type Props=EmptyProps;type State=();
 fn new(_:EmptyProps)->Self{Self}
 fn render(&self,_:&EmptyProps,_:&())->Element{RENDERS.fetch_add(1,Ordering::SeqCst);Element::text("CHILD_OUTPUT").class("w-full h-1")}
}
struct Root{wake:Option<reactive_tui::app::AppWaker>}
impl reactive_tui::app::RootComponent for Root {
 fn attach_waker(&mut self,w:reactive_tui::app::AppWaker){self.wake=Some(w)}
 fn render(&self)->Element{self.wake.as_ref().unwrap().request_stop();Element::component_with_props("AuditRegistered",EmptyProps).class("w-full h-1")}
}
#[test]
fn app_mounts_named_component_without_rendering_its_output() {
 use reactive_tui::backend::{Backend,SuprTuiBackend};
 reactive_tui::component::registry::register_component::<Registered>("AuditRegistered").unwrap();
 let output=Capture::default();let backend=SuprTuiBackend::with_writer(20,3,output.clone()).unwrap();
 reactive_tui::app::App::builder().backend(backend).root(Root{wake:None}).build().unwrap().run().unwrap();
 assert_eq!(RENDERS.load(Ordering::SeqCst),0);
 let text=String::from_utf8_lossy(&output.0.lock().unwrap()).to_string();assert!(!text.contains("CHILD_OUTPUT"));
 let control=Capture::default();let mut b=SuprTuiBackend::with_writer(20,3,control.clone()).unwrap();
 b.render_frame(&Element::text("CONTROL").class("w-full h-1")).unwrap();b.present().unwrap();
 let mut parser=vt100::Parser::new(3,20,0);parser.process(&control.0.lock().unwrap());
 assert!(parser.screen().contents().contains("CONTROL"), "control: {:?}",parser.screen().contents());
 println!("REPRODUCED: App component render calls=0; plain text control paints correctly");
}

#[test]
fn focus_style_applies_without_focus_and_uppercase_does_not_transform() {
 use reactive_tui::backend::{Backend,SuprTuiBackend};
 let control=Capture::default();let mut b=SuprTuiBackend::with_writer(20,3,control.clone()).unwrap();
 b.render_frame(&Element::text("lower").class("w-full h-1 bg-black focus:bg-red-500 uppercase")).unwrap();b.present().unwrap();
 let mut parser=vt100::Parser::new(3,20,0);parser.process(&control.0.lock().unwrap());
 assert_eq!(parser.screen().cell(0,0).unwrap().bgcolor(),vt100::Color::Rgb(239,68,68));
 assert!(parser.screen().contents().contains("lower"));
 println!("REPRODUCED: unfocused cell is red; uppercase leaves lower-case text unchanged");
}

#[test]
fn text_editor_insert_text_counts_bytes_but_backspace_counts_chars() {
 let mut editor=reactive_tui::editor::TextEditor::new();editor.insert_text("界");editor.delete_backward();
 assert_eq!(editor.content(),"界");
 let mut control=reactive_tui::editor::TextEditor::new();control.insert_text("a");control.delete_backward();assert_eq!(control.content(),"");
 println!("REPRODUCED: backspace after insert_text(界) leaves 界; ASCII control deletes correctly");
}
