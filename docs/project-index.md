# Project index

A map for finding code without reading whole files. It complements `CLAUDE.md`, which holds the architecture rules and invariants and isn't repeated here.

Indexed at commit `5afa155` plus the Tools panel change. Line numbers drift, so treat them as hints and search for the symbol name if one doesn't match.

## How to read this repo cheaply

- **Start with this index, then read a line range.** All of `src/` is about 21k tokens, and `editor.rs` alone is about 7.5k.
- **Never open or sweep these:**
  - `target/` (11 GB, about 7,700 files)
  - `.git/`
  - `Cargo.lock` (165 KB, about 41k tokens, more than twice all of `src/`)
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
| `src/editor.rs` | 945 | 7,500 | bin | egui UI: panels, canvas, tool behavior, drawing |
| `src/panels/tools/mod.rs` | 112 | 900 | bin | `Tool`, painted tool icons, Tools panel |
| `src/preview/mod.rs` | 247 | 2,100 | bin | 3D scene, camera, mesh extrusion |
| `src/model.rs` | 314 | 2,200 | lib | `Project`, `PatternPiece`, `Point`, `Units`, validation |
| `src/events.rs` | 213 | 1,700 | lib | `ProjectEvent`, `ProjectChange`, `Project::apply` |
| `src/i18n/uk.rs` | 111 | 1,400 | lib | Ukrainian catalog, `plural` helper |
| `src/i18n/mod.rs` | 174 | 1,200 | lib | `Language`, `Label`, `Message` |
| `src/exports/mod.rs` | 105 | 700 | lib | `ExportError`, shared helpers, tests |
| `src/exports/svg.rs` | 34 | 300 | lib | `to_svg` |
| `src/exports/dxf.rs` | 24 | 200 | lib | `to_dxf` |
| `src/shapes.rs` | 107 | 900 | lib | `rectangle`, `circle` |
| `src/i18n/en.rs` | 88 | 1,000 | lib | English catalog |
| `src/persistence.rs` | 92 | 700 | lib | `save_project`, `load_project`, `write_export` |
| `src/settings.rs` | 52 | 400 | bin | Saved language in the OS config directory |
| `src/document.rs` | 32 | 300 | bin | `ActiveProject`, `ProjectMessage`, `apply_project_events` |
| `src/main.rs` | 33 | 250 | bin | App wiring and system schedule |
| `src/lib.rs` | 6 | 25 | lib | Module declarations |

## Library crate (core, no Bevy)

| File | Lines | Contents |
|---|---:|---|
| `model.rs` | 6-10 | `PROJECT_VERSION` and validation limits (`MAX_PIECES`, `MAX_POINTS_PER_PIECE`, `MAX_COORDINATE_MM`, `MIN_POLYGON_AREA_MM2`) |
| | 14-58 | `Units`, `Point`, `Bounds` |
| | 60-122 | `PatternPiece`: `bounds`, `contains`, `signed_area` |
| | 124-234 | `Project`: `bounds`, `validate` (160), `next_piece_id`, `add_piece`, `remove_piece` |
| | 236-314 | Tests |
| `events.rs` | 8-28 | `ProjectEvent`, `ProjectChange` |
| | 32-75 | `Project::apply` |
| | 77-94 | `edit` (validates a copy), `piece_mut` |
| | 98-213 | Tests |
| `persistence.rs` | 7-35 | `save_project`, `load_project`, `write_export` |
| | 37-92 | Tests |
| `exports/mod.rs` | 13-36 | `ExportError`, conversion to `Message` |
| | 38-60 | `validate_export`, `number` |
| | 61-105 | Tests |
| `exports/svg.rs` | 4 | `to_svg` |
| `exports/dxf.rs` | 4 | `to_dxf` |
| `shapes.rs` | 1-39 | `CIRCLE_SEGMENTS`, `rectangle`, `circle` |
| | 41-107 | Tests |
| `i18n/mod.rs` | 10-62 | `Language`: `tag`, `from_tag`, `native_name`, `label`, `message`, `unit_symbol` |
| | 66-101 | `Label` |
| | 103-127 | `Message` |
| | 137-174 | Tests |

## Binary crate (desktop)

### `editor.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 14-23 | Constants | Zoom limits, grid steps, hit radii |
| 25-67 | Types | `ViewMode`, `ShapeDraft`, `DragState`, `EditorState` |
| 69-180 | `EditorState` methods | `report`, `report_error`, `snap`, `on_project_change`, `clear_released_drag`, `displayed` |
| 182-204 | Helpers | `document_name`, `Viewport` (`to_screen`, `to_world`) |
| 206-290 | Top-level layout | `editor_ui` |
| 292-402 | File bar | `send`, `file_buttons`, `language_control`, `confirm_discard`, `save_as`, `project_dialog`, `export` |
| 404-504 | Sidebar | `view_controls`, `units_control`, `pieces_list` |
| 506-577 | Canvas | `canvas`, `fit_view` |
| 579-680 | Select tool | `handle_select`, `begin_drag`, `update_drag`, `finish_drag` |
| 682-721 | Draw tool | `handle_draw`, `finish_draft` |
| 723-767 | Rectangle and Circle tools | `handle_shape`, `add_piece` |
| 769-783 | Hit testing | `piece_at`, `vertex_at` |
| 785-937 | Painting | `draw_grid`, `draw_pieces`, `draw_draft`, `draw_shape_draft` |
| 939-945 | Formatting | `format_length` |

### `panels/tools/mod.rs`

| Lines | Symbols |
|---:|---|
| 6-8 | Icon constants: `ICON_BUTTON_SIZE`, `ICON_MARGIN`, `ICON_VERTEX_RADIUS` |
| 10-80 | `Tool`: `ALL`, `label`, `hint`, `shape_points`, `paint_icon` |
| 82-100 | `icon_button` |
| 102-112 | `show_panel` |

### `preview/mod.rs`

| Lines | Symbols |
|---:|---|
| 11-47 | Scene constants, `PreviewPlugin`, `PreviewSettings` |
| 49-71 | `setup_scene` |
| 73-119 | `rebuild_preview` |
| 121-138 | `update_camera` |
| 140-153 | `show_controls` |
| 155-247 | `build_piece_mesh` |

## Where to look for common tasks

| Task | Read | Change |
|---|---|---|
| Add a project edit | `events.rs` 8-94 | Add a `ProjectEvent` variant and handle it in `apply`, then send it from `editor.rs` with `send` |
| Add a validation rule | `model.rs` 6-10, 160-213 | `validate`, plus a `Message` variant and both catalogs |
| Add UI text | `i18n/mod.rs` 66-123 | A `Label` or `Message` variant and an arm in `en.rs` and `uk.rs`; the compiler flags missing arms |
| Add a language | `i18n/mod.rs` 10-62 | `Language`, `ALL`, `tag`, `from_tag`, `native_name`, and a new catalog file |
| Add a canvas tool | `panels/tools/mod.rs`, `editor.rs` 506-561, 723-767 | `Tool` and its `ALL`, `label`, `hint`, `paint_icon`, a dispatch arm in `canvas`; a predefined shape also goes in `shapes.rs` |
| Change the file format | `model.rs` 124-134, `persistence.rs` 7-30 | Bump `PROJECT_VERSION` and decide how older files are handled |
| Change SVG or DXF output | `exports/svg.rs`, `exports/dxf.rs` | `to_svg` or `to_dxf` |
| Change the 3D look or mesh | `preview/mod.rs` 49-119, 155-247 | `setup_scene`, `rebuild_preview`, `build_piece_mesh` |
| Change system scheduling | `main.rs`, `document.rs` | Keep the ordering rule in `CLAUDE.md` |

## Tests

There are 21 tests, all inline in the library crate:

| Module | Tests |
|---|---:|
| `model` | 4 |
| `events` | 4 |
| `i18n` | 4 |
| `exports` | 3 |
| `persistence` | 2 |
| `shapes` | 4 |

`document`, `editor`, `preview`, `settings` and `tools` have no tests, so changes there need `cargo check` and a manual run.

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

Update it when a file is added, moved or split, or when `editor.rs` is restructured. Small line drift doesn't need a refresh.
