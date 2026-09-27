# Reactive TUI manual

This manual explains the public framework systems. Start with setup, then open
the chapters needed by your application.

The current Rust source, exported API, Cargo features, examples, and tests are
the authority for this manual. Specifications and decisions explain design
intent and platform limits when the source needs context.

## Table of contents

- [Getting started](#getting-started)
- [Supported API and verification limits](supported-api.md)
- [Widget catalog example](#widget-catalog-example)
- [Applications and components](#applications-and-components)
- [Elements, builders, and the virtual DOM](#elements-builders-and-the-virtual-dom)
- [Reactive state and hooks](#reactive-state-and-hooks)
- [Layout, style, and themes](#layout-style-and-themes)
- [Events, focus, and input](#events-focus-and-input)
- [Input widgets](#input-widgets)
- [Layout widgets](#layout-widgets)
- [Display widgets](#display-widgets)
- [Menus](#menus)
- [Dialogs](#dialogs)
- [Terminal widget](#terminal-widget)
- [Rendering and backends](#rendering-and-backends)
- [Animation and screens](#animation-and-screens)
- [Offscreen graphics](wgpu-graphics.md)
- [Terminal and embedded sessions](#terminal-and-embedded-sessions)
- [Text editing, Markdown, and syntax](#text-editing-markdown-and-syntax)
- [Accessibility](#accessibility)
- [Images and clipboard](#images-and-clipboard)
- [FFI and TypeScript](#ffi-and-typescript)

## Getting started

Add the crate, choose features, create a root component, select a backend, and
build an application. The chapter also explains the prelude and error type.

[Read Getting started](getting-started.md).

## Widget catalog example

Run `cargo run --locked --example widget_catalog` from the repository root.
The example presents focused live widget pages for screenshots and short video
clips. Use arrows or `1`–`8` to select a page, and Tab to focus its controls.
Navigation uses a sidebar at 80 columns or wider and a compact strip below
that width.
Pages use the terminal's full available width. Examples use one column below
80 columns and two above it, with one-cell gaps instead of stretched rows.
The Layout page includes a colored four-column grid with one-, two-, three-,
and four-column spans.
Use F1/F2 on the menus/dialogs page to select one live overlay demo at a time.
Use the mouse wheel to scroll longer pages.

Page `6` displays `manual/assets/logo.jpg` through the image widget. Page `7`
shows a viewport-sized rotating wireframe cube with Braille subpixel edges at
a bounded 80 ms frame interval. The default canvas is capped at 240x100 cells;
larger terminals retain empty space beyond that cap. No assets
are downloaded. Ctrl+Q is the global quit key; Ctrl+C and Escape quit when
the focused widget leaves the key unhandled.

With `--features wgpu-graphics`, Motion instead presents a shaded offscreen
GPU cube with labeled CPU fallback. See [Offscreen graphics](wgpu-graphics.md)
for commands, measurements, and verified-host limits.

The system page uses a bounded terminal command, not an interactive shell.
The known Kitty embedded-shell crash remains outside this example's scope.

## Applications and components

`App` owns the event loop, root component, scheduler, focus state, and backend.
Components produce element trees and receive state, events, layout, and
lifecycle notifications.

[Read Applications and components](app-and-components.md).

## Elements, builders, and the virtual DOM

Elements are the common UI value. Builders and macros create elements. The
virtual DOM provides keyed nodes, diffing, and patch application.

[Read Elements, builders, and the virtual DOM](elements-builders-and-vdom.md).

## Reactive state and hooks

Signals hold changing values. Effects observe changes. Schedulers, timers, and
the application waker move work back into the application loop.

[Read Reactive state and hooks](reactive-state-and-hooks.md).

## Layout, style, and themes

Taffy computes flex and grid layout in terminal cells. Style builders, utility
classes, colors, gradients, and themes control the painted result.

[Read Layout, style, and themes](layout-style-and-themes.md).

## Events, focus, and input

The event system represents keyboard, mouse, paste, resize, focus, and custom
events. Routing uses a node tree, hit targets, capture and bubble phases, and
focus management.

[Read Events, focus, and input](events-focus-and-input.md).

## Input widgets

Input widgets include text input, checkbox, radio button, select, and slider.
Each widget owns explicit props and state and supports builder construction.

[Read Input widgets](input-widgets.md).

## Layout widgets

Layout widgets provide accordion, breadcrumb, scroll view, stack, and tabs.
They add interaction and retained state to common layout patterns.

[Read Layout widgets](layout-widgets.md).

## Display widgets

Display widgets include charts, tables, trees, file exploration, progress,
modals, popovers, and images. This chapter covers the non-image display APIs.

[Read Display widgets](display-widgets.md).

## Menus

The menu system provides menu bars, context menus, popup menus, dialog menus,
items, shortcuts, actions, placement, and themes.

[Read Menus](menus.md).

## Dialogs

The dialog engine manages confirmation, input, autocomplete, progress, toast,
and wizard interactions. Results and lifecycle events are explicit values.

[Read Dialogs](dialogs.md).

## Terminal widget

`TerminalWidget` runs a child process through a pseudo-terminal and exposes its
screen as a component with input, resize, scrolling, focus, and error state.

[Read Terminal widget](terminal-widget.md).

## Rendering and backends

The rendering path converts elements into layout nodes, paints cell frames,
diffs them, writes terminal output, and reports display capabilities and
performance data.

[Read Rendering and backends](rendering-and-backends.md).

## Animation and screens

Animations provide easing, keyframes, springs, staggering, target bindings, and
timelines. Screens provide named content, transitions, hooks, and navigation.

[Read Animation and screens](animation-and-screens.md).

## Terminal and embedded sessions

Terminal modules parse ANSI output and maintain virtual screens. The optional
embedded session API runs an owned Unix PTY through libghostty when its feature
is enabled.

[Read Terminal and embedded sessions](terminal-and-embedded-sessions.md).

## Text editing, Markdown, and syntax

Text editors use cursor and gap-buffer primitives. Markdown becomes styled
terminal lines, and syntax resources highlight source text incrementally.

[Read Text editing, Markdown, and syntax](text-editing-markdown-and-syntax.md).

## Accessibility

Elements can carry AccessKit semantics. On Linux, the application can publish
an accessibility tree through an owned AT-SPI connection.

[Read Accessibility](accessibility.md).

## Images and clipboard

Image widgets decode, resize, animate, and choose terminal image protocols.
Clipboard hooks provide copy and paste state through platform commands.

[Read Images and clipboard](images-and-clipboard.md).

## FFI and TypeScript

The optional C ABI exposes application, rendering, terminal, reactive, editor,
dialog, Markdown, syntax, theme, and widget operations. TypeScript packages wrap
that ABI and also provide higher-level SDK types.

[Read FFI and TypeScript](ffi-and-typescript.md).
