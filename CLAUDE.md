# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Desktop CAD-style leather pattern editor in Rust (edition 2024), built on Bevy 0.18 and `bevy_egui`. `docs/plan.md` holds the MVP plan and scope.

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
  - `persistence`: `.leathercraft` save/load as pretty-printed, versioned JSON.
  - `exports`: `to_svg` and `to_dxf` return strings; `persistence::write_export` writes them to disk.
- **Binary (`src/main.rs`), requires the `desktop` feature:** `editor` and `preview` are modules of the binary, not the library, and import the core via `leathercraft::model`. Code that needs Bevy, egui, `earcutr`, or `rfd` belongs here.

The library must not depend on Bevy, so the binary holds the project in the `editor::ActiveProject` resource wrapper. `preview::rebuild_preview` rebuilds on its change detection: mutating through `ResMut<ActiveProject>` every frame rebuilds every mesh every frame, so write only on an actual change. `editor::editor_ui` must stay in the `EguiPrimaryContextPass` schedule, not `Update`.

## Invariants

- **Internal geometry is always millimeters.** `Project.units` is a display preference only; convert explicitly with `Units::from_millimeters`. SVG and DXF exports are always written in mm, regardless of `Project.units`.
- **A piece is a closed polygon** stored as an ordered point list without a repeated closing point. Validation treats the last point as connected to the first.
- **`Project::validate` guards every file boundary:** `save_project`, `load_project`, `to_svg`, and `to_dxf` all call it. New I/O paths should do the same.
- **Format version:** `PROJECT_VERSION` must match exactly on load; there is no migration path yet. Changing the serialized shape of the model means bumping it and deciding how older files are handled.
- **Round-tripping uses `.leathercraft` only.** SVG and DXF are one-way exports; SVG also shifts geometry so the project's bounding box starts at the origin, while DXF keeps original coordinates.
- **The 3D preview shows individual flat pieces**, extruded with a leather-like material. It intentionally does not infer seams or assemble finished items.

## Conventions

Library functions in `model` and `persistence` return `Result<_, String>` with user-facing messages; `exports` uses the `ExportError` enum.
