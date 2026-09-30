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
Status: Draft

[THM-002] A role that neither the active theme nor a theme it extends defines MUST still resolve to a color: a text role to black or white, whichever contrasts more with its fill; `selection` and `ring` to the theme's `primary`; `input` to its `surface`; `hover` to seven parts of its `surface` mixed with one part of its `foreground`; and every other role to the built-in light preset's color when black contrasts with the theme's `background` more than white does, and to the dark preset's color otherwise. A class that names a role MUST never leave its element without the color it asks for.
Falsifier: Under a theme that defines only `background`, `foreground` and `primary`, the class `bg-R` or `text-R` for a role R of THM-001 leaves the color of its style unset; a text role resolves to a color that contrasts with its fill by less than 4.5 to 1; `selection` or `ring` resolves to a color other than that theme's `primary`; `input` resolves to a color other than `surface` does; `hover` resolves to a color other than seven parts of what `surface` resolves to and one part of that theme's `foreground`; or `surface` resolves to a color other than the dark preset's when the theme's `background` is black and other than the light preset's when it is white.
Mechanism: theme
Rationale: An application's theme written before these roles existed names none of the new ones, and every reworked widget depends on them.
Status: Draft

[THM-003] After the application sets another theme, the next frame presented MUST paint every element whose classes name a role, and every widget a commitment has brought to the widget bar, in the new theme's colors.
Falsifier: After `Theme::set_active` or `App::set_theme` with a theme whose roles all differ from the old theme's, the next presented frame holds a cell of such an element or widget in a color of the old theme.
Mechanism: theme
Status: Draft
