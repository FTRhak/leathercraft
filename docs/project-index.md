# Project index

A map for finding code without reading whole files. It complements `CLAUDE.md`, which holds the architecture rules and invariants and isn't repeated here.

Indexed at commit `12d841a` plus the change to pieces built from a base shape and boolean child shapes. Line numbers drift, so treat them as hints and search for the symbol name if one doesn't match.

## How to read this repo cheaply

- **Start with this index, then read a line range.** All of `src/` is about 50k tokens, and `i18n/uk.rs` alone is about 6k.
- **Never open or sweep these:**
  - `target/` (11 GB, about 7,700 files)
  - `.git/`
  - `Cargo.lock` (165 KB, about 41k tokens, more than all of `src/`)
- **Scope searches.** Pass `path: src` to Grep and use `src/**/*.rs` with Glob. An unscoped `**/*` Glob returned `.git` and `target` entries and hit its 100-result limit, with no source files beyond `src/document.rs`.
- **Skip test modules unless you're changing tests.** The ranges are listed under [Library crate](#library-crate-core-no-bevy).
- **Keep build output short:**
  - `cargo test --lib --no-default-features -q` doesn't compile Bevy.
  - `cargo check --bin leathercraft --message-format=short` is the desktop type-check.
  - `cargo run` opens a GUI and can't be verified from a shell, so don't use it as a check.
- **Treat the plan docs as history.** See [Docs](#docs).

## File sizes

Tokens are estimated as bytes / 4. The estimate is rougher for `uk.rs`, because Cyrillic takes two bytes per character.

| File | Lines | ~Tokens | Crate | Role |
|---|---:|---:|---|---|
| `src/editor/mod.rs` | 86 | 600 | bin | `ViewMode`, `ShapePart` (edge or vertex picked with the Edge tool), `send`, `export`, `format_length` |
| `src/editor/editor_state_struct.rs` | 397 | 3,300 | bin | `EditorState` and its methods (primary piece plus multi-selection, primary child plus multi-selection of children in that piece, selected edge or vertex), `document_name` |
| `src/editor/drag_state_struct.rs` | 53 | 420 | bin | `ShapeRef`, `DragTarget`, `DragState` (with the `group` of pieces or `child_group` of children moved along) |
| `src/editor/shape_draft_struct.rs` | 9 | 60 | bin | `ShapeDraft` |
| `src/editor/canvas.rs` | 798 | 6,700 | bin | `canvas`, `Viewport`, tool behavior (Shift+click toggles a child of the selected piece, else a piece; the Edge tool picks a vertex or a polygon edge), hit testing (pieces, children, handles, vertices, polygon edges), painting |
| `src/editor/confirm_discard.rs` | 14 | 100 | bin | `confirm_discard`: unsaved-changes prompt |
| `src/editor/drag.rs` | 181 | 1,450 | bin | `begin_drag`, `begin_handle_drag`, `update_drag`, `finish_drag`: dragging pieces and children (the whole selection when a selected piece or child is grabbed), children and shape handles with the Select tool, and handles with the Edge tool without changing the selection |
| `src/editor/grid.rs` | 33 | 300 | bin | `draw_grid`: canvas grid and axes, grid steps |
| `src/editor/ui.rs` | 401 | 3,700 | bin | `editor_ui`, `action_button`, `file_menu`, `file_command`, `edit_menu`, `handle_hotkeys`, `window_menu`, `delete_selection`, `select_all`, `language_control`, `project_dialog`, `save_as`: top and bottom bars, dock area, menus with configured shortcuts, hotkey dispatch, new-project and Settings dialogs, language picker |
| `src/editor/panel_viewer.rs` | 193 | 1,300 | bin | `PanelViewer`: the `TabViewer` that draws each dock tab; the History tab title has an icon; the Align tab targets the selected children when there are any, else the selected pieces |
| `src/icons/mod.rs` | 87 | 760 | bin | Icon font (`src/res/leathercrafticons.ttf`): `install_icon_font`, `glyph` codes, `paint_glyph`, `icon_text` (glyph plus text); shared `icon_button` |
| `src/panels/mod.rs` | 6 | 50 | bin | Panel module declarations |
| `src/panels/dock/mod.rs` | 178 | 1,500 | bin | `PanelTab`, `PanelDock`: panel arrangement, default layout, saving and restoring it |
| `src/panels/tools/mod.rs` | 242 | 2,100 | bin | `Tool`, painted tool and operation icons, Tools panel with the regular polygon's sides field |
| `src/panels/layout/mod.rs` | 267 | 2,750 | bin | Layout panel: tree of pieces with collapsible children, lock, delete, reorder and operation controls; Shift+click toggles a piece or child |
| `src/panels/align/mod.rs` | 293 | 2,300 | bin | Align panel: `Reference` (Align To; `Parent` only for children), `Targets` (pieces or children of one piece), `AlignOptions` (kept in `EditorState::align`), painted align, distribute and spacing icons, `show_panel` |
| `src/panels/history/mod.rs` | 39 | 300 | bin | History panel: list of actions, click to jump |
| `src/panels/material/mod.rs` | 241 | 2,100 | bin | Material form, Material panel, new-project dialog |
| `src/panels/settings/mod.rs` | 95 | 700 | bin | Settings window: `SettingsDraft`, `SettingsResult`, `settings_dialog` with Default material and Hotkeys tabs |
| `src/panels/settings/material.rs` | 187 | 1,500 | bin | Default material tab: `material_tab`, its own material form with Reset to defaults |
| `src/panels/settings/hotkeys.rs` | 190 | 1,600 | bin | Hotkeys tab: `Capture`, `hotkeys_tab`, `shortcut_button`, key capture |
| `src/panels/properties/mod.rs` | 272 | 2,150 | bin | Properties panel: view or preview controls, display units, selected piece's name, size and shape fields, selected edge's length, selected vertex's corner radius, To Polygon button |
| `src/preview/mod.rs` | 261 | 2,250 | bin | 3D scene, camera, mesh extrusion |
| `src/model.rs` | 476 | 3,500 | lib | `Project`, `PatternPiece`, `ChildShape`, `BooleanOp`, `Point`, `Units`, validation |
| `src/boolean.rs` | 221 | 1,900 | lib | `Region`, `PatternPiece::region`: combining a piece's shapes |
| `src/material.rs` | 462 | 2,800 | lib | `Material` and the scales of its properties |
| `src/events.rs` | 980 | 6,650 | lib | `ProjectEvent` (including the group `MovePieces`, `RemovePieces`, `ArrangePieces`, `MoveChildren`, `ArrangeChildren`, `RemoveChildren`), `ProjectChange`, `Project::apply` |
| `src/align.rs` | 633 | 4,950 | lib | `Axis`, `Edge`, `Arrange` (Illustrator's align, distribute and spacing operations), `AlignTo` (including `Parent`, the base shape of the children's piece), `Spacing`, `piece_box`, `offsets` (pieces), `child_offsets` (children of one piece), shared `arrange_items` |
| `src/i18n/uk.rs` | 552 | 6,200 | lib | Ukrainian catalog, `plural` helper |
| `src/i18n/mod.rs` | 318 | 2,150 | lib | `Language`, `Label`, `Message` |
| `src/exports/mod.rs` | 187 | 1,200 | lib | `ExportError`, shared helpers, tests |
| `src/exports/svg.rs` | 45 | 430 | lib | `to_svg` |
| `src/exports/dxf.rs` | 25 | 200 | lib | `to_dxf` |
| `src/shapes/mod.rs` | 346 | 2,700 | lib | `Shape`: dispatch to the shape structs, constructors from a drag, `corners`, `corner_radius`, `with_corner_radius` |
| `src/shapes/corners.rs` | 243 | 1,900 | lib | Rounded corners: `round_corners` (fillet arcs), `radius_at`, `set_radius`, `valid_radii` |
| `src/shapes/rectangle.rs` | 128 | 900 | lib | `Rectangle` |
| `src/shapes/ellipse.rs` | 123 | 900 | lib | `Ellipse`, `CIRCLE_SEGMENTS` |
| `src/shapes/regular_polygon.rs` | 154 | 1,150 | lib | `RegularPolygon`, `REGULAR_POLYGON_SIDES` |
| `src/shapes/polygon.rs` | 116 | 850 | lib | `Polygon`, its edges and their lengths |
| `src/i18n/en.rs` | 497 | 4,750 | lib | English catalog |
| `src/history.rs` | 725 | 4,950 | lib | `History`, `HistoryCommand`: snapshots for undo and redo |
| `src/persistence.rs` | 169 | 1,300 | lib | `save_project`, `load_project`, `write_export` |
| `src/settings/mod.rs` | 123 | 1,000 | bin | `settings.json` in the OS config directory: language, panel layout, and `Preferences` (default material, hotkeys) with `load_preferences`, `save_preferences` |
| `src/settings/hotkeys.rs` | 335 | 2,700 | bin | `Action` (bindable commands, default shortcuts), `Hotkeys` (`from_json`, `to_json`, `first`, conflicts, `triggered`), `format_shortcut`, `parse_shortcut`; tests |
| `src/document.rs` | 70 | 550 | bin | `ActiveProject` (`new`, project and history), `ProjectMessage`, `HistoryMessage`, `apply_project_events` |
| `src/main.rs` | 51 | 350 | bin | App wiring and system schedule; loads `Preferences` and starts the project with their default material; the editor UI starts one pass after the icon font is installed |
| `src/lib.rs` | 9 | 40 | lib | Module declarations |

## Library crate (core, no Bevy)

| File | Lines | Contents |
|---|---:|---|
| `material.rs` | 3-4 | `MIN_THICKNESS_MM`, `MAX_THICKNESS_MM` |
| | 8-104 | `Origin`, `Treatment`, `ThicknessCategory` (`from_millimeters`) |
| | 106-255 | `ColorGroup`, `LeatherColor` (`group`, `rgb`) |
| | 258-380 | `Texture`, `Strength`, `Flexibility`, `WaterResistance`, `HeatResistance` |
| | 383-420 | `Material`: `thickness_category`, `has_valid_thickness` |
| | 421-462 | Tests |
| `model.rs` | 8-13 | `PROJECT_VERSION` (2) and validation limits (`MAX_PIECES`, `MAX_POINTS_PER_PIECE`, `MAX_CHILDREN_PER_PIECE`, `MAX_COORDINATE_MM`, `MIN_POLYGON_AREA_MM2`) |
| | 17-70 | `Units`, `Point`, `Bounds` (`union`) |
| | 71-117 | Ring helpers: `ring_bounds`, `ring_contains`, `signed_area` |
| | 119-140 | `BooleanOp`, `ChildShape` |
| | 143-262 | `PatternPiece`: `bounds`, `child`, `child_index`, `next_child_id`, `translate`, `validate_shape`, `validate_ring`, `validate_shapes` |
| | 265-353 | `Project`: `bounds`, `validate` (301), `next_piece_id`, `add_piece`, `remove_piece` |
| | 355-476 | Tests |
| `shapes/mod.rs` | 1-40 | Re-exports, `Shape` enum (`Polygon`, `Rectangle`, `Ellipse`, `RegularPolygon` wrapping the structs of the same name), `to_world`, `to_local`, `positive` |
| | 43-200 | `rectangle_from_corners`, `ellipse_from_center`, `regular_polygon_from_center`, `ring`, `to_polygon` (keeps corners and radii), `corners`, `corner_radius`, `with_corner_radius`, `bounds`, `contains`, `center`, `translate`, `handles` (corners, not ring points), `with_handle_at`, `has_valid_size` |
| | 202-346 | `test_support` (`validate`, `assert_near`), tests |
| `shapes/corners.rs` | 14-128 | `radius_at`, `set_radius` (keeps the list to the vertex count, empties it when all are 0), `valid_radii`, `round_corners`, `fillet` (radius reduced so an arc reaches at most halfway along each edge); tests from 130 |
| `shapes/rectangle.rs` | 5-68 | `Rectangle` (with `radii`): `from_corners`, `ring`, `corners`, `with_handle_at`, `has_valid_size`; tests from 70 |
| `shapes/ellipse.rs` | 6-79 | `CIRCLE_SEGMENTS`, `Ellipse`: `from_center`, `ring`, `handles`, `with_handle_at`, `has_valid_size`; tests from 81 |
| `shapes/regular_polygon.rs` | 7-85 | `REGULAR_POLYGON_SIDES` (3..=64), `ROTATION_SNAP`, `RegularPolygon` (with `radii`): `from_center`, `ring`, `corners`, `with_handle_at`, `has_valid_size`; tests from 87 |
| `shapes/polygon.rs` | 4-67 | `Polygon` (`points`, `radii`): `new`, `ring` (rounded), `has_valid_size`, `translate`, `with_handle_at`, `edge`, `edge_length`, `with_edge_length` (moves the edge's end along it); tests from 69 |
| `boolean.rs` | 11-53 | `ROUND_PRECISION_MM`, `Region`: `rings`, `vertices`, `hole_indices`, `bounds`, `contains` |
| | 67-96 | `PatternPiece::region` |
| | 98-221 | Tests |
| `events.rs` | 10-128 | `ProjectEvent`, `ProjectChange` |
| | 130-314 | `Project::apply` |
| | 316-358 | `edit` (validates a copy), `piece_mut`, `unlocked_piece_mut`, `child_mut`, `child_index` |
| | 360-980 | Tests |
| `history.rs` | 6-48 | `MAX_HISTORY`, `HistoryCommand`, `MergeKey`, `Entry`, `History` |
| | 50-268 | `new`, `record`, `apply`, `actions`, `position`, `can_undo`, `can_redo`, `describe` |
| | 270-725 | Tests |
| `persistence.rs` | 7-34 | `save_project`, `load_project`, `write_export` |
| | 36-169 | Tests |
| `exports/mod.rs` | 14-38 | `ExportError`, conversion to `Message` |
| | 41-66 | `validate_export` (returns each piece's `Region`), `number` |
| | 68-187 | Tests |
| `exports/svg.rs` | 4 | `to_svg` |
| `exports/dxf.rs` | 5 | `to_dxf` |
| `i18n/mod.rs` | 14-66 | `Language`: `tag`, `from_tag`, `native_name`, `label`, `message`, `unit_symbol` |
| | 70-165 | `Label`, with nested variants for material values and `Operation(BooleanOp)` |
| | 169-226 | `Message` |
| | 235-279 | Tests |

## Binary crate (desktop)

### `editor/mod.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 27-40 | Types | `ViewMode`, `ShapePart` (`Edge` or `Vertex`, by index into `Shape::corners`) |
| 34-36 | Events | `send` |
| 38-70 | Menu bar helpers | `export` |
| 72-78 | Formatting | `format_length`, also used by the Properties panel |

### `editor/editor_state_struct.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 12-48 | Type | `EditorState`, with `op_mode`, `selected` (primary piece), `pieces` (every selected piece), `selected_child` (primary child), `children` (every selected child of that piece) and `selected_part` (edge or vertex) |
| 39-62 | Defaults | `Default` impl |
| 67-242 | Methods | `new`, `report`, `report_error`, `snap`, `snap_point`, `reset_document`, `on_project_change`, `on_history_restore`, `selected_child_id`, `selected_children`, `clear_children`, `select`, `toggle_child`, `focus_child`, `select_all_children`, `is_selected`, `toggle_piece`, `focus`, `select_all`, `prune_selection`, `select_part`, `selected_part` (only while its shape is the one shown in Properties), `clear_released_drag`, `displayed` |
| 211-216 | Helpers | `document_name` |

### `editor/canvas.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 15-19 | Constants | Zoom limits, hit radii, edge label limit |
| 21-36 | Viewport | `Viewport` (`to_screen`, `to_world`) |
| 38-109 | Canvas | `canvas`, `fit_view` |
| 114-195 | Select and Edge tools | `handle_select`, `handle_edge` (click picks a vertex of the selected piece, else a polygon edge; drag moves a vertex; Esc clears), `child_of_selected_at`, `select_at` |
| 152-262 | Drawing tools | `handle_draw`, `finish_draft`, `commit_shape` (new piece or `AddChild`), `handle_shape`, `add_piece` |
| 300-440 | Hit testing | `piece_hit`, `piece_at`, `child_at`, `handle_at`, `segment_distance`, `edge_at`; `vertex_at` sits after `draw_edges` |
| 403-702 | Painting | `op_color`, `draft_color`, `dashed_ring`, `draw_handles`, `draw_pieces`, `draw_edges` (selected and hovered edge, selected vertex), `draw_edge_labels` (by corners, so arcs get no labels), `fill_polygon`, `draw_draft`, `draw_shape_draft` |

### `editor/ui.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 21-160 | Top-level layout | `editor_ui`, which also shows the dock area, the new-project dialog and the Settings window (OK saves `Preferences`) |
| 163-266 | Menus | `action_button` (label plus the action's first shortcut), `file_menu`, `file_command` (New, Open, Save, Save As; shared with hotkeys), `edit_menu` (with Settings…), `window_menu` |
| 268-372 | Input | `handle_hotkeys` (dispatches `Hotkeys::triggered`, skipped while drawing, in a dialog or in a text field), `delete_selection` (several children go as `RemoveChildren`, several pieces as `RemovePieces`, locked ones stay), `select_all` (all children of the selected piece while one is selected) |
| 374-388 | Language | `language_control` |
| 390-401 | File dialog | `project_dialog`, `save_as` |

### `panels/`

| File | Lines | Symbols |
|---|---:|---|
| `mod.rs` | 1-8 | Module declarations |
| `dock/mod.rs` | 11-32 | `PanelTab`: `CLOSEABLE`, `label` |
| | 35-121 | `PanelDock`: `load`, `is_open`, `toggle`, `reset`, `set_language`, `save_on_release` |
| | 123-163 | `window_id`, `is_usable`, `restore_windows`, `default_state`, `translations` |
| `tools/mod.rs` | 8-11 | Icon constants |
| | 14-129 | `Tool` (Select, Edge, Draw, Rectangle, Ellipse, RegularPolygon): `ALL`, `label`, `hint`, `shape`, `paint_icon` |
| | 119-175 | `arc`, `paint_operation_icon`, `paint_new_piece_icon` |
| | 191 | `show_panel`: tools, the new-piece and operation modes, then the sides field while the Regular polygon tool is active (`EditorState::polygon_sides`) |
| `layout/mod.rs` | 9-10 | Constants |
| | 14 | `expand_toggle`: expands or collapses a piece's children |
| | 31-52 | `paint_trash` (icon font), `paint_arrow`, `shape_label`; the lock button uses the font's lock and unlock glyphs |
| | 54 | `Selection` (read-only), `Pick` (the clicked row, returned to `PanelViewer`) |
| | 60 | `child_row`: delete, reorder, operation picker, select |
| | 135 | `piece_row` |
| | 195 | `show_panel`, which returns the `ProjectEvent` to send and the `Pick` |
| `history/mod.rs` | 6 | `show_panel`, which returns the `HistoryCommand` to send |
| `material/mod.rs` | 15-103 | `DialogResult`, `combo`, `swatch`, `color_combo`, `thickness_row` |
| | 105 | `material_fields`, shared by the dialog and the Material panel |
| | 189 | `show_panel`, which returns the `ProjectEvent` to send |
| | 214 | `new_project_dialog` |
| `settings/mod.rs` | 12-42 | `DIALOG_WIDTH`, `SettingsTab`, `SettingsDraft` (edited copy of `Preferences`), `SettingsResult` |
| | 45 | `settings_dialog`: modal with Default material and Hotkeys tabs; OK is disabled while shortcuts conflict |
| `settings/material.rs` | 16 | `material_tab`: its own form on the draft's default material, independent of `material_fields`, with Reset to defaults |
| | 37-187 | `basic_rows`, `physical_rows`, `combo`, `thickness_row`, `color_row`, `swatch` |
| `settings/hotkeys.rs` | 8-21 | `HOTKEYS_HEIGHT`, `Capture`, `HotkeyChange` |
| | 23 | `hotkeys_tab`, `shortcut_button` |
| | 140 | `capture_keys` (next key press becomes the shortcut, Esc cancels), `normalize` (Ctrl saved as Command) |
| `properties/mod.rs` | 10-31 | `EditorView`, `ViewControls` (editor or preview), `Selection` (displayed piece, child, selected edge or vertex, rename buffer) |
| | 32-59 | `view_controls`, `units_control` |
| | 63 | `selected_piece_details`: name, size or result error, fields of the selected base or child, To Polygon button for a primitive |
| | 140 | `shape_fields`: center, size and rotation of a primitive in display units; a polygon's point count and the selected edge's length; the selected vertex's corner radius |
| | 240 | `show_panel`, which returns the `ProjectEvent` to send |

### `preview/mod.rs`

| Lines | Symbols |
|---:|---|
| 11-46 | Scene constants, `PreviewPlugin`, `PreviewSettings` |
| 48-70 | `setup_scene` |
| 73-123 | `rebuild_preview`, which skips pieces without a valid region |
| 125-142 | `update_camera` |
| 144-157 | `show_controls` |
| 159-261 | `build_piece_mesh` (from a `Region`) |

## Where to look for common tasks

| Task | Read | Change |
|---|---|---|
| Add a project edit | `events.rs` 10-229, `history.rs` `describe` | Add a `ProjectEvent` variant and handle it in `apply`, give it an entry label in `History::describe` (an `Action…` `Message`), then return it from a panel or send it from `editor/` with `send` |
| Add a validation rule | `model.rs` 8-13, 200-262, 301-333 | `validate`, plus a `Message` variant and both catalogs |
| Add UI text | `i18n/mod.rs` 70-166 | A `Label` or `Message` variant and an arm in `en.rs` and `uk.rs`; the compiler flags missing arms |
| Add a language | `i18n/mod.rs` 14-66 | `Language`, `ALL`, `tag`, `from_tag`, `native_name`, and a new catalog file |
| Add a canvas tool | `panels/tools/mod.rs`, `editor/canvas.rs` 38-93, 152-262 | `Tool` and its `ALL`, `label`, `hint`, `paint_icon`, a dispatch arm in `canvas`; a new primitive also goes in `Shape` (`ring`, `handles`, `with_handle_at`, `has_valid_size`), `shape_label` and `shape_fields` |
| Add a boolean operation | `model.rs` `BooleanOp`, `boolean.rs` `region` | A `BooleanOp` variant and its `ALL`, its `OverlayRule`, catalog arms, `op_color` and `paint_operation_icon` |
| Add or change a material value | `material.rs`, `i18n/en.rs`, `i18n/uk.rs` | The enum and its `ALL`, plus catalog arms; a new property also goes in `Material`, `material_fields` and the Settings form in `panels/settings/material.rs` |
| Change the file format | `model.rs` 119-160, 265-278, `shapes/mod.rs` 14-23 and the shape structs, `persistence.rs` 7-30 | Bump `PROJECT_VERSION` and decide how older files are handled |
| Change SVG or DXF output | `exports/svg.rs`, `exports/dxf.rs` | `to_svg` or `to_dxf` |
| Change the 3D look or mesh | `preview/mod.rs` 49-123, 159-261 | `setup_scene`, `rebuild_preview`, `build_piece_mesh` |
| Change align or distribute | `align.rs` `arrange_items`, `panels/align/mod.rs` | The rules live in `arrange_items`, shared by `offsets` (pieces) and `child_offsets` (children); the panel only builds an `ArrangePieces` or `ArrangeChildren` event |
| Add a dockable panel | `panels/dock/mod.rs`, `editor/panel_viewer.rs` | A `PanelTab` variant, its `CLOSEABLE` and `label` entries, `default_state`, and arms in `PanelViewer::ui` and `scroll_bars` |
| Change how the app is compiled or linked | `Cargo.toml`, `.cargo/config.toml` | See [Build configuration](#build-configuration) |
| Change system scheduling | `main.rs`, `document.rs` | Keep the ordering rule in `CLAUDE.md` |
| Add a hotkey or preference | `settings/hotkeys.rs`, `settings/mod.rs`, `editor/ui.rs` `handle_hotkeys` | A hotkey: an `Action` variant with `ALL`, `label`, `default_shortcuts`, and a dispatch arm. A preference: a field of `Preferences`, a raw-JSON section in `Settings` read in `load_preferences` and written in `save_preferences`, and a tab or field in `panels/settings` |

## Build configuration

| File | Holds |
|---|---|
| `Cargo.toml` | The `dynamic_linking` feature (on by default) builds Bevy into a separate `bevy_dylib` DLL, so a code change relinks only the small `leathercraft.exe`. `[profile.dev.package."*"]` sets `opt-level = 3` for dependencies, which also keeps that DLL under the 65,535-export limit of Windows. |
| `.cargo/config.toml` | `rust-lld` as the Windows linker; the `cargo release` alias, a static build without `dynamic_linking`. |

## Tests

There are 102 tests inline in the library crate, plus 4 in the binary (`settings::hotkeys`, run with `cargo test --bin leathercraft`, which compiles Bevy):

| Module | Tests |
|---|---:|
| `model` | 6 |
| `shapes` | 26 |
| `boolean` | 7 |
| `material` | 3 |
| `align` | 13 |
| `events` | 17 |
| `history` | 13 |
| `i18n` | 5 |
| `exports` | 7 |
| `persistence` | 5 |

Apart from shortcut parsing, `document`, `editor`, `preview`, `settings` and `panels` have no tests, so changes there need `cargo check` and a manual run.

## Docs

| File | Status |
|---|---|
| `CLAUDE.md` | Current. Loaded automatically in every session, so keep it short. |
| `docs/icons.md` | Current. Spec for the icon font, `paint_glyph` and `icon_button`: adding glyphs, sizes, toggles, disabled state. |
| `docs/history.md` | Current. Spec for undo, redo and the History panel: user controls, snapshots and merging, the message flow in the binary, adding a new edit. |
| `docs/settings.md` | Current. Spec for `settings.json` and the Settings window: sections and their fallbacks, `Action` and `Hotkeys`, shortcut text format, the draft and key capture, adding a hotkey or preference. |
| `docs/i18n.md` | Current. Spec for the localization module: `Label` vs `Message`, rendering, language selection, adding text and languages, translation conventions. |
| `docs/plan.md` | Historical MVP plan. It still says the repo contains only a license. Its todo 6 asks for build and run docs, and there is no README; `CLAUDE.md` only lists the commands. |


## Refreshing this index

To regenerate the symbol list and line numbers:

```sh
rg -n "^\s*(pub(\(\w+\))? )?(fn|struct|enum|trait|impl|mod|const) " src
```

Update it when a file is added, moved or split, or when `editor/mod.rs` or `editor/canvas.rs` is restructured. Small line drift doesn't need a refresh.
