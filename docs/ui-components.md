# UI components

How to use the design-system widgets in `src/ui_components/`. This page covers using them in panels and dialogs. For writing a new one, see `docs/design.md`. For icons, see `docs/icons.md`.

Every component lives in the binary crate, is drawn in the colors of the current theme (`Palette`), and follows the Brilliant design: https://brilliant.design/neat-pansy/leathercraft-design/main

## Common rules

These hold for every component below unless its section says otherwise.

| Rule | How |
|---|---|
| **Adding** | Each widget implements `egui::Widget`: `ui.add(Widget::new(..))`. Builders are `#[must_use]`, so a widget you forget to add is a warning. |
| **Text** | Takes `impl Into<egui::WidgetText>`. Pass a translated label, `language.label(Label::..)`, never an English literal. |
| **Bound value** | Stateful widgets take `&mut` to the value and write to it themselves. Check `response.changed()` to react to the change. |
| **Disabled** | Use egui's own way: `ui.add_enabled(false, ..)` for one widget, `ui.add_enabled_ui(false, |ui| ..)` for a group. No widget has its own `enabled` flag. |
| **Colors** | Never pass a color. A component takes every color from `Palette::of(ui.painter())`, so it follows the theme as soon as Settings change it. |
| **Focus** | Every widget can take keyboard focus and draws a focus ring from the palette. |
| **Project edits** | Widgets only change local values: a draft, `EditorState`, a copy. A change to the document is still sent as a `ProjectEvent` (see `CLAUDE.md`). |

The `*_value` helpers (`radio_value`, `dropdown_value`, `tab_value`) work like `ui.selectable_value`. Each one draws an option that is selected while `*current == value`, sets `current` on click, and marks the response as changed. Prefer them to wiring `clicked()` by hand.

## Which component to use

| Need | Component |
|---|---|
| A dialog action: OK, Cancel, Reset | `Button` |
| An on/off setting | `Checkbox` |
| One of 2 to 4 short options, shown side by side | `GroupButton` |
| One of several options, as a vertical list | `Radio` / `radio_value` |
| One of many options, or options with swatches or groups | `Dropdown` |
| Pages of a dialog | `tab_bar` + `Tab` / `tab_value` |
| Free text: a name | `TextInput` |
| A number with a unit and range | `NumberInput` |
| Filtering a list | `SearchInput` |
| Rows to pick from, with an icon and a trailing status, such as the History panel | `list` + `ListItem` |
| A square tool or toolbar button with a pictogram | `icons::icon_button` |
| Panel arrangement (docking, tabs, floating) | `dock::PanelDock` (a resource, not a widget) |

---

## Button

`button::Button`: a text button in one of three kinds.

```rust
use crate::ui_components::button::Button;

if ui.add_enabled(can_save, Button::primary(language.label(Label::Ok))).clicked() { .. }
if ui.add(Button::secondary(language.label(Label::Cancel))).clicked() { .. }
if ui.add(Button::outline(language.label(Label::ResetToDefaults))).clicked() { .. }
```

| Constructor | Kind | Use for |
|---|---|---|
| `Button::primary(text)` | `ButtonKind::Primary` | The main action of a dialog. **At most one per dialog.** |
| `Button::secondary(text)` | `ButtonKind::Secondary` (default) | The action beside the main one, such as Cancel |
| `Button::outline(text)` | `ButtonKind::Outline` | Helper actions, such as Reset to defaults |
| `Button::new(kind, text)` | any | When the kind is chosen at run time |

- **States:** default, hover (also while pressed), disabled, plus a focus ring.
- **Size:** fits the text with 16 × 6 px padding, at least 64 px wide. The text doesn't wrap.
- **Dialog footer:** lay the buttons out right to left, so OK ends at the right edge with Cancel before it:
  ```rust
  ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
      ui.add(Button::primary(..));
      ui.add(Button::secondary(..));
  });
  ```
- There is no icon or toggle variant. Use `icon_button` for those.

## Checkbox

`checkbox::Checkbox`: a box bound to a `bool`, with an optional label to its right.

```rust
use crate::ui_components::checkbox::Checkbox;

if ui.add(Checkbox::new(&mut snap.grid, language.label(Label::SnapGrid))).changed() { .. }
```

- **Value:** a click toggles the `bool` and marks the response as changed. Clicking the label toggles it too.
- **No label:** pass `""`. The widget then takes just the 15 px box.
- **Mixed:** `.mixed(true)` shows a dash for a setting that differs across a selection, and a click then checks it. It has no caller yet (reserved for multi-piece editing).
- **States:** checked, unchecked or mixed, each one default, hover or disabled, plus a focus ring.

## Radio

`radio::Radio` and `radio::radio_value`: a round option with an optional label. It has the checkbox's size and colors (`Palette::checkbox`).

```rust
use crate::ui_components::radio::radio_value;

for theme in Theme::ALL {
    radio_value(ui, &mut theme_value, theme, language.label(theme.label()));
}
```

- `Radio::new(selected, text)` only draws the option. It changes nothing itself, so check `clicked()`. Prefer `radio_value`.
- Lay the options out in a vertical list. For 2 to 4 short options in a row, use `GroupButton` instead.
- The module has no caller at the moment (`#[allow(dead_code)]` in `mod.rs`). Remove that attribute once it is used.

## GroupButton

`group_button::GroupButton`: a row of joined segments, one per option, with exactly one selected.

```rust
use crate::ui_components::group_button::GroupButton;

ui.add(
    GroupButton::new(&mut draft.preferences.theme)
        .option(Theme::Dark, language.label(Label::ThemeDark))
        .option(Theme::Light, language.label(Label::ThemeLight)),
);
```

To build it from a list of values, fold over the list:

```rust
let group = Theme::ALL.into_iter().fold(
    GroupButton::new(&mut draft.preferences.theme),
    |group, theme| group.option(theme, language.label(theme.label())),
);
ui.add(group);
```

- **Value:** `V: PartialEq`. Segments appear in the order of the `.option` calls. A click on another segment sets the value and marks the response as changed. A click on the selected segment does nothing.
- **Selection:** if `current` matches no option, no segment is selected. Make sure the options cover every value.
- **Size:** each segment is its text plus 12 px padding on each side, at least 44 px wide. The group is 26 px tall. Keep labels short; the group doesn't wrap or shrink.
- **Disabled:** the whole group only. `ui.add_enabled(false, GroupButton::new(..))` still shows the selected segment.
- **Focus:** each segment takes focus on its own (Tab moves between them).

## Dropdown

`dropdown::{Dropdown, DropdownItem, dropdown_value, group_label}`: a field showing the chosen value that opens a list below it.

Unlike the other components, `Dropdown` is **not** an `egui::Widget`. It is shown with `show_ui`, the way `egui::ComboBox` is.

```rust
use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value, group_label};

Dropdown::new("units", language.label(units.label()))
    .width(200.0)
    .max_height(300.0)
    .show_ui(ui, |ui| {
        for option in Units::ALL {
            let item = DropdownItem::new(language.label(option.label()));
            dropdown_value(ui, &mut units, option, item);
        }
    });
```

### `Dropdown`

| Method | Meaning |
|---|---|
| `new(id_salt, selected_text)` | `id_salt` keeps the list open across frames and **must be unique within the parent `Ui`**. `selected_text` is what the closed field shows. You keep it in sync with the value. |
| `.width(w)` | Width of the field and of its list (default `spacing().combo_width`). Text that doesn't fit is cut with an ellipsis. |
| `.max_height(h)` | How tall the list grows before it scrolls (default 360 px) |
| `.leading(size, paint)` | Paints something of `size` before the text, such as the swatch of the chosen color |
| `.show_ui(ui, add_items)` | Draws the field and, while it is open, the list. It returns `InnerResponse<Option<R>>`, whose `inner` is `None` while the list is closed. |

- A click on an item closes the list.
- **Disabled:** wrap the dropdown in `ui.add_enabled_ui(false, |ui| ..)`. `add_enabled` doesn't apply, since it isn't a `Widget`.
- **Hover hint:** put one on the field with `.show_ui(..).response.on_hover_text(..)`.

### Items

- `DropdownItem::new(text)`, with `.selected(bool)` and `.leading(size, paint)`. Add it with `ui.add(item)`, or better with `dropdown_value(ui, &mut value, option, item)`, which sets `selected` for you.
- Disable a single item with `ui.add_enabled(false, DropdownItem::new(..))`.
- `dropdown_value` returns the item's `Response`, so a per-item hover hint is `dropdown_value(..).on_hover_text(..)`.
- `group_label(ui, text)` draws a small heading above a group of items. It adds spacing before every group but the first.

### Leading slot

The `paint` closure gets the painter and the rect of `size`. A swatch shows document data, not a theme color, so painting its own color there is allowed:

```rust
Dropdown::new(field, language.label(Label::Color(current)))
    .leading(SWATCH, move |painter, rect| swatch(painter, rect, current))
    .show_ui(ui, |ui| {
        for group in ColorGroup::ALL {
            group_label(ui, language.label(Label::ColorGroup(group)));
            for option in LeatherColor::ALL.into_iter().filter(|c| c.group() == group) {
                let item = DropdownItem::new(language.label(Label::Color(option)))
                    .leading(SWATCH, move |painter, rect| swatch(painter, rect, option));
                dropdown_value(ui, value, option, item);
            }
        }
    });
```

Anything else in a leading slot, such as an icon, takes its color from the palette.

## Tabs

`tabs::{tab_bar, Tab, tab_value}`: a strip of underlined tabs for the pages of a dialog.

```rust
use crate::ui_components::tabs::{Tab, tab_bar, tab_value};

tab_bar(ui, |ui| {
    tab_value(ui, &mut draft.tab, SettingsTab::Material, Tab::new(language.label(Label::DefaultMaterial)));
    tab_value(ui, &mut draft.tab, SettingsTab::Hotkeys, Tab::new(language.label(Label::Hotkeys)).icon(glyph::LOCK));
});
match draft.tab { .. } // draw the open page below
```

- `tab_bar` takes the full available width and is 34 px tall. It lays the tabs out left to right.
- `Tab::new(text)`, with `.icon(glyph)` (a glyph from `icons::glyph`, drawn before the label) and `.selected(bool)`. Prefer `tab_value`.
- **States:** active (underlined), inactive, hover, disabled, plus focus.
- Disable one tab with `ui.add_enabled(false, Tab::new(..))`, or the whole bar with `ui.add_enabled_ui`.
- These tabs are for pages inside a dialog. Docked panel tabs are drawn by `egui_dock` through `PanelDock`.

## Inputs

`input::{TextInput, NumberInput, SearchInput}`: themed text fields.

They share these rules:

- **Value:** each one edits a `&mut` value in place. Check `response.changed()`.
- **Width:** `.width(w)`. The default is `spacing().text_edit_width`, or 96 px for numbers.
- **Esc** leaves a focused field without closing the dialog around it.
- **`.id_salt(..)`:** keeps the cursor, the focus and the text being typed when fields earlier in the same `Ui` appear or disappear, such as rows of a list that changes. Set it on any field whose position can shift.
- A click on the padding, the unit or the icon puts the cursor in the text.
- **States:** default, hover, focus, disabled; for `NumberInput` also accent.

### `TextInput`

```rust
ui.add(TextInput::new(&mut name).hint(language.label(Label::PieceName)));
```

`.hint(text)` is shown while the field is empty. The Properties panel uses it for the piece name.

### `NumberInput`

```rust
ui.add(
    NumberInput::new(&mut material.thickness_mm)
        .range(MIN_THICKNESS_MM..=MAX_THICKNESS_MM)
        .step(0.1)
        .decimals(2)
        .unit(language.unit_symbol(Units::Millimeters)),
);
```

| Method | Default | Meaning |
|---|---|---|
| `new(&mut n)` | — | Any `emath::Numeric`: `f32`, `f64`, the integer types |
| `.range(a..=b)` | the type's full range | Typed and stepped values are clamped into it |
| `.step(s)` | `1.0` | Applied by the stepper and the Up and Down arrow keys; Shift steps 10 times as much |
| `.decimals(d)` | 2, or 0 for integers | How many decimals are shown, and how the value is rounded |
| `.unit(text)` | none | Shown after the value, such as "mm" |
| `.accent(true)` | off | Draws the value in the accent color, for a value that matches a canvas layer, such as the selected edge's length in Properties. |

- **Committing:** a typed value is committed on Enter or when the field loses focus. Esc drops it. Focusing the field selects its text, so typing replaces the value. Text that doesn't parse as a number is dropped.
- **`changed()` caveat:** the bound value only changes on commit, a step or an arrow key. But the response also includes the inner text edit, so `changed()` is true on each keystroke too. Before you send an event, compare the value with its previous value, or react on `lost_focus()`.
- **Units:** the bound value is in internal units. Internal geometry is always mm. If the field shows `Project.units`, convert both ways with `Units` and label it with the matching unit symbol. Don't store the converted number.

### `SearchInput`

```rust
ui.add(
    SearchInput::new(&mut draft.hotkey_filter)
        .hint(language.label(Label::SearchHotkeys)),
);
```

It has a search icon, and a clear button while it has text. `.hint`, `.width` and `.id_salt` work as in `TextInput`. Filtering is up to the caller: read the string each frame.

## List

`list::ListItem`: one full-width row with an optional icon, a label truncated to fit, and an optional trailing text. `list(ui, |ui| ..)` lays the rows out with the design's spacing.

```rust
use crate::ui_components::list::{ListItem, list};

list(ui, |ui| {
    for (index, entry) in entries.iter().enumerate() {
        let item = ListItem::new(language.message(&entry.action))
            .icon(glyph::NEW_PIECE)
            .trailing(language.label(Label::HistoryCurrent))
            .selected(index == current);
        if ui.add(item).clicked() { .. }
    }
});
```

| Builder | Effect |
|---|---|
| `icon(glyph)` / `icon_opt(Option<glyph>)` | A glyph of the icon font before the label |
| `trailing(text)` / `trailing_monospace(text)` | A 10 px text at the end of the row, such as a status or a time |
| `selected(bool)` | The current row: accent fill and border |
| `dimmed(bool)` | A row that no longer applies: icon and text at 45% opacity; ignored on a selected row |
| `strikethrough(bool)` | Strikes the label through |

- **States:** default, hover, selected, dimmed (with a hover fill under the pointer), disabled, plus a focus ring.
- The row has no bound value; react to `clicked()`. Rows are 28 px high and 3 px apart.

## Icons

`icons`: the icon font and the square icon button. The full spec is in `docs/icons.md`. In short:

- `icon_button(ui, size, selected, name, paint)`: a square button. `name` is the translated tooltip and accessibility label. `paint` draws the icon with the stroke it is given, usually `|p, r, s| paint_glyph(p, r, s, glyph::TRASH)`.
- `paint_glyph(painter, rect, stroke, glyph)`: draws a glyph of the icon font centered in `rect`.
- `icon_font_id(size)`: the icon font, for laying a glyph out with text, as the dock tab titles do.
- `install_icon_font` runs once in `EguiPrimaryContextPass`. Glyphs show from the next frame.

`icon_button` paints with egui's `Visuals` (`interact_selectable`), not with `Palette`, which `docs/design.md` allows. It is the only component that does.

## Dock

`dock::PanelDock` is a Bevy `Resource` that holds the panel arrangement (`egui_dock::DockState<PanelTab>`). It is not a widget.

| Call | Use |
|---|---|
| `PanelDock::load(language)` | At startup: the saved layout from `settings.json`, else the default one |
| `is_open(tab)` / `toggle(tab, center)` | For the Window menu. A reopened panel floats at `center`, 280 × 380 px. |
| `reset(language)` | Restores the default layout |
| `set_language(language)` | After a language change, for the tab context menu strings |
| `save_on_release(ctx)` | Every frame. It writes only when a pointer is released and the layout changed. |

The look of the dock comes from the Brilliant `DockTab` component and the dock tab strips, in `dock/style.rs`, with the colors of `Palette::dock`:

- `dock_style(style)`: the `egui_dock::Style` of the current theme, built every frame and passed to `DockArea::style`. 32 px strips, square tabs, the panel body with no border (the 1 px separators are the borders), and a close cross that only brightens under the pointer.
- `TabDecor` (`PanelDock::decor`) adds what that style can't express. Call `begin_frame(&state)` before the dock is shown, build the titles in `TabViewer::title` with `title(style, tab, glyph, text)`, which colors the glyph for the tab's state, and call `decorate(tab, response)` from `TabViewer::on_tab_button`. It paints the 2 px accent line along the top of the open tab and the divider right of an inactive one.

`PanelTab::Canvas` always stays on the main surface; a saved layout without it is discarded. To add a panel, add a `PanelTab` variant with a `label`, list it in `CLOSEABLE`, place it in `default_state`, and draw it in `editor/panel_viewer.rs`.

## Checklist for a new panel or dialog

1. Use these components, not raw `egui` buttons, checkboxes, combo boxes or text edits, so the panel follows the theme and the design.
2. Pass translated labels only.
3. Disable with `add_enabled` or `add_enabled_ui`.
4. Give every `Dropdown` a unique `id_salt`, and give fields in changing lists an `.id_salt`.
5. Send a `ProjectEvent` for document changes on `changed()`, or on commit for a draft.
6. Check every state in both themes (Settings → Appearance).
