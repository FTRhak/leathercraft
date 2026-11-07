# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Desktop CAD-style leather pattern editor in Rust (edition 2024), built on Bevy 0.18 and `bevy_egui`. `docs/plan.md` holds the MVP plan and scope.

Project structure `docs/project-index.md` should be updated after any changes

## Design

https://brilliant.design/neat-pansy/leathercraft-design/main

## Commands

```sh
cargo run                                   # desktop app (default `desktop` + `dynamic_linking` features)
cargo release                               # static release build to ship (alias in .cargo/config.toml)
cargo check --bin leathercraft              # type-check the desktop binary
cargo test --lib --no-default-features      # core tests, without compiling Bevy
cargo test --lib --no-default-features project_save_and_load_round_trip   # single test by name
cargo fmt
cargo clippy
```

In dev builds Bevy is linked dynamically (`bevy_dylib` DLL plus Rust's `std` DLL), so the debug exe only runs through `cargo run`, which puts those DLLs on `PATH`. Builds to ship must use `cargo release`.

All existing tests are inline `#[cfg(test)]` modules in the library crate, so `--no-default-features` covers them.

## Architecture

The package has two crates with a deliberate boundary: geometry and file-format logic stay free of UI and rendering so they can be tested without launching the app.

- **Library (`src/lib.rs`), always built, depends only on serde and `i_overlay` (polygon subtraction):**
  - `model`: `Project`, `PatternPiece`, `ChildShape`, `BooleanOp`, `Point`, `Units`, and `Project::validate`.
  - `boolean`: `PatternPiece::region` combines a piece's shapes into a `Region` (outline and holes) with `i_overlay`.
  - `material`: `Material` and the scales of its properties. Each piece has its own material; `Project.default_material`, chosen in the new-project dialog, is copied into each new piece.
  - `events`: `ProjectEvent`, `ProjectChange`, and `Project::apply`, the single entry point for changing a project.
  - `persistence`: `.leathercraft` save/load as pretty-printed, versioned JSON.
  - `exports`: `to_svg` and `to_dxf` return strings; `persistence::write_export` writes them to disk.
  - `shapes`: `Shape` (polygon, or parametric rectangle, ellipse and regular polygon of 3 to 64 sides, with a rotation, or an open `Line` with a solid, dashed or dash-dot `LineStyle` in mm, used as a centerline): its `ring`, `handles` and `with_handle_at` for reshaping. Polygons, rectangles and regular polygons keep a radius per vertex (`radii`, `with_corner_radius`); `ring` replaces each rounded corner with an arc, while `corners` and `handles` stay on the sharp vertices.
  - `align`: `Arrange`, `AlignTo`, `Spacing`, `offsets` and `child_offsets`, the Illustrator-style align and distribute rules behind `ProjectEvent::ArrangePieces` and `ProjectEvent::ArrangeChildren`. Pieces are measured by their outline, or by all their shapes while the result is invalid; children by their own shape. Locked pieces and the key object stay put. `AlignTo::Parent` lines children up with their piece's base shape, and doesn't apply to pieces.
  - `snap`: `SnapSettings`, `SnapTargets`, `snap_point` and `snap_offset`: snapping to vertices, then smart guides (box edges and centers), then a grid around the origin. The editor picks the targets; a child snaps only within its own piece (`docs/snap.md`).
  - `history`: `History` keeps a `Project` snapshot after each accepted edit, for undo, redo and the History panel. Consecutive renames or material changes of the same piece share one entry, and `New`/`Open` start a new history.
  - `i18n`: `Language`, `Label` (fixed text) and `Message` (text with arguments), with one exhaustive catalog per language, so a missing translation is a compile error.
- **Binary (`src/main.rs`), requires the `desktop` feature:** `document`, `editor`, `panels`, `preview`, `settings` and `ui_components` are modules of the binary, not the library, and import the core via `leathercraft::model`. Code that needs Bevy, egui, `egui_dock`, `earcutr`, `rfd`, `dirs`, or `sys-locale` belongs here.

All document data lives in `Project`; `EditorState`, `ViewMode` and `PreviewSettings` hold interface state only. The interface language is interface state, stored in `EditorState` and saved by `settings`, never in `Project`; so is the panel arrangement, stored in `ui_components::dock::PanelDock`, and so are `settings::Preferences` (the default material the new-project dialog starts from, the hotkeys, and the dark or light theme), a resource edited in the Settings window. All of these share one `settings.json` in the OS config directory, each section parsed on its own so a broken one falls back to its default without losing the others. Keyboard commands go through `settings::Action` and `Hotkeys::triggered`, not hard-coded keys. Fields that are not part of the file format, such as `path` and `modified`, are `#[serde(skip)]`.

The UI never writes to the project. It sends a `ProjectEvent` wrapped in `document::ProjectMessage`, and `document::apply_project_events` calls `Project::apply` and tells `EditorState` the outcome. `document::ActiveProject` exposes the project read-only, so this system is its only writer. To add an edit, add a `ProjectEvent` variant, handle it in `Project::apply`, and give it an entry label in `History::describe`. `ActiveProject` also owns the `History`; undo, redo and jumps are sent as `document::HistoryMessage` and applied by the same system after the project events. In-progress work (drawing, dragging, a name being typed) stays in `EditorState` until it is committed by an event.

`preview::rebuild_preview` rebuilds on the change detection of `ActiveProject`, which `apply_project_events` triggers only for events that changed the content. `editor::editor_ui` must stay in the `EguiPrimaryContextPass` schedule, not `Update`, chained before `apply_project_events` so events apply in the frame they are sent.

## Invariants

- **Internal geometry is always millimeters.** `Project.units` is a display preference only; convert explicitly with `Units::from_millimeters`. SVG and DXF exports are always written in mm, regardless of `Project.units`.
- **A piece is a base shape plus ordered child shapes.** Each child subtracts from, unites with or intersects the result so far, in list order; the outline is never stored, only computed by `PatternPiece::region`. Open children (`Shape::is_open`, the centerlines) take no part in it and their `op` is ignored; the base is never open. They are drawn unclipped and exported on their own layer (`docs/centerline.md`). Rings are point lists without a repeated closing point. Children have ids unique within their piece, so selection survives reordering.
- **Validation checks shapes, not their result.** `Project::validate` checks each shape's parameters and ring, but a piece whose result is empty or falls apart (`ResultEmpty`, `ResultSplit`) is still a valid project, so a child can be dragged through such states. The editor flags these pieces, the 3D preview skips them, and SVG and DXF export refuse them.
- **`Project::validate` guards every file boundary:** `save_project`, `load_project`, `to_svg`, and `to_dxf` all call it. New I/O paths should do the same.
- **The project is always valid:** `Project::apply` validates every edit on a copy, so a rejected event leaves the project untouched.
- **Format version:** `PROJECT_VERSION` must match exactly on load; there is no migration path yet. Changing the serialized shape of the model means bumping it and deciding how older files are handled. Adding a `#[serde(default)]` field doesn't need a bump, since older files still load.
- **Round-tripping uses `.leathercraft` only.** SVG and DXF are one-way exports; SVG also shifts geometry so the project's bounding box starts at the origin, while DXF keeps original coordinates.
- **The 3D preview shows individual flat pieces**, extruded from their region to the thickness of their material, with their holes cut through, and shown in its color. It intentionally does not infer seams or assemble finished items.

## Conventions

Library functions in `model`, `events` and `persistence` return `Result<_, i18n::Message>`, which the UI renders in the current language; `exports` uses the `ExportError` enum, which converts into a `Message`. New user-facing text goes through `i18n`; log messages stay in English.

Every color comes from `ui_components::palette::Palette`, with a value for both themes; add a new color there instead of writing a literal. See `docs/design.md` for this and for writing UI components.
