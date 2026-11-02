# Project index

A map for finding code without reading whole files. It complements `CLAUDE.md`, which holds the architecture rules and invariants and isn't repeated here.

Indexed at commit `5afa155` plus the Tools and Layout panel, material, dockable panel and history changes. Line numbers drift, so treat them as hints and search for the symbol name if one doesn't match.

## How to read this repo cheaply

- **Start with this index, then read a line range.** All of `src/` is about 34k tokens, and `i18n/uk.rs` alone is about 4.5k.
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
| `src/editor/mod.rs` | 164 | 1,150 | bin | `ViewMode`, `send`, menu helpers, sidebar, `format_length` |
| `src/editor/editor_state_struct.rs` | 159 | 1,250 | bin | `EditorState` and its methods, `document_name` |
| `src/editor/drag_state_struct.rs` | 23 | 150 | bin | `DragState`, `DragTarget` |
| `src/editor/shape_draft_struct.rs` | 7 | 50 | bin | `ShapeDraft` |
| `src/editor/canvas.rs` | 395 | 3,200 | bin | `canvas`, `Viewport`, tool behavior, hit testing, painting |
| `src/editor/confirm_discard.rs` | 14 | 100 | bin | `confirm_discard`: unsaved-changes prompt |
| `src/editor/drag.rs` | 118 | 850 | bin | `begin_drag`, `update_drag`, `finish_drag`: dragging pieces, holes and vertices with the Select tool |
| `src/editor/grid.rs` | 33 | 300 | bin | `draw_grid`: canvas grid and axes, grid steps |
| `src/editor/ui.rs` | 219 | 2,000 | bin | `editor_ui`, `file_menu`, `window_menu`, `handle_delete_key`, `language_control`, `project_dialog`, `save_as`: top and bottom bars, dock area, menus, Delete key, language picker |
| `src/editor/panel_viewer.rs` | 102 | 800 | bin | `PanelViewer`: the `TabViewer` that draws each dock tab |
| `src/panels/mod.rs` | 32 | 250 | bin | Shared `icon_button` |
| `src/panels/dock/mod.rs` | 163 | 1,400 | bin | `PanelTab`, `PanelDock`: panel arrangement, default layout, saving and restoring it |
| `src/panels/tools/mod.rs` | 100 | 800 | bin | `Tool`, painted tool icons, Tools panel |
| `src/panels/layout/mod.rs` | 127 | 900 | bin | Layout panel: piece tree, lock and delete buttons |
| `src/panels/history/mod.rs` | 39 | 300 | bin | History panel: list of actions, click to jump |
| `src/panels/material/mod.rs` | 241 | 2,100 | bin | Material form, Material panel, new-project dialog |
| `src/preview/mod.rs` | 247 | 2,200 | bin | 3D scene, camera, mesh extrusion |
| `src/model.rs` | 335 | 2,400 | lib | `Project`, `PatternPiece`, `Point`, `Units`, validation |
| `src/material.rs` | 462 | 2,800 | lib | `Material` and the scales of its properties |
| `src/events.rs` | 365 | 2,900 | lib | `ProjectEvent`, `ProjectChange`, `Project::apply` |
| `src/i18n/uk.rs` | 382 | 4,500 | lib | Ukrainian catalog, `plural` helper |
| `src/i18n/mod.rs` | 225 | 1,500 | lib | `Language`, `Label`, `Message` |
| `src/exports/mod.rs` | 105 | 700 | lib | `ExportError`, shared helpers, tests |
| `src/exports/svg.rs` | 34 | 300 | lib | `to_svg` |
| `src/exports/dxf.rs` | 24 | 200 | lib | `to_dxf` |
| `src/shapes.rs` | 107 | 900 | lib | `rectangle`, `circle` |
| `src/i18n/en.rs` | 351 | 3,500 | lib | English catalog |
| `src/history.rs` | 445 | 3,600 | lib | `History`, `HistoryCommand`: snapshots for undo and redo |
| `src/persistence.rs` | 175 | 1,400 | lib | `save_project`, `load_project`, `write_export` |
| `src/settings.rs` | 71 | 500 | bin | Saved language and panel layout in the OS config directory |
| `src/document.rs` | 64 | 500 | bin | `ActiveProject` (project and history), `ProjectMessage`, `HistoryMessage`, `apply_project_events` |
| `src/main.rs` | 36 | 250 | bin | App wiring and system schedule |
| `src/lib.rs` | 6 | 25 | lib | Module declarations |

## Library crate (core, no Bevy)

| File | Lines | Contents |
|---|---:|---|
| `material.rs` | 3-4 | `MIN_THICKNESS_MM`, `MAX_THICKNESS_MM` |
| | 8-104 | `Origin`, `Treatment`, `ThicknessCategory` (`from_millimeters`) |
| | 106-255 | `ColorGroup`, `LeatherColor` (`group`, `rgb`) |
| | 258-380 | `Texture`, `Strength`, `Flexibility`, `WaterResistance`, `HeatResistance` |
| | 383-420 | `Material`: `thickness_category`, `has_valid_thickness` |
| | 421-462 | Tests |
| `model.rs` | 7-11 | `PROJECT_VERSION` and validation limits (`MAX_PIECES`, `MAX_POINTS_PER_PIECE`, `MAX_COORDINATE_MM`, `MIN_POLYGON_AREA_MM2`) |
| | 15-59 | `Units`, `Point`, `Bounds` |
| | 50-150 | `Bounds::overlaps`; ring helpers: `ring_bounds`, `ring_contains`, `segments_touch`, `rings_cross`, `signed_area` |
| | 150-290 | `PatternPiece`: `bounds`, `contains`, `hole_at`, `signed_area`, `rings`, `vertices`, `hole_indices`, `cut`, `outline_without`, `validate_ring`, `validate_holes` |
| | 133-254 | `Project`: `bounds`, `validate` (173), `next_piece_id`, `add_piece`, `remove_piece` |
| | 256-335 | Tests |
| `events.rs` | 9-31 | `ProjectEvent`, `ProjectChange` |
| | 35-91 | `Project::apply` |
| | 93-121 | `edit` (validates a copy), `piece_mut`, `unlocked_piece_mut` |
| | 123-365 | Tests |
| `history.rs` | 6-46 | `MAX_HISTORY`, `HistoryCommand`, `MergeKey`, `Entry`, `History` |
| | 48-205 | `new`, `record`, `apply`, `actions`, `position`, `can_undo`, `can_redo`, `describe` |
| | 207-445 | Tests |
| `persistence.rs` | 7-34 | `save_project`, `load_project`, `write_export` |
| | 36-175 | Tests |
| `exports/mod.rs` | 13-36 | `ExportError`, conversion to `Message` |
| | 38-60 | `validate_export`, `number` |
| | 61-105 | Tests |
| `exports/svg.rs` | 4 | `to_svg` |
| `exports/dxf.rs` | 4 | `to_dxf` |
| `shapes.rs` | 1-39 | `CIRCLE_SEGMENTS`, `rectangle`, `circle` |
| | 41-107 | Tests |
| `i18n/mod.rs` | 14-66 | `Language`: `tag`, `from_tag`, `native_name`, `label`, `message`, `unit_symbol` |
| | 70-140 | `Label`, with nested variants for material values |
| | 144-171 | `Message` |
| | 181-225 | Tests |

## Binary crate (desktop)

### `editor/mod.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 27-32 | Types | `ViewMode` |
| 34-36 | Events | `send` |
| 38-70 | Menu bar helpers | `export` |
| 72-156 | Sidebar | `view_controls`, `units_control`, `selected_piece_details` |
| 158-164 | Formatting | `format_length` |

### `editor/editor_state_struct.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 12-33 | Type | `EditorState` |
| 35-57 | Defaults | `Default` impl |
| 59-152 | Methods | `new`, `report`, `report_error`, `snap`, `snap_point`, `reset_document`, `on_project_change`, `clear_released_drag`, `displayed` |
| 154-159 | Helpers | `document_name` |

### `editor/canvas.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 13-17 | Constants | Zoom limits, hit radii, edge label limit |
| 19-34 | Viewport | `Viewport` (`to_screen`, `to_world`) |
| 36-107 | Canvas | `canvas`, `fit_view` |
| 109-136 | Select tool | `handle_select` |
| 138-177 | Draw tool | `handle_draw`, `finish_draft` |
| 179-223 | Rectangle and Circle tools | `handle_shape`, `add_piece` |
| Search by name | Holes | `select_at`, `commit_shape`, `hole_vertex_at` |
| Search by name | Hit testing | `piece_at`, `vertex_at` |
| Search by name | Painting | `draw_pieces`, `draw_edge_labels`, `fill_polygon`, `draw_draft`, `draw_shape_draft` |

### `editor/ui.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 18-119 | Top-level layout | `editor_ui`, which also shows the dock area and the new-project dialog |
| Search by name | Menus | `file_menu`, `edit_menu`, `window_menu` |
| Search by name | Input | `handle_delete_key`, `handle_history_keys` and the `UNDO`/`REDO` shortcuts |
| 192-206 | Language | `language_control` |
| 208-219 | File dialog | `project_dialog`, `save_as` |

### `panels/`

| File | Lines | Symbols |
|---|---:|---|
| `mod.rs` | 8 | `icon_button` |
| `dock/mod.rs` | 11-32 | `PanelTab`: `CLOSEABLE`, `label` |
| | 35-121 | `PanelDock`: `load`, `is_open`, `toggle`, `reset`, `set_language`, `save_on_release` |
| | 123-163 | `window_id`, `is_usable`, `restore_windows`, `default_state`, `translations` |
| `tools/mod.rs` | 7-9 | Icon constants: `ICON_BUTTON_SIZE`, `ICON_MARGIN`, `ICON_VERTEX_RADIUS` |
| | 11-81 | `Tool`: `ALL`, `label`, `hint`, `shape_points`, `paint_icon` |
| | 83 | `show_panel` |
| `layout/mod.rs` | 7-8 | Icon constants: `ICON_BUTTON_SIZE`, `ICON_MARGIN` |
| | 10-47 | `paint_lock`, `paint_trash` |
| | 48 | `piece_row` |
| | 99 | `show_panel`, which returns the `ProjectEvent` to send |
| `history/mod.rs` | 6 | `show_panel`, which returns the `HistoryCommand` to send |
| `material/mod.rs` | 15-103 | `DialogResult`, `combo`, `swatch`, `color_combo`, `thickness_row` |
| | 105 | `material_fields`, shared by the dialog and the Material panel |
| | 189 | `show_panel`, which returns the `ProjectEvent` to send |
| | 214 | `new_project_dialog` |

### `preview/mod.rs`

| Lines | Symbols |
|---:|---|
| 11-46 | Scene constants, `PreviewPlugin`, `PreviewSettings` |
| 48-70 | `setup_scene` |
| 72-118 | `rebuild_preview` |
| 120-137 | `update_camera` |
| 139-152 | `show_controls` |
| 154-247 | `build_piece_mesh` |

## Where to look for common tasks

| Task | Read | Change |
|---|---|---|
| Add a project edit | `events.rs` 9-121, `history.rs` `describe` | Add a `ProjectEvent` variant and handle it in `apply`, give it an entry label in `History::describe` (an `Action…` `Message`), then send it from `editor/mod.rs` with `send` |
| Add a validation rule | `model.rs` 7-11, 173-234 | `validate`, plus a `Message` variant and both catalogs |
| Add UI text | `i18n/mod.rs` 70-166 | A `Label` or `Message` variant and an arm in `en.rs` and `uk.rs`; the compiler flags missing arms |
| Add a language | `i18n/mod.rs` 14-66 | `Language`, `ALL`, `tag`, `from_tag`, `native_name`, and a new catalog file |
| Add a canvas tool | `panels/tools/mod.rs`, `editor/canvas.rs` 36-91, 179-223 | `Tool` and its `ALL`, `label`, `hint`, `paint_icon`, a dispatch arm in `canvas`; a predefined shape also goes in `shapes.rs` |
| Add or change a material value | `material.rs`, `i18n/en.rs`, `i18n/uk.rs` | The enum and its `ALL`, plus catalog arms; a new property also goes in `Material` and `material_fields` |
| Change the file format | `model.rs` 133-146, `persistence.rs` 7-30 | Bump `PROJECT_VERSION` and decide how older files are handled |
| Change SVG or DXF output | `exports/svg.rs`, `exports/dxf.rs` | `to_svg` or `to_dxf` |
| Change the 3D look or mesh | `preview/mod.rs` 48-118, 154-247 | `setup_scene`, `rebuild_preview`, `build_piece_mesh` |
| Add a dockable panel | `panels/dock/mod.rs`, `editor/panel_viewer.rs` | A `PanelTab` variant, its `CLOSEABLE` and `label` entries, `default_state`, and arms in `PanelViewer::ui` and `scroll_bars` |
| Change system scheduling | `main.rs`, `document.rs` | Keep the ordering rule in `CLAUDE.md` |

## Tests

There are 58 tests, all inline in the library crate:

| Module | Tests |
|---|---:|
| `model` | 14 |
| `material` | 3 |
| `events` | 13 |
| `history` | 9 |
| `i18n` | 5 |
| `exports` | 5 |
| `persistence` | 5 |
| `shapes` | 4 |

`document`, `editor`, `preview`, `settings` and `panels` have no tests, so changes there need `cargo check` and a manual run.

## Docs

| File | Status |
|---|---|
| `CLAUDE.md` | Current. Loaded automatically in every session, so keep it short. |
| `docs/plan.md` | Historical MVP plan. It still says the repo contains only a license. Its todo 6 asks for build and run docs, and there is no README; `CLAUDE.md` only lists the commands. |
| `docs/plans/localization.md` | Historical plan for finished work (~1,700 tokens). It refers to `src/preview.rs`, which is now `src/preview/mod.rs`. Read the "Decisions" and "Out of scope" sections only if you need the reasoning. |
| `docs/plans/add-tools-panel-where-harmonic-nautilus.md` | Historical plan for the Tools panel. |
| `docs/plans/render-icons-instend-text-peppy-hejlsberg.md` | Historical plan for the Tools panel icons. |

## Refreshing this index

To regenerate the symbol list and line numbers:

```sh
rg -n "^\s*(pub(\(\w+\))? )?(fn|struct|enum|trait|impl|mod|const) " src
```

Update it when a file is added, moved or split, or when `editor/mod.rs` or `editor/canvas.rs` is restructured. Small line drift doesn't need a refresh.
