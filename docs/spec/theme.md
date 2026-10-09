# Theme

Prefix: THM

A theme is a set of named color variables (src/theme/mod.rs). A utility
class names one as `bg-primary` or `text-muted`, and `Theme::resolve_color`
turns the name into a color of the active theme. Read on 2026-09-29: the
five presets (dark, light, high contrast, Solarized Dark, Gruvbox Dark)
define `primary`, `secondary`, `accent`, `background`, `surface`,
`foreground`, `text-muted`, `border`, `success`, `warning`, `error`, `info`
and the chart colors (src/theme/presets.rs). No preset names the text that
is drawn on a fill, the current row of a list, the row under the pointer, a
field, the focus border, the veil behind a modal or a shadow. A class that
names a variable the theme lacks is dropped without a message. No widget
under src/widgets names a theme variable in a class: the menus, dialogs,
modal, popover and toasts write palette classes such as `bg-gray-800` and
`text-white`, so they look the same under every theme. Seen in the widget
catalog the same day: the menus are white boxes on the dark page, and five
dialogs draw light text on a light grey box. The chart colors are in
charts.md (CHT-017).

## Observed

(none yet)

## Draft

[THM-001] Every built-in preset MUST define every color role: `background`, `surface`, `foreground`, `text-muted`, `border`, `input`, `ring`, `hover`, `overlay` and `shadow`; the fills `primary`, `secondary`, `accent`, `success`, `warning`, `error`, `info` and `selection`; and for each fill the text role drawn on it, named by the fill with `-foreground` added. In every preset each text role MUST contrast with its fill by at least 4.5 to 1, as MUST `foreground` with `background`, `surface`, `input` and `hover`, and `text-muted` with `background` and `surface`; and `ring` MUST contrast with `background` and with `surface` by at least 3 to 1.
Falsifier: A built-in preset lacks one of the roles; or, by the contrast ratio of WCAG 2.1, a text role contrasts with its fill by less than 4.5 to 1, `foreground` with `background`, `surface`, `input` or `hover` by less than 4.5 to 1, `text-muted` with `background` or `surface` by less than 4.5 to 1, or `ring` with `background` or `surface` by less than 3 to 1.
Mechanism: theme
Rationale: Black or white reaches 4.5 to 1 on every fill, so no preset has to change a fill it has today.
Status: Agreed 2026-09-29

[THM-002] A role that neither the active theme nor a theme it extends defines MUST still resolve to a color: a text role to black or white, whichever contrasts more with its fill; `selection` and `ring` to the theme's `primary`; `input` to its `surface`; `hover` to seven parts of its `surface` mixed with one part of its `foreground`; and every other role to the built-in light preset's color when black contrasts with the theme's `background` more than white does, and to the dark preset's color otherwise. A class that names a role MUST never leave its element without the color it asks for.
Falsifier: Under a theme that defines only `background`, `foreground` and `primary`, the class `bg-R` or `text-R` for a role R of THM-001 leaves the color of its style unset; a text role resolves to a color that contrasts with its fill by less than 4.5 to 1; `selection` or `ring` resolves to a color other than that theme's `primary`; `input` resolves to a color other than `surface` does; `hover` resolves to a color other than seven parts of what `surface` resolves to and one part of that theme's `foreground`; or `surface` resolves to a color other than the dark preset's when the theme's `background` is black and other than the light preset's when it is white.
Mechanism: theme
Rationale: An application's theme written before these roles existed names none of the new ones, and every reworked widget depends on them.
Status: Agreed 2026-09-29

[THM-003] After the application sets another theme, the next frame presented MUST paint every element whose classes name a role, and every widget a commitment has brought to the widget bar, in the new theme's colors.
Falsifier: After `Theme::set_active` or `App::set_theme` with a theme whose roles all differ from the old theme's, the next presented frame holds a cell of such an element or widget in a color of the old theme.
Mechanism: theme
Status: Agreed 2026-09-29

[THM-004] Resolving a color MUST end for every theme: when a variable's value names another variable and following the names comes back to one already followed, or passes 32 names, the variable MUST resolve as one no theme defines, so a role takes the color THM-002 gives it and any other name resolves to nothing, through this theme and every theme it extends.
Falsifier: With a theme whose variables say `--color-a: b` and `--color-b: a`, or `--color-input: hover` and `--color-hover: input`, `Theme::resolve_color` of `a` or of `input` overflows the stack or does not return within a second, or returns a color for `a`, or for `input` another color than its `surface`.
Mechanism: review-high
Rationale: Two variables naming each other made resolution recurse until the process ran out of stack (the developer's code review of 2026-10-04, W02).
Status: Agreed 2026-10-04

[THM-005] `ansi256_to_rgb` MUST decode the 216-color cube (indices 16 to 231) with xterm's six levels per component, 0, 95, 135, 175, 215 and 255, and `rgb_to_ansi256` MUST encode each component to the nearest of those levels; the sixteen base colors and the grey ramp are unaffected.
Falsifier: Index 17 decodes to other than (0, 0, 95) or index 231 to other than (255, 255, 255); (60, 0, 0) encodes to an index whose decoded red is not 95; or a decoded cube color encodes to a different index than it came from.
Mechanism: review-widgets
Rationale: The developer's code review of 2026-10-04, W05: the cube was decoded with multiples of 51 and encoded with a 48 threshold, so a child terminal's or a captured image's indexed colors lost their shades.
Status: Agreed 2026-10-08

[THM-006] A theme MUST be loadable from a JSON document through `Theme::from_json(&str)` and `Theme::from_file(path)`: an object with `name`, a string; `extends`, optional, the name of a built-in preset (`dark`, `light`, `high-contrast`, `solarized-dark` or `gruvbox-dark`); and `variables`, an object whose keys are variable names such as `--color-primary` and whose values are the strings `ThemeVariables::set` takes. The loaded theme MUST resolve every role as THM-002 and THM-004 say and MUST take effect through `Theme::set_active` and `App::set_theme`. A document that is not such an object, names an unknown preset, has a key outside `name`, `extends` and `variables`, or holds a value that is not a string MUST be refused with an error that names the offending key or the preset. Every built-in preset MUST round-trip through `Theme::to_json` and `Theme::from_json` with equal name and variables.
Falsifier: `Theme::from_json` of `{"name":"ocean","extends":"dark","variables":{"--color-primary":"#0077aa"}}` fails, or the result's `primary` resolves to other than #0077aa or its `surface` to other than the dark preset's; a document with `"extends":"ocean"`, with a key `colors`, or with `"--color-primary": 7` is accepted, or its error names neither the key nor the preset; or a built-in preset's `to_json` does not load back with equal name and variables.
Mechanism: widget-behavior
Rationale: A theme can be built only in code (src/theme/mod.rs:49-52, src/theme/presets.rs); gpui-kit loads themes from files (docs/widget-study.md, change 4), and an application's users cannot change its colors without a rebuild.
Status: Agreed 2026-10-09
