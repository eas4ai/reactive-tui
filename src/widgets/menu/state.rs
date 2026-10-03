pub(super) fn step(selected: Option<usize>, count: usize, forward: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match selected {
        None => {
            if forward {
                0
            } else {
                count - 1
            }
        }
        Some(index) => {
            let index = index % count;
            if forward {
                (index + 1) % count
            } else if index == 0 {
                count - 1
            } else {
                index - 1
            }
        }
    })
}

pub(super) fn keep_visible(selected: Option<usize>, offset: &mut usize, maximum: usize) {
    let Some(selected) = selected else {
        *offset = 0;
        return;
    };
    let maximum = maximum.max(1);
    if selected < *offset {
        *offset = selected;
    } else if selected - *offset >= maximum {
        *offset = selected - (maximum - 1);
    }
}

pub(super) fn contains(rect: (u16, u16, u16, u16), point: (u16, u16)) -> bool {
    let (x, y, width, height) = rect;
    let (px, py) = point;
    px >= x
        && py >= y
        && u32::from(px) < u32::from(x) + u32::from(width)
        && u32::from(py) < u32::from(y) + u32::from(height)
}

#[cfg(test)]
mod tests {
    use crate::widgets::menu::{ContextMenuState, DialogMenuState, MenuBarState, PopupMenuState};
    #[test]
    fn public_menu_states_bound_oversized_selection_and_zero_windows() {
        let mut bar = MenuBarState {
            selected_index: Some(usize::MAX),
            submenu_selected_index: Some(usize::MAX),
            dropdown_scroll_offset: usize::MAX,
            ..Default::default()
        };
        bar.select_next(3);
        assert!(bar.selected_index.unwrap() < 3);
        bar.select_previous(0);
        assert_eq!(bar.selected_index, None);
        bar.select_next_submenu(3);
        assert!(bar.submenu_selected_index.unwrap() < 3);
        bar.update_submenu_scroll(0);
        assert_eq!(
            bar.dropdown_scroll_offset,
            bar.submenu_selected_index.unwrap()
        );
        let mut popup = PopupMenuState {
            selected_index: Some(usize::MAX),
            scroll_offset: usize::MAX,
            ..Default::default()
        };
        popup.select_next(3);
        assert!(popup.selected_index.unwrap() < 3);
        popup.update_scroll(0);
        assert_eq!(popup.scroll_offset, popup.selected_index.unwrap());
        popup.select_previous(0);
        assert_eq!(popup.selected_index, None);
        let mut dialog = DialogMenuState {
            selected_index: Some(usize::MAX),
            scroll_offset: usize::MAX,
            ..Default::default()
        };
        dialog.select_next(3);
        assert!(dialog.selected_index.unwrap() < 3);
        dialog.update_scroll(0);
        assert_eq!(dialog.scroll_offset, dialog.selected_index.unwrap());
        dialog.select_previous(0);
        assert_eq!(dialog.selected_index, None);
    }
    #[test]
    fn public_menu_rectangles_do_not_overflow_at_terminal_coordinate_limits() {
        let popup = PopupMenuState {
            popup_area: Some((65530, 65530, 20, 20)),
            ..Default::default()
        };
        assert!(popup.contains_point(65535, 65535));
        assert!(!popup.contains_point(0, 0));
        let dialog = DialogMenuState {
            dialog_area: Some((65530, 65530, 20, 20)),
            ..Default::default()
        };
        assert!(dialog.contains_point(65535, 65535));
        let context = ContextMenuState::default();
        assert!(context.is_in_trigger_area(65535, 65535, &[(65530, 65530, 20, 20)]));
    }
    #[test]
    fn public_long_press_movement_uses_unsigned_coordinate_distance() {
        let mut context = ContextMenuState::default();
        context.start_long_press(0, 0);
        assert!(!context.update_long_press(32768, 0, 0));
        assert!(context.press_start_time.is_none());
    }
}
