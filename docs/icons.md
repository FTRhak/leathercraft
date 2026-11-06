# Icons and icon buttons

How the desktop app draws icons and the square buttons that carry them. Everything here lives in the binary crate, in `src/icons/mod.rs`; the library crate never sees icons.

## Two ways to draw an icon

An icon is anything that paints into a rect with a stroke: `FnOnce(&egui::Painter, egui::Rect, egui::Stroke)`. There are two kinds:

| Kind | When to use | How |
|---|---|---|
| **Font glyph** | Fixed pictograms: trash can, padlock, tool, new piece, boolean operation, align and distribute icons | `paint_glyph(painter, rect, stroke, glyph::TRASH)` |
| **Painted** | Icons built from geometry or that depend on state: reorder arrows, spacing icons | Draw with `painter` lines and shapes inside `rect`, in `stroke` |

Prefer a glyph for a new pictogram. Paint an icon only when it depends on state or must follow the canvas geometry.

Both kinds must use only the `stroke` they are given for color and width. That is how they follow hover, selected and disabled states and the light or dark theme.

## The icon font

- **Font file:** `src/res/leathercrafticons.ttf`, embedded in the binary with `include_bytes!`.
- **Source:** `src/res/leathercraft-icons.fcp` is the editable font project. Edit the glyphs there, then export it over the `.ttf`. Commit both files together.
- **Family:** the font is registered as its own egui family, `leathercraft-icons`, not added to the text fonts, so glyphs never appear in ordinary labels and never replace a text character.
- **Code points:** glyphs sit in the Unicode private use area, starting at `U+E110`.

### Installation

`icons::install_icon_font` registers the font. `main.rs` runs it once in `EguiPrimaryContextPass`:

```rust
icons::install_icon_font.run_if(run_once),
```

A font added this way takes effect from the next egui pass, so glyphs drawn in the first frame may not show. Don't move this system to `Startup`; the egui context isn't ready there.

### Current glyphs

Constants are in `icons::glyph`:

| Constant | Code point | Used for |
|---|---|---|
| `TRASH` | `U+E110` | Delete piece, delete child (Layout panel) |
| `LOCK` | `U+E115` | Locked piece (Layout panel) |
| `UNLOCK` | `U+E116` | Unlocked piece (Layout panel) |
| `HISTORY` | `U+E121` | History panel tab title, via `icon_text` |
| `SELECT` | `U+E130` | Select tool (Tools panel) |
| `DRAW` | `U+E132` | Draw tool (Tools panel) |
| `RECTANGLE` | `U+E133` | Rectangle tool (Tools panel) |
| `ELLIPSE` | `U+E134` | Ellipse tool (Tools panel) |
| `EDGE` | `U+E135` | Edge tool (Tools panel) |
| `STITCH_HOLES` | `U+E136` | Stitch holes tool (Tools panel) |
| `CENTERLINE` | `U+E137` | Centerline tool (Tools panel) |
| `REGULAR_POLYGON` | `U+E138` | Regular polygon tool (Tools panel) |
| `NEW_PIECE` | `U+E13C` | New piece mode (Tools panel) |
| `SUBTRACT` | `U+E13D` | Subtract operation (Tools panel) |
| `UNION` | `U+E13E` | Union operation (Tools panel) |
| `INTERSECT` | `U+E13F` | Intersect operation (Tools panel) |
| `ALIGN_LEFT` | `U+E140` | Align left edges (Align panel) |
| `ALIGN_CENTER_X` | `U+E141` | Align horizontal centers (Align panel) |
| `ALIGN_RIGHT` | `U+E142` | Align right edges (Align panel) |
| `ALIGN_TOP` | `U+E143` | Align top edges (Align panel) |
| `ALIGN_CENTER_Y` | `U+E144` | Align vertical centers (Align panel) |
| `ALIGN_BOTTOM` | `U+E145` | Align bottom edges (Align panel) |
| `DISTRIBUTE_TOP` | `U+E146` | Distribute top edges (Align panel) |
| `DISTRIBUTE_CENTER_Y` | `U+E147` | Distribute vertical centers (Align panel) |
| `DISTRIBUTE_BOTTOM` | `U+E148` | Distribute bottom edges (Align panel) |
| `DISTRIBUTE_LEFT` | `U+E149` | Distribute left edges (Align panel) |
| `DISTRIBUTE_CENTER_X` | `U+E14A` | Distribute horizontal centers (Align panel) |
| `DISTRIBUTE_RIGHT` | `U+E14B` | Distribute right edges (Align panel) |

### Adding a glyph

1. Draw the glyph in `leathercraft-icons.fcp` at a free private use code point, and export the `.ttf`.
2. Add a `pub const NAME: char = '\u{E1xx}';` to `icons::glyph`, keeping the list ordered by code point.
3. Draw it with `paint_glyph`.
4. Add it to the table above.

Keep glyphs centered in the em square with even side bearings: `paint_glyph` centers the glyph on the rect and sizes it to 75% of the rect height, so an off-center glyph looks off-center in every button.

## `paint_glyph`

```rust
pub fn paint_glyph(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, glyph: char)
```

Paints `glyph` centered in `rect`, at a font size of `rect.height() * 0.75`, in `stroke.color`. Stroke width is ignored, since a glyph is filled. It has the same shape as a paint closure apart from the extra `glyph`, so it drops into `icon_button` directly or through a small wrapper:

```rust
fn paint_trash(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    paint_glyph(painter, rect, stroke, glyph::TRASH);
}
```

## `icon_button`

```rust
pub fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    selected: bool,
    name: &str,
    paint: impl FnOnce(&egui::Painter, egui::Rect, egui::Stroke),
) -> egui::Response
```

A square button of `size` × `size` points with no text.

| Argument | Meaning |
|---|---|
| `size` | Side of the square, in points. Use the calling panel's `ICON_BUTTON_SIZE` constant. |
| `selected` | Draws the button in the selected style. Use it for toggles and for the active choice of a group (current tool, locked piece). Pass `false` for plain actions. |
| `name` | The tooltip shown on hover and the name given to screen readers. Always a translated label from `i18n`, such as `language.label(Label::DeletePiece)`; never a literal string. |
| `paint` | Draws the icon. Gets the full button rect and the foreground stroke for the current state. |

It returns the `Response`; check `.clicked()` on it. The tooltip is already attached, so don't add another `on_hover_text`.

Behavior it provides, which callers should not reimplement:

- **Background and border** from `ui.style().interact_selectable`, so hover, press and selected states match egui's selectable widgets.
- **Icon color** from the same visuals' `fg_stroke`, passed to `paint`.
- **Accessibility:** reports itself as a button with its name and selected state.
- **Culling:** skips painting when the rect isn't visible.

### Disabled buttons

`icon_button` has no `enabled` argument. Wrap it in `ui.add_enabled_ui`, which dims the stroke and blocks clicks:

```rust
ui.add_enabled_ui(!piece.locked, |ui| {
    let name = language.label(Label::DeletePiece);
    if icon_button(ui, ICON_BUTTON_SIZE, false, name, paint_trash).clicked() {
        event = Some(ProjectEvent::RemovePiece { id: piece.id });
    }
});
```

### Toggles

Pass the current state as `selected`, choose the label for the action the click will take, and pick the glyph from the state:

```rust
let name = language.label(if piece.locked { Label::UnlockPiece } else { Label::LockPiece });
let response = icon_button(ui, ICON_BUTTON_SIZE, piece.locked, name, |painter, rect, stroke| {
    let icon = if piece.locked { glyph::LOCK } else { glyph::UNLOCK };
    paint_glyph(painter, rect, stroke, icon);
});
```

### A group of exclusive choices

One button per option, `selected` when it is the current one, as in the Tools panel:

```rust
for option in Tool::ALL {
    let name = language.label(option.label());
    let response = icon_button(ui, ICON_BUTTON_SIZE, *tool == option, name, |painter, rect, stroke| {
        option.paint_icon(painter, rect, stroke)
    });
    if response.clicked() {
        *tool = option;
    }
}
```

### Buttons that edit the project

The click handler follows the usual rule from `CLAUDE.md`: it returns or sends a `ProjectEvent`, never writes to the project. Interface-only state, such as the current tool, may be changed directly.

## Sizes and margins

Each panel defines its own constants rather than sharing them:

| Panel | `ICON_BUTTON_SIZE` | `ICON_MARGIN` | Context |
|---|---:|---:|---|
| Tools | 32 | 8 | Main tool palette |
| Layout | 20 | 4 | Inline in tree rows, matching text height |

`paint` receives the whole button rect. Painted icons shrink it themselves, typically `rect.shrink(ICON_MARGIN)` (Layout adds 1 more for its small triangles). Glyphs don't need a margin: the 75% font size leaves one.

Non-button icons in a row of icon buttons, such as the Layout panel's expand toggle or the blank space for a piece without children, should allocate the same `ICON_BUTTON_SIZE` square so the row stays aligned.

## Checklist for a new icon button

- [ ] Icon is a glyph from `icons::glyph`, or painted only with the given `stroke`.
- [ ] `name` is an `i18n` label, with arms in both catalogs.
- [ ] Size comes from the panel's `ICON_BUTTON_SIZE`.
- [ ] Disabled state uses `add_enabled_ui`; toggle state uses `selected`.
- [ ] Project changes go through a `ProjectEvent`.
- [ ] New glyphs are added to the font source, the `.ttf`, `icons::glyph` and the glyph table here.
