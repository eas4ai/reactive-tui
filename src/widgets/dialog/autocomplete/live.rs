use super::*;
use crate::{
    accessibility::{Node, Role},
    component::{Component, LayoutInfo, LifecycleEvent, Props},
    event::router::EventResult,
    reactive::{
        component_scope,
        scheduler::{Scheduler, TimerId},
        ThreadSafeSignal,
    },
    widgets::{
        display::modal::{ModalButton, ModalButtonAction, ModalProps, ModalSize},
        TextInput, TextInputProps,
    },
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub id: DialogId,
    pub options: AutocompleteDialogOptions,
    pub value: String,
    pub bounds: Rect,
    pub theme: DialogTheme,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Runtime {
    activity: super::super::frame::Activity,
    options: Mutex<Arc<AutocompleteDialogOptions>>,
    value: ThreadSafeSignal<String>,
    suggestions: ThreadSafeSignal<Vec<AutocompleteSuggestion>>,
    selected: ThreadSafeSignal<Option<usize>>,
    first: ThreadSafeSignal<usize>,
    reveal: AtomicBool,
    rows: Mutex<HashMap<usize, LayoutInfo>>,
    expanded: ThreadSafeSignal<bool>,
    visible: ThreadSafeSignal<bool>,
    loading: ThreadSafeSignal<bool>,
    error: ThreadSafeSignal<Option<String>>,
    scheduler: Option<Arc<Scheduler>>,
    timer: Mutex<Option<TimerId>>,
    job: Mutex<Option<super::super::http::Job>>,
    width: ThreadSafeSignal<Option<usize>>,
}

impl Runtime {
    fn options(&self) -> Arc<AutocompleteDialogOptions> {
        self.options.lock().unwrap().clone()
    }
    fn cancel(&self) {
        let timer = self.timer.lock().unwrap().take();
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, timer) {
            scheduler.cancel_timer(timer);
        }
        let job = self.job.lock().unwrap().take();
        drop(job);
        self.loading.set(false);
    }
    fn finish(&self, result: DialogResult) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        self.cancel();
        self.visible.set(false);
        if let Some(callback) = &self.options().on_close {
            callback(result);
        }
    }
    fn select(&self, index: usize) {
        if !self.visible.get() || !self.activity.active() || !self.expanded.get() {
            return;
        }
        let Some(suggestion) = self.suggestions.get().get(index).cloned() else {
            return;
        };
        if self
            .options()
            .on_select
            .as_ref()
            .is_some_and(|callback| !callback(&suggestion.value))
        {
            return;
        }
        self.value.set(suggestion.value.clone());
        self.finish(DialogResult::Selected(suggestion.value));
    }
    fn submit(&self) {
        if let Some(index) = self.selected.get().filter(|_| self.expanded.get()) {
            self.select(index);
        } else {
            self.finish(DialogResult::Confirmed(Some(self.value.get())));
        }
    }
    fn navigate(&self, down: bool) -> bool {
        let count = self.suggestions.get().len();
        if count == 0 {
            return false;
        }
        self.expanded.set(true);
        self.reveal.store(true, Ordering::Release);
        self.selected.set(Some(match self.selected.get() {
            Some(index) if down => (index + 1) % count,
            Some(0) | None if !down => count - 1,
            Some(index) => index.saturating_sub(1),
            None => 0,
        }));
        if let Some(selected) = self.selected.get() {
            let clipped = self
                .rows
                .lock()
                .unwrap()
                .get(&selected)
                .is_none_or(suggestion_is_clipped);
            if selected < self.first.get() || clipped {
                self.first.set(selected);
            }
        }
        true
    }
    fn changed(self: &Arc<Self>, value: String) {
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        self.value.set(value.clone());
        if let Some(callback) = &self.options().on_change {
            callback(&value);
        }
        self.refresh();
    }
    fn refresh(self: &Arc<Self>) {
        self.cancel();
        self.suggestions.set(Vec::new());
        self.selected.set(None);
        self.expanded.set(false);
        self.error.set(None);
        if !self.visible.get() || !self.activity.active() {
            return;
        }
        let options = self.options();
        let query = self.value.get();
        if query.graphemes(true).count() < options.autocomplete.min_chars {
            return;
        }
        if options.autocomplete.suggestions_url.is_none() {
            self.accept(filtered_suggestions(&options.autocomplete, &query));
        } else {
            self.loading.set(true);
            self.schedule(options.autocomplete.debounce_delay, false);
        }
    }
    fn accept(&self, mut suggestions: Vec<AutocompleteSuggestion>) {
        suggestions.truncate(self.options().autocomplete.max_suggestions);
        self.first.set(0);
        self.rows.lock().unwrap().clear();
        self.reveal.store(true, Ordering::Release);
        self.selected.set((!suggestions.is_empty()).then_some(0));
        self.expanded.set(true);
        self.suggestions.set(suggestions);
        self.loading.set(false);
    }
    fn schedule(self: &Arc<Self>, delay: Duration, polling: bool) {
        if Instant::now().checked_add(delay).is_none() {
            self.error.set(Some(
                "Suggestion delay exceeds the supported clock range".into(),
            ));
            self.loading.set(false);
            return;
        }
        let Some(scheduler) = &self.scheduler else {
            self.error
                .set(Some("HTTP suggestions require the App scheduler".into()));
            self.loading.set(false);
            return;
        };
        let owner = Arc::downgrade(self);
        *self.timer.lock().unwrap() = Some(scheduler.schedule_timeout(delay, move || {
            if let Some(owner) = owner.upgrade() {
                owner.timer.lock().unwrap().take();
                if owner.visible.get() && owner.activity.active() {
                    if polling {
                        owner.poll();
                    } else {
                        owner.request();
                    }
                }
            }
        }));
    }
    fn request(self: &Arc<Self>) {
        let options = self.options();
        let Some(url) = &options.autocomplete.suggestions_url else {
            return;
        };
        match super::super::http::Job::start(
            url,
            "query",
            &self.value.get(),
            options.autocomplete.headers.as_ref(),
        ) {
            Ok(job) => {
                *self.job.lock().unwrap() = Some(job);
                self.schedule(Duration::from_millis(10), true);
            }
            Err(error) => {
                self.error.set(Some(error));
                self.loading.set(false);
            }
        }
    }
    fn poll(self: &Arc<Self>) {
        let result = self
            .job
            .lock()
            .unwrap()
            .as_mut()
            .and_then(super::super::http::Job::poll);
        let Some(result) = result else {
            self.schedule(Duration::from_millis(10), true);
            return;
        };
        self.job.lock().unwrap().take();
        match result.and_then(|bytes| parse_suggestions(&bytes)) {
            Ok(suggestions) => self.accept(suggestions),
            Err(error) => {
                self.error.set(Some(error));
                self.loading.set(false);
            }
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn parse_suggestions(bytes: &[u8]) -> Result<Vec<AutocompleteSuggestion>, String> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Entry {
        Text(String),
        Suggestion(AutocompleteSuggestion),
    }
    serde_json::from_slice::<Vec<Entry>>(bytes)
        .map(|entries| {
            entries
                .into_iter()
                .map(|entry| match entry {
                    Entry::Text(value) => AutocompleteSuggestion::simple(&value),
                    Entry::Suggestion(value) => value,
                })
                .collect()
        })
        .map_err(|_| {
            "Invalid suggestion response: expected an array of strings or suggestion objects".into()
        })
}

pub(super) struct LiveAutocomplete {
    runtime: Arc<Runtime>,
    seed: String,
    id: DialogId,
    layout: Option<LayoutInfo>,
}
impl Component for LiveAutocomplete {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let runtime = Arc::new(Runtime {
            activity: super::super::frame::activity(),
            options: Mutex::new(Arc::new(props.options)),
            value: ThreadSafeSignal::new(props.value.clone()),
            suggestions: ThreadSafeSignal::new(Vec::new()),
            selected: ThreadSafeSignal::new(None),
            first: ThreadSafeSignal::new(0),
            reveal: AtomicBool::new(true),
            rows: Mutex::new(HashMap::new()),
            expanded: ThreadSafeSignal::new(false),
            visible: ThreadSafeSignal::new(true),
            loading: ThreadSafeSignal::new(false),
            error: ThreadSafeSignal::new(None),
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            timer: Mutex::new(None),
            job: Mutex::new(None),
            width: ThreadSafeSignal::new(None),
        });
        runtime.refresh();
        Self {
            runtime,
            seed: props.value,
            id: props.id,
            layout: None,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        let previous = self.runtime.options();
        let before = &previous.autocomplete;
        let after = &props.options.autocomplete;
        let query_changed = before.suggestions_url != after.suggestions_url
            || before.headers != after.headers
            || before.min_chars != after.min_chars
            || before.max_suggestions != after.max_suggestions
            || before.debounce_delay != after.debounce_delay
            || before.static_suggestions != after.static_suggestions;
        *self.runtime.options.lock().unwrap() = Arc::new(props.options.clone());
        let seed_changed = self.seed != props.value || self.id != props.id;
        if seed_changed {
            self.runtime.value.set(props.value.clone());
            self.seed = props.value.clone();
        }
        if self.id != props.id {
            self.runtime.visible.set(true);
            self.id = props.id;
        }
        // Filter callback replacement affects static results only. Callback identity
        // must not cancel an HTTP request when a parent recreates its closures.
        let filter_changed = after.suggestions_url.is_none()
            && !crate::widgets::display::overlay::same_callback(
                &before.filter_function,
                &after.filter_function,
            );
        if seed_changed || query_changed {
            self.runtime.refresh();
        } else if filter_changed {
            let suggestions = filtered_suggestions(after, &self.runtime.value.get());
            if self.runtime.suggestions.get() != suggestions {
                self.runtime.accept(suggestions);
            }
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self.layout != Some(layout);
        self.layout = Some(layout);
        if changed {
            self.runtime.reveal.store(true, Ordering::Release);
        }
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        if !self.runtime.activity.active() {
            self.runtime.cancel();
        }
        let options = &props.options;
        let owner = self.runtime.clone();
        let mut input = Element::typed_with::<TextInput>(
            TextInputProps {
                value: owner.value.get(),
                placeholder: options.autocomplete.placeholder.clone(),
                width: None,
                ..Default::default()
            },
            move |props| {
                let changed = owner.clone();
                let submitted = owner.clone();
                TextInput::new(props)
                    .with_on_change(move |value| changed.changed(value))
                    .with_on_submit(move |_| submitted.submit())
            },
        )
        .with_key("input")
        .auto_focus()
        .with_accessibility_label(&options.prompt)
        .class(
            options
                .css_classes
                .get("input")
                .map_or("w-full", String::as_str),
        );
        let owner = self.runtime.clone();
        input.metadata.capture_events.push(Arc::new(move |event| {
            if let Event::Key(key) = event {
                if key.kind == crate::event::types::KeyEventKind::Release {
                    return EventResult::Ignored;
                }
                match key.code {
                    KeyCode::Up if owner.navigate(false) => return EventResult::Consumed,
                    KeyCode::Down if owner.navigate(true) => return EventResult::Consumed,
                    KeyCode::Tab if owner.expanded.get() && owner.selected.get().is_some() => {
                        owner.submit();
                        return EventResult::Consumed;
                    }
                    KeyCode::Escape if owner.expanded.get() => {
                        owner.expanded.set(false);
                        owner.selected.set(None);
                        return EventResult::Consumed;
                    }
                    _ => {}
                }
            }
            EventResult::Ignored
        }));
        let mut content = vec![Element::text(&options.prompt), input];
        if self.runtime.loading.get() {
            content.push(Element::text("Loading...").with_accessibility(Node::new(Role::Status)));
        }
        if let Some(error) = self.runtime.error.get() {
            content.push(
                Element::text(error)
                    .class("text-red-500 whitespace-pre-wrap w-full")
                    .with_accessibility(Node::new(Role::Alert)),
            );
        }
        let suggestions = self.runtime.suggestions.get();
        if self.runtime.expanded.get() {
            let mut rows = Vec::new();
            if suggestions.is_empty() {
                content.push(
                    Element::text("No suggestions").with_accessibility(Node::new(Role::Status)),
                );
            }
            for (index, suggestion) in suggestions
                .iter()
                .enumerate()
                .skip(self.runtime.first.get())
            {
                let selected = self.runtime.selected.get() == Some(index);
                let child =
                    render_suggestion(suggestion, &options.autocomplete, &self.runtime.value.get());
                let owner = self.runtime.clone();
                let mut accessible = Node::new(Role::ListBoxOption);
                accessible.set_label(
                    suggestion
                        .display
                        .as_ref()
                        .unwrap_or(&suggestion.value)
                        .clone(),
                );
                accessible.set_selected(selected);
                accessible.set_clickable();
                let mut row = crate::builder::div()
                    .class(if selected {
                        "flex-col w-full bg-blue-700 text-white"
                    } else {
                        "flex-col w-full"
                    })
                    .child(child)
                    .on_click(move || owner.select(index))
                    .build()
                    .with_key(format!("suggestion-{index}"))
                    .with_accessibility(accessible);
                let measured = self.runtime.clone();
                row.metadata.layout.push(Arc::new(move |layout| {
                    measured.rows.lock().unwrap().insert(index, layout);
                    if measured.selected.get() != Some(index)
                        || !measured.reveal.load(Ordering::Acquire)
                    {
                        return false;
                    }
                    if suggestion_is_clipped(&layout) && index > measured.first.get() {
                        measured.first.set(index);
                        return true;
                    }
                    measured.reveal.store(false, Ordering::Release);
                    false
                }));
                rows.push(row);
            }
            if !rows.is_empty() {
                let mut list = crate::builder::div()
                    .class("flex-col w-full")
                    .children(rows)
                    .build()
                    .with_accessibility(Node::new(Role::ListBox))
                    .with_accessibility_label("Suggestions");
                let owner = self.runtime.clone();
                list.metadata.capture_events.push(Arc::new(move |event| {
                    if let Event::Mouse(mouse) = event {
                        if mouse.kind == MouseEventKind::Wheel {
                            let delta = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                                Some(
                                    crate::event::types::WheelDelta::Lines { y, .. }
                                    | crate::event::types::WheelDelta::Pixels { y, .. },
                                ) => f64::from(*y),
                                None => 0.0,
                            };
                            if delta.is_finite() && delta != 0.0 {
                                let count = owner.suggestions.get().len();
                                let amount = delta.abs().ceil().max(1.0) as usize;
                                let first = owner.first.get();
                                owner.first.set(if delta < 0.0 {
                                    first.saturating_sub(amount)
                                } else {
                                    first.saturating_add(amount).min(count.saturating_sub(1))
                                });
                                owner.reveal.store(false, Ordering::Release);
                                return EventResult::Consumed;
                            }
                        }
                    }
                    EventResult::Ignored
                }));
                content.push(list);
            }
        }
        let mut content = crate::builder::div().class("flex-col").children(content);
        if let Some(width) = self.runtime.width.get() {
            content =
                content.styles(crate::layout::style::StyleBuilder::new().width_px(width as f32));
        }
        let mut content = content.build();
        let measured = self.runtime.clone();
        content.metadata.layout.push(Arc::new(move |layout| {
            let width = Some(layout.clip.width.max(1.0).floor() as usize);
            let changed = measured.width.get() != width;
            if changed {
                measured.width.set(width);
            }
            changed
        }));
        let position = match super::super::frame::position(&options.position, self.layout) {
            Ok(position) => position,
            Err(error) => return Element::text(error),
        };
        let mut ok = ModalButton::ok();
        ok.autofocus = false;
        ok.action = ModalButtonAction::Custom("ok".into());
        let mut cancel = ModalButton::cancel();
        cancel.action = ModalButtonAction::Custom("cancel".into());
        let submitted = self.runtime.clone();
        let closed = self.runtime.clone();
        let mut modal = ModalProps {
            visible: self.runtime.visible.get(),
            title: Some(options.title.clone()),
            content: Some(content),
            buttons: vec![cancel, ok],
            position,
            width: options.size.map_or(ModalSize::Fixed(40), |size| {
                ModalSize::Fixed(size.width.min(u16::MAX as usize) as u16)
            }),
            height: options.size.map_or(ModalSize::Auto, |size| {
                ModalSize::Fixed(size.height.min(u16::MAX as usize) as u16)
            }),
            focus_trap: options.modal,
            backdrop_clickable: options.backdrop_closable,
            keyboard_navigation: true,
            backdrop_style: options.modal.then(|| props.theme.backdrop_color.clone()),
            modal_style: Some(format!(
                "{} {} {}",
                props.theme.dialog_bg,
                props.theme.border_style,
                options.css_classes.get("dialog").map_or("", String::as_str)
            )),
            header_style: Some(props.theme.title_style.clone()),
            content_style: options.css_classes.get("content").cloned(),
            on_button_click: Some(Arc::new(move |id| {
                if id == "ok" {
                    submitted.submit();
                } else {
                    submitted.finish(DialogResult::Cancelled);
                }
            })),
            on_close: Some(Arc::new(move |_| closed.finish(DialogResult::Cancelled))),
            ..Default::default()
        };
        super::super::frame::apply_bounds(&mut modal, props.bounds);
        super::super::frame::modal(
            modal,
            options.escape_closable,
            crate::accessibility::Role::Dialog,
        )
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.runtime.visible.set(false);
            self.runtime.cancel();
        }
    }
}

fn suggestion_is_clipped(layout: &LayoutInfo) -> bool {
    let across = layout.transform[1] * layout.size.0;
    let down = layout.transform[3] * layout.size.1;
    let top = layout.transform[5] + across.min(0.0) + down.min(0.0);
    let bottom = layout.transform[5] + across.max(0.0) + down.max(0.0);
    top < layout.clip.y - 0.5 || bottom > layout.clip.y + layout.clip.height + 0.5
}

fn render_suggestion(
    suggestion: &AutocompleteSuggestion,
    config: &AutocompleteConfig,
    query: &str,
) -> Element {
    if let Some(renderer) = &config.suggestion_renderer {
        return renderer(suggestion);
    }
    let display = suggestion.display.as_ref().unwrap_or(&suggestion.value);
    let mut title = Vec::new();
    if let Some(icon) = &suggestion.icon {
        title.push(Element::text(format!("{icon} ")));
    }
    let mut previous = 0;
    for (start, end) in match_ranges(display, query)
        .into_iter()
        .filter(|_| config.highlight_matches)
    {
        if previous < start {
            title.push(Element::text(&display[previous..start]));
        }
        title.push(Element::text(&display[start..end]).class("font-bold underline"));
        previous = end;
    }
    if previous < display.len() {
        title.push(Element::text(&display[previous..]));
    }
    let mut children = vec![crate::builder::div()
        .class("flex-row")
        .children(title)
        .build()];
    if config.show_descriptions {
        if let Some(description) = &suggestion.description {
            children.push(Element::text(description).class("whitespace-pre-wrap w-full"));
        }
    }
    crate::builder::div()
        .class("flex-col w-full")
        .children(children)
        .build()
}

// Lowercase offsets may differ from original offsets. Record whole grapheme
// boundaries once, then search without rescanning each suffix or splitting glyphs.
fn match_ranges(display: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let query = query.to_lowercase();
    let mut folded = String::new();
    let mut boundaries = Vec::new();
    for (offset, grapheme) in display.grapheme_indices(true) {
        boundaries.push((folded.len(), offset));
        folded.push_str(&grapheme.to_lowercase());
    }
    boundaries.push((folded.len(), display.len()));
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(found) = folded[cursor..].find(&query) {
        let start = cursor + found;
        let end = start + query.len();
        if let (Ok(start), Ok(end)) = (
            boundaries.binary_search_by_key(&start, |boundary| boundary.0),
            boundaries.binary_search_by_key(&end, |boundary| boundary.0),
        ) {
            result.push((boundaries[start].1, boundaries[end].1));
        }
        cursor = end;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestion_highlighting_preserves_expanded_case_and_graphemes() {
        assert_eq!(match_ranges("İxİ", "i\u{307}"), [(0, 2), (3, 5)]);
        assert!(match_ranges("İe\u{301}👩‍💻", "i").is_empty());
        assert!(match_ranges("e\u{301}", "e").is_empty());
        assert!(match_ranges("👩‍💻", "💻").is_empty());
        assert_eq!(match_ranges("Alpha ALPHA", "alpha"), [(0, 5), (6, 11)]);
        assert!(match_ranges("Alpha", "").is_empty());
    }

    #[test]
    fn suggestions_require_a_value_and_preserve_metadata() {
        let suggestions = parse_suggestions(
            br#"["plain",{"value":"id","display":"label","metadata":{"kind":"result"}}]"#,
        )
        .unwrap();
        assert_eq!(suggestions[0].value, "plain");
        assert_eq!(suggestions[1].display.as_deref(), Some("label"));
        assert_eq!(suggestions[1].metadata["kind"], "result");
        for invalid in [
            "{}",
            "[null]",
            "[{\"display\":\"missing\"}]",
            "[5]",
            "not json",
        ] {
            assert!(parse_suggestions(invalid.as_bytes()).is_err(), "{invalid}");
        }
    }
}
