# Colors and components

How the desktop app picks its colors, and what a new UI component must do to follow the dark and light theme. Everything here lives in the binary crate; the library crate has no colors.

The visual source of truth is the Brilliant design: https://brilliant.design/neat-pansy/leathercraft-design/main

## The rule

**Every color the app paints comes from `Palette`** (`src/ui_components/palette.rs`). Components, panels and canvas code never write a color literal (`Color32::from_rgb`, `from_gray`, `from_rgba_unmultiplied`, `hex(0x…)`) themselves.

A new color is first added to `Palette`, with a value for both themes, and then read from there. This keeps the two themes complete and lets one file be compared against the design.

Allowed outside `palette.rs`:

| Allowed | Why |
|---|---|
| `Color32::TRANSPARENT` | No color, not a theme choice |
| A palette color changed with `gamma_multiply` or `linear_multiply` | Faded variants of a color the palette already has, such as an unselected hole |
| Colors that are document data, such as a material's color swatch | They come from `Project`, not from the theme |
| Colors egui draws for its own widgets (`egui::Visuals`) | Set by the theme, not by our code |

## Reading the palette

`Palette::of` picks `DARK` or `LIGHT` from `visuals.dark_mode` and returns a `&'static Palette`:

```rust
let palette = Palette::of(painter);
painter.line_segment(points, egui::Stroke::new(1.5, palette.outline));
```

Inside a widget, take the painter from the `Ui`: `Palette::of(ui.painter())`. Call `Palette::of` once per paint, not once per color, and never cache it across frames: the theme can change in the Settings window at any time.

## Adding a color

1. **Name it by role, not by hue:** `selection`, `snap_marker`, `error`, not `orange` or `pink`. The same role may need a different hue in the other theme.
2. **Add the field** to `Palette`, with a one-line doc comment when the name doesn't say where it is used.
3. **Give it a value in both `DARK` and `LIGHT`.** The struct is a `const`, so a missing value is a compile error. Take the values from the design; if the design has none yet, pick one and add it to the design too.
4. **Check contrast** against the background of that theme: lines and markers must stay visible on the canvas, text must stay readable (aim for 4.5:1 for text, 3:1 for strokes).
5. **Use it** through `Palette::of`. Remove the literal it replaces.

Keep the fields grouped by area, canvas first, then each component.

## Component colors

A component with several kinds or states (default, hover, pressed, disabled, focused) keeps its colors in the palette as a nested struct, one per component, defined in `palette.rs`:

```rust
pub(crate) struct Palette {
    // canvas
    pub(crate) outline: egui::Color32,
    // ...
    pub(crate) button: ButtonColors,
}

pub(crate) struct ButtonColors {
    pub(crate) primary: StateColors,
    pub(crate) secondary: StateColors,
    pub(crate) outline: StateColors,
    pub(crate) focus: egui::Color32,
}
```

The component module keeps only the logic: which kind and state it is in, and so which palette entry to paint with. It owns sizes, padding and corner radius, but no colors.

## Writing a component

A reusable component goes in its own module under `src/ui_components/` and:

- implements `egui::Widget`, so it is added with `ui.add(...)`;
- takes the disabled state the egui way, from `ui.add_enabled(false, ...)` (`ui.is_enabled()`), not from its own flag;
- reads every color from `Palette::of(ui.painter())`, including hover, disabled and focus colors;
- draws icons only with the stroke it is given (see `docs/icons.md`);
- takes its text as `impl Into<egui::WidgetText>`, so callers pass translated `i18n` labels;
- is checked by hand in both themes before it is merged: switch the theme in Settings and look at each state.

## Not yet migrated

These colors predate the rule and still live outside the palette. Move them in when the file is next changed:

| Where | Colors | Target in `Palette` |
|---|---|---|
| `ui_components/button/style.rs` | `ButtonStyle::of` tables and `focus_color`, dark and light | `button: ButtonColors` |
| `editor/canvas.rs` | `SNAP_COLOR`, `SELECTED_COLOR` | `snap_marker`, `selection` |
| `editor/canvas.rs` `op_color`, `draft_color` | Subtract, union, intersect, and the draft color with no operation | `op_subtract`, `op_union`, `op_intersect`, `draft` |
| `editor/canvas.rs` piece drawing | Error outline, piece fill | `error`, `piece_fill` |

These have a single value for both themes today; give each a light-theme value when moving it.
