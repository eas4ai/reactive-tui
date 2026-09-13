"""Real reader actions and announcements for each retained menu family."""


def exercise(root, keyboard, wait, named, state, click, speech, Atspi, speech_start):
    for stage in range(1, 5):
        keyboard.key(0xffc1)  # F4; replace the previous controls with a fresh menu.
        wait("menu fixture did not switch", lambda: named(root, f"Menu fixture stage {stage}"))
        if stage == 1:
            file = wait("menubar item absent", lambda: named(root, "Menu file"))
            assert file.get_role() == Atspi.Role.MENU_ITEM
            assert file.get_component_iface().grab_focus(), "menu assistive focus unavailable"
            wait("menubar item focus absent", lambda: state(file, Atspi.StateType.FOCUSED))
            keyboard.key(0xff54)  # Down opens the dropdown.
        checkbox = wait("menu checkbox absent", lambda: named(root, "Enable notifications"))
        assert checkbox.get_role() == Atspi.Role.CHECK_MENU_ITEM
        disabled = named(root, "Unavailable item")
        assert disabled is not None and not state(disabled, Atspi.StateType.ENABLED)
        action = disabled.get_action_iface()
        assert action is None or action.get_n_actions() == 0
        assert checkbox.get_component_iface().grab_focus(), "menu checkbox assistive focus unavailable"
        wait("menu checkbox focus absent", lambda: state(checkbox, Atspi.StateType.FOCUSED))
        # Keyboard opening can focus and announce the checkbox before this probe
        # records its log position. Require Orca to present the intermediate focus
        # before returning, then require a new checkbox announcement.
        group = named(root, "Recent menu")
        group_start = speech_start()
        assert group is not None and group.get_component_iface().grab_focus()
        wait("submenu assistive focus absent", lambda: state(group, Atspi.StateType.FOCUSED))
        wait("Orca did not announce intermediate menu focus",
             lambda: speech("Recent menu", group_start))
        start = speech_start()
        assert checkbox.get_component_iface().grab_focus()
        wait("menu checkbox focus did not return", lambda: state(checkbox, Atspi.StateType.FOCUSED))
        wait("Orca did not announce menu checkbox", lambda: speech("Enable notifications", start))
        assert not state(checkbox, Atspi.StateType.CHECKED)
        if stage != 3:
            start = speech_start()
            click(checkbox)
            if stage == 1:
                # The assistive click is queued. Wait for its close and focus
                # restoration before asking the menubar to reopen.
                wait("menubar checkbox activation did not close the menu",
                     lambda: named(root, "Enable notifications") is None)
                wait("menubar focus was not restored",
                     lambda: state(file, Atspi.StateType.FOCUSED))
                keyboard.key(0xff54)
                checkbox = wait("menubar did not reopen", lambda: named(root, "Enable notifications"))
            wait("menu checked state absent", lambda: state(checkbox, Atspi.StateType.CHECKED))
            wait("Orca did not announce checked menu state", lambda: speech("check menu item checked", start) or speech("'checked'", start))
        group = named(root, "Recent menu")
        assert group is not None and group.get_component_iface().grab_focus()
        wait("submenu focus absent", lambda: state(group, Atspi.StateType.FOCUSED))
        start = speech_start()
        keyboard.key(0xff53)
        leaf = wait("nested menu entry absent", lambda: named(root, "Recent document"))
        wait("nested menu focus absent", lambda: state(leaf, Atspi.StateType.FOCUSED))
        wait("Orca did not announce nested menu entry", lambda: speech("Recent document", start))
        assert state(group, Atspi.StateType.EXPANDED)
        click(leaf)
        if stage == 2:
            keyboard.key(0xff1b)
            wait("popup submenu remained after Escape", lambda: named(root, "Recent document") is None)
            keyboard.key(0xff1b)
        elif stage == 4:
            confirm = named(root, "Confirm")
            assert confirm is not None
            click(confirm)
        wait("closed menu remains accessible", lambda: named(root, "Enable notifications") is None)
        print(f"A11Y menu family {stage}: roles, disabled actions, assistive/keyboard focus, nested activation and removal passed", flush=True)
    keyboard.key(0xffc2)  # F5 returns to the existing CSS fixture.
    wait("menu fixture remained mounted", lambda: named(root, "Menu fixture stage 4") is None)
