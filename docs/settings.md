# Settings and preferences (`settings`, `panels::settings`)

How the app keeps interface state between runs, and how the user edits the default material, the hotkeys and the theme in the Settings window. All of it is interface state of the binary: nothing here is part of a `Project` or the `.leathercraft` format.

| File | Contents |
|---|---|
| `src/settings/mod.rs` | `settings.json` storage: `Settings` (private), `Preferences`, `load_preferences`, `save_preferences`, `load_language`, `save_language`, `load_dock`, `save_dock` |
| `src/settings/hotkeys.rs` | `Action`, `Hotkeys`, `format_shortcut`, `parse_shortcut`, tests |
| `src/settings/theme.rs` | `Theme` (Dark, Light), `apply_theme`: sets egui's theme when `Preferences::theme` changes |
| `src/panels/settings/mod.rs` | Settings window: `SettingsDraft`, `SettingsResult`, `settings_dialog` |
| `src/panels/settings/material.rs` | Default material tab: `material_tab` |
| `src/panels/settings/hotkeys.rs` | Hotkeys tab: `Capture`, `hotkeys_tab`, `capture_keys` |
| `src/panels/settings/appearance.rs` | Appearance tab: `appearance_tab` (the theme) |
| `src/editor/ui.rs` | Opening the window (`edit_menu`), applying its result (`editor_ui`), dispatching hotkeys (`handle_hotkeys`), menu shortcuts (`action_button`), language picker (`language_control`) |
| `src/ui_components/dock/mod.rs` | `PanelDock::load`, `save_on_release`: the panel layout section |
| `src/main.rs` | Loads the language and `Preferences` at startup |

## For users

| Action | How |
|---|---|
| Open the Settings window | **Edit → Settings…** |
| Change the material new projects start with | **Default material** tab |
| Switch between the dark and light theme | **Appearance** tab |
| Find a command or shortcut | **Hotkeys** tab: type in the search field above the list |
| Change a shortcut | **Hotkeys** tab: click the shortcut, then press the new keys |
| Add a shortcut to a command | Click **+** in its row, then press the keys |
| Remove a shortcut | Click **×** next to it |
| Undo everything in a tab | **Reset to defaults** (only that tab) |
| Keep the changes | **OK** |
| Discard the changes | **Cancel** or Esc |

- Changes take effect only on **OK**. Cancel, Esc or closing discards the whole draft, all tabs included.
- Clicking outside the window does nothing, so a stray click can't lose the edits.
- Esc first closes an open drop-down, and only then the window.
- While a shortcut waits for keys, its button reads *Press keys… (Esc to cancel)*. The next key press, with its modifiers, becomes the shortcut. Esc on its own cancels; Esc with a modifier is a valid shortcut.
- A shortcut assigned more than once, to two commands or twice to one, is shown in red with a warning under the list. **OK** is disabled until the conflict is resolved, and also while a shortcut is waiting for keys.
- The default material is used by the new-project dialog and by the project the app starts with. Changing it doesn't change the current project or its pieces.
- Ctrl and Cmd are the same modifier: a shortcut saved as `Ctrl+S` is Ctrl+S on Windows and Linux and Cmd+S on macOS.
- Hotkeys don't act while a text field has focus, while a shape is being drawn, or while the new-project or Settings dialog is open. Menus still work then.
- The interface language (top bar) and the panel layout (dragging panels) are saved too, immediately and without the Settings window.

### Default shortcuts

| Command | Shortcuts |
|---|---|
| New | Ctrl+N |
| Open | Ctrl+O |
| Save | Ctrl+S |
| Save As | Ctrl+Shift+S |
| Undo | Ctrl+Z |
| Redo | Ctrl+Shift+Z, Ctrl+Y |
| Select all | Ctrl+A |
| Delete selection | Delete, Backspace |
| Select tool | V |
| Edge tool | E |
| Draw tool | P |
| Rectangle tool | R |
| Ellipse tool | L |
| Regular polygon tool | G |
| Toggle editor / 3D preview | Tab |
| Snapping on / off | F9 |

## Storage: `settings.json`

One file, `<config dir>/leathercraft/settings.json`, where the config directory comes from `dirs::config_dir()` (`%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `$XDG_CONFIG_HOME` or `~/.config` on Linux). It is pretty-printed JSON with a trailing newline. There is no version field.

```json
{
  "language": "uk",
  "dock": { "...": "egui_dock state and floating window rects" },
  "default_material": { "origin": "...", "thickness_mm": 2.0, "...": "..." },
  "hotkeys": {
    "undo": ["Ctrl+Z"],
    "redo": ["Ctrl+Shift+Z", "Ctrl+Y"],
    "delete": []
  },
  "snap": { "enabled": true, "grid": true, "grid_step_mm": 1.0, "vertices": true, "guides": true, "distance_px": 8.0 },
  "theme": "light"
}
```

| Section | Type in `Settings` | Owner | Written | Missing or unusable |
|---|---|---|---|---|
| `language` | `Option<String>`, a `Language::tag` | `EditorState::language` | `save_language`, when the language picker changes | OS locale via `sys-locale`, else `Language::default()` |
| `dock` | raw `serde_json::Value` | `PanelDock` | `save_dock`, on a pointer release when the layout differs from the last saved one | `default_state()` layout |
| `default_material` | raw `serde_json::Value` | `Preferences::default_material` | `save_preferences`, on Settings OK | `Material::default()` |
| `hotkeys` | raw `serde_json::Value` | `Preferences::hotkeys` | `save_preferences`, on Settings OK | Per action: `Action::default_shortcuts` |
| `theme` | raw `serde_json::Value`, `"dark"` or `"light"` | `Preferences::theme` | `save_preferences`, on Settings OK | `Theme::Dark`, with a warning in the log |
| `snap` | raw `serde_json::Value` | `EditorState::snap` (`leathercraft::snap::SnapSettings`) | `save_snap`, from `EditorState::save_snap_when_settled` once a value changed and no pointer button is down, or at once by the F9 hotkey | `SnapSettings::default()`; missing fields take their defaults (`#[serde(default)]`), and numbers are clamped (`clamped`). See `docs/snap.md` |

Every section is optional and omitted while unset (`skip_serializing_if`). `language` stays absent until the user picks one, so the OS locale keeps applying until then.

### Fallbacks

Each section after `language` is stored as raw JSON and parsed on its own, so one bad section never costs the others:

- **`default_material`**: deserialized as a whole `Material`. If it doesn't parse (an unknown enum value, a missing field) or `has_valid_thickness` fails, the whole material falls back to the default, with a warning in the log.
- **`hotkeys`**: a map from the snake_case `Action` name to a list of shortcut strings. If the value isn't such a map, every action gets its defaults. Otherwise, per entry:
  - an unknown action name is skipped with a warning;
  - an unparsable shortcut string is skipped with a warning, the rest of that action's list is kept;
  - an action missing from the map keeps its defaults;
  - an action with an empty list is deliberately unbound, which is different from missing.
- **`dock`**: kept even when it doesn't parse into a usable layout, so `PanelDock` can compare against it; the default layout is shown instead.

### Reading and writing

- `read_settings` reads and parses the file on every call. It returns `None` if the file or config directory is missing, or if the top level doesn't parse as `Settings`.
- Every save goes through `update_settings`: read, change one section, write the whole file. Sections the call doesn't touch are written back unchanged, including raw JSON the app couldn't use.
- Write failures are logged with `bevy::log::error!` and never shown to the user: preferences aren't worth interrupting work over.
- Log messages are English and don't go through `i18n`.

### Known limits

- `language` is typed as a string, not raw JSON. A non-string value there makes the whole file unparsable, so every section falls back, and the next save rewrites the file from defaults plus the saved section.
- More generally, a file whose top level doesn't parse is replaced on the next save. Nothing is backed up.
- Two running instances share the file. Each save is a read-modify-write of the whole file, so the last writer wins for the section it changes, but they don't lose each other's other sections.
- Unknown hotkey actions are dropped on the next `save_preferences`, because `to_json` writes only the bindings in memory.

## Shortcuts model (`settings::hotkeys`)

### `Action`

A command that can be bound. Serialized in snake_case (`save_as`, `tool_regular_polygon`).

| Item | Purpose |
|---|---|
| `Action::ALL` | Every action, in the order the Hotkeys tab lists them |
| `label()` | The `Label` shown in the Hotkeys tab, reused from the menus and tools |
| `tool()` | The `Tool` a tool action switches to, else `None` |
| `default_shortcuts()` | The shortcuts of a fresh install; `Modifiers::COMMAND` for Ctrl/Cmd |

### `Hotkeys`

A `BTreeMap<Action, Vec<KeyboardShortcut>>`. The order within a list matters: the first shortcut is the one menus show.

| Item | Purpose |
|---|---|
| `Hotkeys::default()` | Every action with its `default_shortcuts` |
| `from_json(value)`, `to_json()` | Conversion to and from the `hotkeys` section, with the fallbacks above |
| `shortcuts(action)`, `shortcuts_mut(action)` | Read or edit one action's list |
| `first(action)` | The shortcut shown next to the action in menus (`action_button`) |
| `is_conflicting(shortcut)` | Bound more than once anywhere, including twice to one action |
| `has_conflicts()` | Any conflict; disables OK |
| `triggered(ctx)` | Consumes the first pressed shortcut and returns its action |

`triggered` tries shortcuts with more modifiers first. egui's `consume_shortcut` ignores an extra Shift or Alt, so without that order Ctrl+Z would also fire on Ctrl+Shift+Z and Redo could never trigger. Ctrl and Command count as one modifier.

### Text format

`format_shortcut` writes `Ctrl+Shift+Alt+<Key>`, in that modifier order, with egui's `Key::name` for the key (`Z`, `Delete`, `Tab`, `F5`, `+`). `Ctrl` stands for Command on macOS.

`parse_shortcut` accepts:

- modifier names in any case: `ctrl`, `control`, `cmd` and `command` all mean `COMMAND`; `shift`; `alt` and `option`;
- key names as egui spells them, or in any case (`delete` becomes `Delete`);
- a trailing `+` as the key itself (`Ctrl++`);
- spaces around the parts.

Anything else returns `None`. The menus and the Hotkeys tab display shortcuts with `ctx.format_shortcut`, the platform's own spelling (`⌘S` on macOS), not the saved text.

## The Settings window (`panels::settings`)

### Flow

```text
Edit → Settings…
   │  clears the polygon and shape drafts
   ▼
EditorState::settings_dialog = Some(SettingsDraft::new(&preferences))
   │  each frame, in editor_ui:
   ▼
settings_dialog(ctx, draft, language) -> Option<SettingsResult>
   ├─ Save:   if the draft differs, *preferences = draft, save_preferences; close
   ├─ Cancel: close, draft dropped
   └─ None:   keep showing
```

- `Preferences` is a Bevy `Resource`, inserted in `main` from `load_preferences`. `editor_ui` holds it as `ResMut` and is its only writer.
- The window never edits `Preferences` directly. It edits `SettingsDraft::preferences`, a clone, and the caller copies it back on `Save`.
- `Save` writes the file only when the draft differs from the current preferences (`Preferences: PartialEq`).
- Opening the window clears the polygon draft and shape draft, because they would otherwise still react to Enter behind it.

### `SettingsDraft`

| Field | Meaning |
|---|---|
| `preferences` | The edited copy, public so the caller can apply it |
| `tab` | `SettingsTab::Material` (default), `SettingsTab::Hotkeys` or `SettingsTab::Appearance` |
| `capture` | `Some(Capture)` while a shortcut waits for keys |
| `hotkey_filter` | The text of the Hotkeys search field; kept while switching tabs, dropped with the draft |

### `settings_dialog`

An `egui::Modal` of fixed width `DIALOG_WIDTH` (420 px), with the title, a row of tab buttons, the active tab, then OK and Cancel.

- `capture_keys` runs **before** the modal is drawn, so a captured key reaches neither the window's widgets nor the Esc check.
- OK is enabled only when `capture.is_none()` and there are no conflicts.
- Esc returns `Cancel` only when no popup (combo box) is open, so the first Esc closes the popup. Esc in a focused input field only leaves the field: the input consumes the key.
- The modal has no backdrop-click handling, so clicking outside is ignored.

### Default material tab (`material_tab`)

- Two striped grids, **Basic properties** (origin, treatment, thickness, color, texture) and **Physical properties** (strength, flexibility, water and heat resistance), editing `draft.preferences.default_material`.
- It has its own form, separate from `panels::material::material_fields`, which edits a piece through events. A new `Material` property has to be added to both.
- Thickness is a `NumberInput` (step 0.1, two decimals, the `mm` unit) limited to `MIN_THICKNESS_MM..=MAX_THICKNESS_MM`, always in millimeters whatever the display units, with the thickness category next to it. Since the range is enforced, the saved material always passes `has_valid_thickness`.
- Values that have a description in the catalog show it on hover, both in the list and on the closed combo box.
- Colors are grouped by `ColorGroup`, each with a swatch.
- **Reset to defaults** sets the draft material to `Material::default()`; the hotkeys are untouched.
- The tab is wrapped in `push_id("settings_material")`, so its widget ids don't collide with the Material panel or the new-project dialog.

### Hotkeys tab (`hotkeys_tab`)

- A `SearchInput` above the list filters it: a row is shown when its label or one of its shortcuts, as displayed, contains the text, ignoring case. Removing or capturing works on the real indices, so filtering never changes which shortcut is edited.
- A scrollable grid (max height 360 px) with one row per shown `Action::ALL` entry: its label, then a button per shortcut with a **×** after each, then **+**.
- Clicks are collected into one `HotkeyChange` and applied after the grid is drawn, so the list isn't changed while it is being iterated:

| Change | Effect |
|---|---|
| `Capture(Capture { action, index })` | Starts waiting: `index` is `Some(i)` to replace shortcut `i`, `None` to add one |
| `Remove(action, index)` | Removes that shortcut and cancels any capture |
| `Reset` | `Hotkeys::default()` and cancels any capture |

- Clicking another shortcut while one is waiting moves the capture to the new one.
- Conflicting shortcuts are drawn in `error_fg_color`, and the conflict message appears next to **Reset to defaults**.

### Appearance tab (`appearance_tab`)

- **Theme**: radio buttons for `Theme::ALL`, editing `draft.preferences.theme`. Like the other tabs, it applies only on OK.
- `settings::apply_theme` (in `EguiPrimaryContextPass`) calls `ctx.set_theme` when the theme differs from the one it last applied, including on the first pass. `preview::apply_theme` (in `Update`) sets `ClearColor` and the ground plane's color the same way.
- The canvas reads `visuals.dark_mode` through `editor::palette::Palette::of`, which picks the grid, outline, label, stitch hole, centerline and edge colors for that background. Selection, snap and boolean-operation colors are the same in both themes.

### Key capture (`capture_keys`)

While `draft.capture` is set, each frame:

1. Find the first `Event::Key` that is pressed and not a repeat, and **remove it** from egui's input, so no widget, shortcut or Esc handler sees it.
2. Clear the capture whatever the key.
3. Esc with no modifiers cancels.
4. Otherwise normalize the modifiers: Ctrl, Command and macOS Cmd become `COMMAND`; Shift and Alt are kept; anything else is dropped. This keeps captured shortcuts in the same form as the defaults, so conflict detection compares like with like.
5. Replace shortcut `index` if it still exists; otherwise add the shortcut, unless the action already has it.

A shortcut that is already bound to *another* action is accepted and then flagged as a conflict; the user resolves it.

## How the rest of the app uses it

| Consumer | Uses |
|---|---|
| `main` | `load_language` and `load_snap` for `EditorState::new`; `load_preferences` for the resource and the startup project's `default_material` |
| `file_command` (New) | Opens the new-project dialog with `preferences.default_material` |
| `action_button` | Menu items with `hotkeys.first(action)` as their shortcut text |
| `handle_hotkeys` | `hotkeys.triggered(ctx)`, then a tool switch for tool actions or a `match` for the rest; skipped while drawing, in a dialog, or when `ctx.wants_keyboard_input()` |
| `language_control` | `save_language`, then `PanelDock::set_language` |
| `PanelDock` | `load_dock` on startup, `save_dock` from `save_on_release` |

## Adding things

### A new hotkey

1. Add an `Action` variant, put it in `Action::ALL` (fix the array length) where it should appear in the Hotkeys tab, and give it a `label` and `default_shortcuts`. Make sure the defaults don't conflict: the `json_round_trip` test asserts that.
2. If it's a tool, return it from `tool()`. Otherwise add an arm to the `match` in `handle_hotkeys`. That `match` ends in `_ => {}`, so the compiler won't flag a missing arm: the shortcut would just do nothing.
3. If it has a menu item, draw it with `action_button` so the menu shows the configured shortcut.
4. If it needs a new `Label`, add it to both catalogs (see `docs/i18n.md`).

Old `settings.json` files need nothing: the new action is missing from them, so it gets its defaults.

### A new preference

1. Add a field to `Preferences`.
2. Add an `Option<serde_json::Value>` section to `Settings` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Keep it raw JSON so a bad value falls back on its own.
3. Parse it in `load_preferences` with a logged fallback to the default, and write it in `save_preferences`.
4. Edit it in the Settings window: a field in an existing tab, or a `SettingsTab` variant, its tab button and its `match` arm in `settings_dialog`, plus a `Label` for the tab name.
5. If the field can make OK unsafe, add its check to `can_save`.

### A setting saved outside the window

Like `language` and `dock`: own the value in `EditorState` or a resource, give it a `load_*` and `save_*` pair in `settings/mod.rs` that goes through `read_settings` and `update_settings`, and save it when it changes rather than every frame.

## Rules

- **Never in `Project`.** Settings are interface state. The one crossing point is copying `default_material` into a new project, after which the project owns its copy.
- **Every write goes through `update_settings`**, so other sections survive. Never write `settings.json` directly.
- **Each section falls back on its own**, with a warning in the log. A new section must not be able to break the parse of the others.
- **Commands are bound through `Action`**, never matched on hard-coded keys. Dialog-internal keys (Esc to close, Enter to confirm) are the exception.
- **The Settings window edits a draft.** Nothing reaches `Preferences` or disk before OK.
- **`Preferences` has one writer**, `editor_ui`.
- **User-facing text goes through `i18n`**; log messages stay in English.

## Tests

`src/settings/hotkeys.rs` has 4 tests in the binary crate, which compiles Bevy:

```sh
cargo test --bin leathercraft settings
```

They cover formatting and parsing every default shortcut back unchanged, parsing in any case and rejecting unknown modifiers and keys, the per-entry fallbacks of `from_json`, and the JSON round trip with no default conflicts. `load_preferences`, the file fallbacks, the Settings window and key capture have no tests and need a manual run.
