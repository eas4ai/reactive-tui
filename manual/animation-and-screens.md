# Animation and screens

Crate modules: `animation`, `screen`

## Purpose

Animations change supported values over time. Screens group named content and
apply transitions when an application moves between views.

## Main API

- `Animation`, `AnimationBuilder`, `AnimationManager`, and
  `AnimationController` manage duration-based animation.
- Easing, typed keyframes, springs, staggering, timelines, bindings, and target
  contexts support different motion models.
- Animation hooks include `use_animation`, `use_transition`, `use_keyframes`,
  `use_spring`, and `use_stagger`.
- `ScreenId`, `Screen`, and `ScreenManager` identify and manage views.
- `TransitionType`, `TransitionConfig`, `TransitionState`, and easing presets
  control changes between screens.
- Screen hooks run on enter, exit, pause, and resume.

## Basic use

Create an animation or hook with start and end values, duration, and easing.
Attach it to a supported property or sample its current value during rendering.
For navigation, register screens with a manager and request a transition to a
screen ID.

## Behavior

The app advances active animations from its monotonic time source and schedules
the next required frame. Target bindings apply sampled values to presented
layout or paint properties. Keyed targets keep motion state across compatible
tree updates and release clocks when the target disappears.

A screen owns content and lifecycle hooks. The manager keeps navigation history
and transition state. Transition completion activates the destination and
releases temporary transition data.

## Limits

- Only supported property types can be interpolated.
- Incompatible CSS units preserve endpoints and report errors in checked paths.
- Percentage translations need the last presented target size.
- Reduced-motion mode applies static final values and releases clocks.
- Screen IDs must be stable and unique within a manager.

## Source map

- Animation exports: [`src/animation/mod.rs`](../src/animation/mod.rs)
- Public animation API: [`src/animation/api.rs`](../src/animation/api.rs)
- Screen exports: [`src/screen/mod.rs`](../src/screen/mod.rs)
- Screen manager: [`src/screen/manager.rs`](../src/screen/manager.rs)
- Animation API tests: [`tests/test_animation_api.rs`](../tests/test_animation_api.rs)
- Screen behavior tests: [`tests/screen_system_tests.rs`](../tests/screen_system_tests.rs)

## Related chapters

- [Reactive state and hooks](reactive-state-and-hooks.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [Applications and components](app-and-components.md)

[Back to the manual](README.md)
