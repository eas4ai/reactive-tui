from pathlib import Path
from types import SimpleNamespace

source = Path("tests/api_widget_behavior/orca_menus.py").read_text()
cut = source.index('        group = named(root, "Recent menu")', source.index('wait("menu checked state absent"'))
source = source[:cut] + "        return True\n"
source = source.replace("range(1, 5)", "[1]")
namespace = {}
exec(compile(source, "orca_menus_stage_one", "exec"), namespace)

class Node:
    def __init__(self, world, name):
        self.world, self.name = world, name
    def get_role(self):
        return "menu" if self.name == "file" else "checkbox"
    def get_component_iface(self):
        return self
    def get_action_iface(self):
        return None
    def grab_focus(self):
        self.world.focus = self
        return True

class World:
    def __init__(self):
        self.file = Node(self, "file")
        self.checkbox = Node(self, "checkbox")
        self.group = Node(self, "group")
        self.disabled = Node(self, "disabled")
        self.focus = self.file
        self.open = False
        self.pending_click = False
        self.checked = False
        self.old_checkbox = None
    def key(self, value):
        if value == 0xff54:
            if not self.open:
                self.open = True
                self.checkbox = Node(self, "checkbox")
            else:
                self.focus = self.group
    def deliver_click(self):
        self.pending_click = False
        self.checked = True
        self.open = False
        self.old_checkbox = self.checkbox
        self.focus = self.file
    def click(self, node):
        assert node is self.checkbox
        self.pending_click = True
    def named(self, root, label):
        if label.startswith("Menu fixture"):
            return True
        if label == "Menu file":
            return self.file
        if label == "Enable notifications":
            return self.checkbox if self.open else None
        if label == "Recent menu":
            return self.group
        if label == "Unavailable item":
            return self.disabled
        raise AssertionError(label)
    def state(self, node, flag):
        if flag == "focused":
            return self.focus is node
        if flag == "enabled":
            return node is not self.disabled
        if flag == "checked":
            if self.pending_click:
                self.deliver_click()
            return self.open and node is self.checkbox and node is not self.old_checkbox and self.checked
        raise AssertionError(flag)
    def wait(self, reason, predicate):
        for _ in range(3):
            result = predicate()
            if result:
                return result
            if self.pending_click:
                self.deliver_click()
        raise AssertionError(reason)

world = World()
atspi = SimpleNamespace(Role=SimpleNamespace(MENU_ITEM="menu", CHECK_MENU_ITEM="checkbox"),
                        StateType=SimpleNamespace(FOCUSED="focused", ENABLED="enabled", CHECKED="checked"))
namespace["exercise"](None, world, world.wait, world.named, world.state, world.click,
                      lambda *args: True, atspi, lambda: 0)
assert world.checked and world.open
print("PASS queued click closes before reopen; checked state belongs to the new menu item")
