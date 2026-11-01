# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Desktop CAD-style leather pattern editor in Rust (edition 2024), built on Bevy 0.18 and `bevy_egui`. `docs/plan.md` holds the MVP plan and scope.

Project structure `docs/project-index.md` should be updated after any changes

## Commands

```sh
cargo run                                   # desktop app (default `desktop` feature)
cargo check --bin leathercraft              # type-check the desktop binary
cargo test --lib --no-default-features      # core tests, without compiling Bevy
cargo test --lib --no-default-features project_save_and_load_round_trip   # single test by name
cargo fmt
cargo clippy
```

All existing tests are inline `#[cfg(test)]` modules in the library crate, so `--no-default-features` covers them.

## Architecture

The package has two crates with a deliberate boundary: geometry and file-format logic stay free of UI and rendering so they can be tested without launching the app.

- **Library (`src/lib.rs`), always built, depends only on serde:**
  - `model`: `Project`, `PatternPiece`, `Point`, `Units`, and `Project::validate`.
  - `material`: `Material` and the scales of its properties. Each piece has its own material; `Project.default_material`, chosen in the new-project dialog, is copied into each new piece.
  - `events`: `ProjectEvent`, `ProjectChange`, and `Project::apply`, the single entry point for changing a project.
  - `persistence`: `.leathercraft` save/load as pretty-printed, versioned JSON.
  - `exports`: `to_svg` and `to_dxf` return strings; `persistence::write_export` writes them to disk.
  - `shapes`: `rectangle` and `circle` build the point lists of predefined shapes.
  - `i18n`: `Language`, `Label` (fixed text) and `Message` (text with arguments), with one exhaustive catalog per language, so a missing translation is a compile error.
- **Binary (`src/main.rs`), requires the `desktop` feature:** `document`, `editor`, `preview`, `settings` and `tools` are modules of the binary, not the library, and import the core via `leathercraft::model`. Code that needs Bevy, egui, `earcutr`, `rfd`, `dirs`, or `sys-locale` belongs here.

All document data lives in `Project`; `EditorState`, `ViewMode` and `PreviewSettings` hold interface state only. The interface language is interface state, stored in `EditorState` and saved by `settings`, never in `Project`. Fields that are not part of the file format, such as `path` and `modified`, are `#[serde(skip)]`.

The UI never writes to the project. It sends a `ProjectEvent` wrapped in `document::ProjectMessage`, and `document::apply_project_events` calls `Project::apply` and tells `EditorState` the outcome. `document::ActiveProject` exposes the project read-only, so this system is its only writer. To add an edit, add a `ProjectEvent` variant and handle it in `Project::apply`. In-progress work (drawing, dragging, a name being typed) stays in `EditorState` until it is committed by an event.

`preview::rebuild_preview` rebuilds on the change detection of `ActiveProject`, which `apply_project_events` triggers only for events that changed the content. `editor::editor_ui` must stay in the `EguiPrimaryContextPass` schedule, not `Update`, chained before `apply_project_events` so events apply in the frame they are sent.

## Invariants

- **Internal geometry is always millimeters.** `Project.units` is a display preference only; convert explicitly with `Units::from_millimeters`. SVG and DXF exports are always written in mm, regardless of `Project.units`.
- **A piece is a closed polygon** stored as an ordered point list without a repeated closing point. Validation treats the last point as connected to the first.
- **`Project::validate` guards every file boundary:** `save_project`, `load_project`, `to_svg`, and `to_dxf` all call it. New I/O paths should do the same.
- **The project is always valid:** `Project::apply` validates every edit on a copy, so a rejected event leaves the project untouched.
- **Format version:** `PROJECT_VERSION` must match exactly on load; there is no migration path yet. Changing the serialized shape of the model means bumping it and deciding how older files are handled. Adding a `#[serde(default)]` field doesn't need a bump, since older files still load.
- **Round-tripping uses `.leathercraft` only.** SVG and DXF are one-way exports; SVG also shifts geometry so the project's bounding box starts at the origin, while DXF keeps original coordinates.
- **The 3D preview shows individual flat pieces**, extruded to the thickness of their material and shown in its color. It intentionally does not infer seams or assemble finished items.

## Conventions

Library functions in `model`, `events` and `persistence` return `Result<_, i18n::Message>`, which the UI renders in the current language; `exports` uses the `ExportError` enum, which converts into a `Message`. New user-facing text goes through `i18n`; log messages stay in English.
