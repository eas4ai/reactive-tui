"""Genuine reader checks for data controls, separate from the input/menu run."""
import time


def exercise(root, keyboard, wait, named, state, click, speech, Atspi, speech_start):
    keyboard.key(0xffc9)  # F12 (GNOME Terminal reserves F10 for its menu)
    table = wait("table label absent", lambda: named(root, "Project members"))
    assert table.get_role() == Atspi.Role.TABLE, table.get_role_name()
    ada = wait("table cell absent", lambda: named(root, "Ada member"))
    assert ada.get_role() == Atspi.Role.TABLE_CELL, ada.get_role_name()
    keyboard.key(0xff54)
    wait("first table cell did not receive focus", lambda: state(ada, Atspi.StateType.FOCUSED))
    wait("Orca did not announce the selected row", lambda: speech("Ada member"))
    locked = named(root, "Unavailable member")
    assert locked is not None and not state(locked, Atspi.StateType.ENABLED)
    actions = locked.get_action_iface()
    assert actions is None or actions.get_n_actions() == 0
    start = speech_start()
    keyboard.key(0xff54)
    wait("Orca did not announce the next enabled row", lambda: speech("Bea member", start))
    bea = named(root, "Bea member")
    wait("next table cell did not receive focus", lambda: state(bea, Atspi.StateType.FOCUSED))
    assert state(bea, Atspi.StateType.SELECTED)
    assert not state(ada, Atspi.StateType.SELECTED)
    start = speech_start()
    assert ada.get_component_iface().grab_focus()
    wait("assistive cell focus absent", lambda: state(ada, Atspi.StateType.FOCUSED))
    wait("Orca did not announce assistive cell focus", lambda: speech("Ada member", start))
    assert state(bea, Atspi.StateType.SELECTED), "focus changed selection"
    print("A11Y table: roles, labels, keyboard selection and disabled skipping announced", flush=True)

    keyboard.key(0xffc9)
    search = wait("DataTable search absent", lambda: named(root, "Search table"))
    assert search.get_role() == Atspi.Role.ENTRY
    start = speech_start()
    assert search.get_component_iface().grab_focus()
    wait("Orca did not announce search", lambda: speech("Search table", start))
    next_page = named(root, "Next")
    previous_page = named(root, "Prev")
    assert next_page is not None and next_page.get_role() == Atspi.Role.PUSH_BUTTON
    assert not state(previous_page, Atspi.StateType.ENABLED)
    click(next_page)
    cam = wait("next page data absent", lambda: named(root, "Cam customer"))
    assert named(root, "Ada customer") is None
    start = speech_start()
    assert cam.get_component_iface().grab_focus()
    wait("Orca did not announce paged row", lambda: speech("Cam customer", start))
    assert not state(next_page, Atspi.StateType.ENABLED)
    assert search.get_component_iface().grab_focus()
    keyboard.key(ord('B'))
    bea = wait("search result absent", lambda: named(root, "Bea customer"))
    wait("old page remained after search", lambda: named(root, "Cam customer") is None)
    start = speech_start()
    assert bea.get_component_iface().grab_focus()
    wait("Orca did not announce filtered row", lambda: speech("Bea customer", start))
    print("A11Y DataTable: search entry, paging actions, disabled controls, filtered and paged cell focus announced", flush=True)

    keyboard.key(0xffc9)
    tree = wait("Tree label absent", lambda: named(root, "Workspace tree"))
    assert tree.get_role() == Atspi.Role.TREE
    keyboard.key(0xff50)  # Home
    folder = wait("Tree node absent", lambda: named(root, "Documentation folder"))
    assert folder.get_role() == Atspi.Role.TREE_ITEM
    keyboard.key(0xff54)
    wait("Tree item keyboard focus absent", lambda: state(folder, Atspi.StateType.FOCUSED))
    start = speech_start()
    keyboard.key(0xff53)
    wait("Tree expansion absent", lambda: state(folder, Atspi.StateType.EXPANDED))
    wait("Orca did not announce tree expansion", lambda: speech("'expanded'", start))
    leaf = wait("Tree child absent", lambda: named(root, "User guide"))
    start = speech_start()
    assert leaf.get_component_iface().grab_focus()
    wait("Tree child assistive focus absent", lambda: state(leaf, Atspi.StateType.FOCUSED))
    wait("Orca did not announce tree child", lambda: speech("User guide", start))
    checkbox = named(root, "User guide checkbox")
    assert checkbox is not None and checkbox.get_role() == Atspi.Role.CHECK_BOX
    start = speech_start()
    keyboard.key(0x20)
    wait("Tree checked state absent", lambda: state(checkbox, Atspi.StateType.CHECKED))
    wait("Orca did not announce Tree checked state", lambda: speech("checked", start))
    # Orca suppresses repeated event types from one app within 100 ms.
    # Separate the two user actions without changing the reader or its settings.
    ready = time.monotonic() + 0.15
    wait("reader pacing elapsed", lambda: time.monotonic() >= ready)
    start = speech_start()
    click(checkbox)
    wait("Tree assistive uncheck failed", lambda: not state(checkbox, Atspi.StateType.CHECKED))
    wait("Orca did not announce Tree uncheck", lambda: speech("not checked", start))
    keyboard.key(0xff54)
    other = named(root, "Test folder")
    wait("Tree did not skip inert row", lambda: state(other, Atspi.StateType.FOCUSED))
    locked = named(root, "Unavailable folder")
    assert not state(locked, Atspi.StateType.ENABLED)
    action = locked.get_action_iface()
    assert action is None or action.get_n_actions() == 0
    wait("Orca did not announce skipped-to Tree row", lambda: speech("Test folder", start))
    ready = time.monotonic() + 0.15
    wait("reader focus pacing elapsed", lambda: time.monotonic() >= ready)
    start = speech_start()
    assert folder.get_component_iface().grab_focus()
    wait("Tree folder focus absent", lambda: state(folder, Atspi.StateType.FOCUSED))
    wait("Orca did not announce refocused Tree folder", lambda: speech("Documentation folder", start))
    start = speech_start()
    click(folder)
    wait("Tree assistive collapse failed", lambda: not state(folder, Atspi.StateType.EXPANDED))
    wait("Tree collapsed child remains", lambda: named(root, "User guide") is None)
    wait("Orca did not announce Tree collapse", lambda: speech("collapsed", start))
    print("A11Y Tree: hierarchy roles, labels, expansion, child focus and disabled skipping announced", flush=True)

    keyboard.key(0xffc9)
    files = wait("FileExplorer label absent", lambda: named(root, "Project files"))
    assert files.get_role() == Atspi.Role.LIST_BOX
    docs = wait("Directory absent", lambda: named(root, "Docs"))
    assert docs.get_role() == Atspi.Role.LIST_ITEM
    start = speech_start()
    assert docs.get_component_iface().grab_focus()
    wait("Directory focus absent", lambda: state(docs, Atspi.StateType.FOCUSED))
    wait("Orca did not announce directory", lambda: speech("Docs", start))
    # Navigation can focus and announce the first file before the tree query returns.
    start = speech_start()
    keyboard.key(0xff0d)
    guide = wait("Directory navigation failed", lambda: named(root, "guide.txt"))
    assert guide.get_component_iface().grab_focus()
    wait("File focus absent", lambda: state(guide, Atspi.StateType.FOCUSED))
    wait("Orca did not announce file", lambda: speech("guide.txt", start))
    keyboard.key(0xff08)  # Backspace
    wait("Parent navigation failed", lambda: named(root, "alpha.txt"))
    search = wait("Search files action label absent", lambda: named(root, "Search files"))
    click(search)
    entry = wait("Filter input absent", lambda: named(root, "Filter files"))
    assert entry.get_role() == Atspi.Role.ENTRY
    wait("Filter input focus absent", lambda: state(entry, Atspi.StateType.FOCUSED))
    keyboard.key(ord('a'))
    keyboard.key(0xff0d)
    wait("Filtered directory remains", lambda: named(root, "Docs") is None)
    alpha = named(root, "alpha.txt")
    assert alpha is not None
    print("A11Y FileExplorer: directory/file roles, focus speech, navigation and filter controls passed", flush=True)
