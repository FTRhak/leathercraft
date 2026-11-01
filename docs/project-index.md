# Project index

A map for finding code without reading whole files. It complements `CLAUDE.md`, which holds the architecture rules and invariants and isn't repeated here.

Indexed at commit `5afa155` plus the Tools and Layout panel and material changes. Line numbers drift, so treat them as hints and search for the symbol name if one doesn't match.

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
| `src/editor/drag_state_struct.rs` | 12 | 100 | bin | `DragState` |
| `src/editor/shape_draft_struct.rs` | 7 | 50 | bin | `ShapeDraft` |
| `src/editor/canvas.rs` | 373 | 3,000 | bin | `canvas`, `Viewport`, tool behavior, hit testing, painting |
| `src/editor/confirm_discard.rs` | 14 | 100 | bin | `confirm_discard`: unsaved-changes prompt |
| `src/editor/drag.rs` | 80 | 600 | bin | `begin_drag`, `update_drag`, `finish_drag`: dragging pieces and vertices with the Select tool |
| `src/editor/grid.rs` | 33 | 300 | bin | `draw_grid`: canvas grid and axes, grid steps |
| `src/editor/ui.rs` | 245 | 2,200 | bin | `editor_ui`, `file_menu`, `window_menu`, `handle_delete_key`, `language_control`, `project_dialog`, `save_as`: top-level panel layout, menus, Delete key, language picker |
| `src/panels/mod.rs` | 31 | 250 | bin | Shared `icon_button` |
| `src/panels/tools/mod.rs` | 100 | 800 | bin | `Tool`, painted tool icons, Tools panel |
| `src/panels/layout/mod.rs` | 127 | 900 | bin | Layout panel: piece tree, lock and delete buttons |
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
| `src/persistence.rs` | 175 | 1,400 | lib | `save_project`, `load_project`, `write_export` |
| `src/settings.rs` | 89 | 650 | bin | Saved language and `PanelVisibility` in the OS config directory |
| `src/document.rs` | 32 | 300 | bin | `ActiveProject`, `ProjectMessage`, `apply_project_events` |
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
| | 61-130 | `PatternPiece`: `bounds`, `contains`, `signed_area` |
| | 133-254 | `Project`: `bounds`, `validate` (173), `next_piece_id`, `add_piece`, `remove_piece` |
| | 256-335 | Tests |
| `events.rs` | 9-31 | `ProjectEvent`, `ProjectChange` |
| | 35-91 | `Project::apply` |
| | 93-121 | `edit` (validates a copy), `piece_mut`, `unlocked_piece_mut` |
| | 123-365 | Tests |
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
| 12-16 | Constants | Zoom limits, hit radii, edge label limit |
| 18-33 | Viewport | `Viewport` (`to_screen`, `to_world`) |
| 35-106 | Canvas | `canvas`, `fit_view` |
| 108-135 | Select tool | `handle_select` |
| 137-176 | Draw tool | `handle_draw`, `finish_draft` |
| 178-222 | Rectangle and Circle tools | `handle_shape`, `add_piece` |
| 224-242 | Hit testing | `piece_at`, `vertex_at` |
| 244-373 | Painting | `draw_pieces`, `draw_draft`, `draw_shape_draft` |

### `editor/ui.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 18-151 | Top-level layout | `editor_ui`, which also shows the new-project dialog and applies panel visibility |
| 153-200 | Menus | `file_menu`, `window_menu` |
| 202-217 | Input | `handle_delete_key` |
| 219-232 | Language | `language_control` |
| 234-245 | File dialog | `project_dialog`, `save_as` |

### `panels/`

| File | Lines | Symbols |
|---|---:|---|
| `mod.rs` | 7 | `icon_button` |
| `tools/mod.rs` | 7-9 | Icon constants: `ICON_BUTTON_SIZE`, `ICON_MARGIN`, `ICON_VERTEX_RADIUS` |
| | 11-81 | `Tool`: `ALL`, `label`, `hint`, `shape_points`, `paint_icon` |
| | 83 | `show_panel` |
| `layout/mod.rs` | 7-8 | Icon constants: `ICON_BUTTON_SIZE`, `ICON_MARGIN` |
| | 10-47 | `paint_lock`, `paint_trash` |
| | 48 | `piece_row` |
| | 99 | `show_panel`, which returns the `ProjectEvent` to send |
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
| Add a project edit | `events.rs` 9-121 | Add a `ProjectEvent` variant and handle it in `apply`, then send it from `editor/mod.rs` with `send` |
| Add a validation rule | `model.rs` 7-11, 173-234 | `validate`, plus a `Message` variant and both catalogs |
| Add UI text | `i18n/mod.rs` 70-166 | A `Label` or `Message` variant and an arm in `en.rs` and `uk.rs`; the compiler flags missing arms |
| Add a language | `i18n/mod.rs` 14-66 | `Language`, `ALL`, `tag`, `from_tag`, `native_name`, and a new catalog file |
| Add a canvas tool | `panels/tools/mod.rs`, `editor/canvas.rs` 35-90, 178-222 | `Tool` and its `ALL`, `label`, `hint`, `paint_icon`, a dispatch arm in `canvas`; a predefined shape also goes in `shapes.rs` |
| Add or change a material value | `material.rs`, `i18n/en.rs`, `i18n/uk.rs` | The enum and its `ALL`, plus catalog arms; a new property also goes in `Material` and `material_fields` |
| Change the file format | `model.rs` 133-146, `persistence.rs` 7-30 | Bump `PROJECT_VERSION` and decide how older files are handled |
| Change SVG or DXF output | `exports/svg.rs`, `exports/dxf.rs` | `to_svg` or `to_dxf` |
| Change the 3D look or mesh | `preview/mod.rs` 48-118, 154-247 | `setup_scene`, `rebuild_preview`, `build_piece_mesh` |
| Change system scheduling | `main.rs`, `document.rs` | Keep the ordering rule in `CLAUDE.md` |

## Tests

There are 31 tests, all inline in the library crate:

| Module | Tests |
|---|---:|
| `model` | 4 |
| `material` | 3 |
| `events` | 8 |
| `i18n` | 5 |
| `exports` | 3 |
| `persistence` | 4 |
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
