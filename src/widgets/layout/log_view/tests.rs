use super::*;
use crate::event::hit::Bounds;
use crate::event::types::{KeyCode, KeyEvent};
use crate::reactive::wake::{AppWaker, Scope};

/// `count` rows `Row 1`, `Row 2`... numbered from `first`.
fn rows(first: usize, count: usize) -> Vec<LogRow> {
    (first..first + count)
        .map(|n| LogRow::new(&format!("Row {n}")))
        .collect()
}

fn props(handle: &LogHandle) -> LogViewProps {
    LogViewProps {
        handle: handle.clone(),
        new_word: None,
        aria_label: None,
        width: None,
        height: None,
    }
}

/// A view of `handle` laid out at `width` by `height` cells, with the focus.
fn laid_out(handle: &LogHandle, width: f32, height: f32) -> (LogView, LogViewProps) {
    let mut props = props(handle);
    let mut view = LogView::new(props.clone());
    view.layout(
        LayoutInfo::from_bounds(Bounds::new(0.0, 0.0, width, height)),
        &mut props,
        &mut (),
    );
    view.handle_event(
        &Event::Focus(crate::event::types::FocusEvent {
            kind: FocusEventKind::Gained,
            timestamp: std::time::Instant::now(),
        }),
        &mut props,
        &mut (),
    );
    (view, props)
}

fn press(view: &mut LogView, props: &mut LogViewProps, code: KeyCode) {
    view.handle_event(&Event::Key(KeyEvent::new(code)), props, &mut ());
}

/// The first line in view, the greatest it can be, and the rows that arrived
/// since the view stopped following.
fn place(view: &LogView, props: &LogViewProps) -> (usize, usize, usize) {
    props.handle.read(|rows| {
        let geometry = view.measure(rows, props);
        (geometry.top, geometry.max_top, geometry.fresh)
    })
}

#[test]
fn nav_009_a_change_from_another_thread_wakes_the_app_that_shows_the_view() {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 3));
    let app = AppWaker::new();
    let (view, props) = laid_out(&handle, 40.0, 10.0);
    {
        let _scope = Scope::enter(&app);
        view.render(&props, &());
    }
    assert!(!app.is_pending(), "a render asks for no redraw");
    let other = handle.clone();
    std::thread::spawn(move || other.append(LogRow::new("From a worker")))
        .join()
        .unwrap();
    assert!(
        app.take().redraw,
        "NAV-009: a row appended on another thread wakes the App that shows the view"
    );
    for change in [
        Box::new(|handle: &LogHandle| handle.prepend(rows(0, 1))) as Box<dyn Fn(&LogHandle)>,
        Box::new(|handle: &LogHandle| handle.clear()),
    ] {
        {
            let _scope = Scope::enter(&app);
            view.render(&props, &());
        }
        change(&handle);
        assert!(app.take().redraw, "every change of the rows wakes the App");
    }
}

#[test]
fn nav_009_a_handle_no_view_shows_wakes_nothing_and_keeps_its_rows() {
    let handle = LogHandle::new();
    let app = AppWaker::new();
    handle.append(LogRow::new("Before any view"));
    assert!(!app.is_pending(), "a handle no view showed wakes nothing");

    let (view, props) = laid_out(&handle, 40.0, 10.0);
    {
        let _scope = Scope::enter(&app);
        view.render(&props, &());
    }
    // The App's next frame no longer shows the view.
    drop(Scope::enter(&app));
    handle.append(LogRow::new("After the view left"));
    assert!(
        !app.is_pending(),
        "a handle whose view the App's latest frame did not show wakes nothing"
    );
    assert_eq!(handle.len(), 2, "the handle keeps its rows");

    {
        let _scope = Scope::enter(&app);
        view.render(&props, &());
    }
    app.close();
    handle.append(LogRow::new("After the App ended"));
    assert!(!app.is_pending(), "an App that ended is never woken");
}

#[test]
fn nav_009_page_up_stops_the_following_and_counts_the_rows_that_arrive() {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 100));
    let (mut view, mut props) = laid_out(&handle, 40.0, 20.0);
    assert_eq!(
        place(&view, &props),
        (80, 80, 0),
        "the view follows its last row"
    );

    press(&mut view, &mut props, KeyCode::PageUp);
    assert_eq!(place(&view, &props), (60, 80, 0), "Page Up scrolls a page");
    handle.append_all(rows(101, 5));
    assert_eq!(
        place(&view, &props),
        (60, 85, 5),
        "rows appended while the view does not follow leave it where it was, counted"
    );
    handle.prepend(rows(0, 10));
    assert_eq!(
        place(&view, &props),
        (70, 95, 5),
        "rows prepended keep the rows in view on their lines and are not new"
    );
    press(&mut view, &mut props, KeyCode::End);
    assert_eq!(place(&view, &props), (95, 95, 0), "End follows again");
    handle.append_all(rows(106, 2));
    assert_eq!(
        place(&view, &props),
        (97, 97, 0),
        "a following view moves with new rows"
    );
}

#[test]
fn nav_009_scrolling_down_to_the_last_line_resumes_the_following() {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 30));
    let (mut view, mut props) = laid_out(&handle, 40.0, 10.0);
    press(&mut view, &mut props, KeyCode::Up);
    handle.append(LogRow::new("New"));
    assert_eq!(place(&view, &props), (19, 21, 1));
    press(&mut view, &mut props, KeyCode::Down);
    press(&mut view, &mut props, KeyCode::Down);
    assert_eq!(
        place(&view, &props),
        (21, 21, 0),
        "Down onto the last line follows again"
    );
}

#[test]
fn nav_009_a_cleared_log_is_followed_again() {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 30));
    let (mut view, mut props) = laid_out(&handle, 40.0, 10.0);
    press(&mut view, &mut props, KeyCode::Home);
    assert_eq!(place(&view, &props), (0, 20, 0));
    handle.clear();
    handle.append_all(rows(1, 12));
    assert_eq!(
        place(&view, &props),
        (2, 2, 0),
        "the row the view stood on is gone, so it follows the new rows"
    );
}

#[test]
fn nav_010_a_log_view_is_a_polite_log_with_its_offsets() {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 100));
    let (view, mut props) = laid_out(&handle, 40.0, 20.0);
    props.aria_label = Some("Build log".to_owned());
    let element = view.render(&props, &());
    let node = &element.metadata.accessibility.as_ref().unwrap().inner;
    assert_eq!(node.role(), Role::Log);
    assert_eq!(node.label(), Some("Build log"));
    assert_eq!(node.live(), Some(Live::Polite));
    assert_eq!(
        (node.scroll_y(), node.scroll_y_min(), node.scroll_y_max()),
        (Some(80.0), Some(0.0), Some(80.0))
    );
}
