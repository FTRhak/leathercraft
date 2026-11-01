# Tools panel with Draw, Rectangle and Circle

## Context

Tool selection is currently a row of buttons inside the left sidebar (`tool_controls` in `src/editor.rs`), with only Select and Draw. The goal is a dedicated Tools panel, implemented in its own `tools` module, that also offers Rectangle and Circle drawing.

## Decisions

- **Select stays in the panel, as its first entry.** It is the only way to move or edit pieces, so the panel has Select, Draw, Rectangle and Circle.
- **Layout:** a narrow, resizable left panel outside the sidebar, shown only in 2D editor mode. Buttons are stacked vertically.
- **Rectangle:** press and drag from one corner to the opposite corner, axis-aligned.
- **Circle:** press at the center and drag out to set the radius.
- **After a shape is added** the tool returns to Select and the piece is selected, as Draw already does through `on_project_change`.
- **No file format change.** A piece is a closed polygon, so a circle is stored as a 72-point polygon. It has no special circle behaviour afterwards: dragging one vertex moves only that vertex. 72 is above `MAX_POINTS_WITH_EDGE_LABELS` (64), so its edges don't get length labels.

## Changes

### `src/shapes.rs` (new, library)

Pure geometry, tested without Bevy, added to `src/lib.rs` as `pub mod shapes;`:

- `rectangle(a: Point, b: Point) -> Option<Vec<Point>>`: four corners from the min/max of `a` and `b`. `None` if the width or height is zero.
- `circle(center: Point, edge: Point) -> Option<Vec<Point>>`: `CIRCLE_SEGMENTS` (72) evenly spaced points at radius `center`-to-`edge`. `None` if the radius is zero.

Inline tests:

- Rectangle corners are the same for any drag direction.
- Every circle point is at the radius.
- Degenerate input returns `None`.
- Both results pass `Project::validate` when added as a piece.

### `src/panels/tools/mod.rs` (new, binary; `mod panels;` in `src/main.rs`)

Mirrors `preview::show_controls`, taking only what it needs and not `EditorState`:

- `Tool` enum, moved from `editor.rs` and made `pub`, with `Select`, `Draw`, `Rectangle`, `Circle`, plus `ALL`, `label()` and `hint()` returning a `Label`.
- `pub fn show_panel(ui, tool: &mut Tool, language)`: the "Tools" heading and a full-width `selectable_value` for each tool.

### `src/editor.rs`

- **`editor_ui`:**
  - In 2D mode, show `egui::SidePanel::left("tools")` calling `tools::show_panel`, before the sidebar so it sits on the outer edge.
  - Rename the existing panel's ID from `"tools"` to `"sidebar"`, to avoid a clash.
- **`tool_controls`:** rename to `view_controls` and drop the tool buttons. It keeps the active tool's hint (`state.tool.hint()`), which is too long for the narrow panel, plus the navigation hint, snap and zoom to fit.
- **`EditorState`:** add `shape_draft: Option<ShapeDraft>` (`start`, `end`, both snapped), cleared in `reset_document`. It holds in-progress state, per the architecture rule.
- **`canvas` dispatch:** add a `Tool::Rectangle | Tool::Circle` arm calling `handle_shape`. Replace the `draft.clear()` in the Select arm with one that runs for every tool except Draw, so that switching to a shape tool can't leave a polygon draft on screen.
- **`handle_shape`:**
  - `drag_started_by(Primary)`: start the draft at the snapped `press_origin`, as `handle_select` does.
  - `dragged_by(Primary)`: update `end`.
  - `drag_stopped_by(Primary)`: take the draft, build the points, and add the piece. A `None` from `shapes` is ignored silently, so a zero-size drag does nothing.
  - Esc cancels, behind the same `wants_keyboard_input` guard as `handle_draw`.
- **Adding a piece:** extract `finish_draft`'s naming and send into `add_piece(project, events, language, points)` and use it from both tools. Everything still goes through `ProjectEvent::AddPiece`, so `Project::apply` still validates it, and no new event is needed.
- **`draw_shape_draft`:** a closed outline in the draft color and a size label next to the cursor end: `W × H` for a rectangle, `Ø D` for a circle, formatted with `format_length`.

### `src/i18n/` (`mod.rs`, `en.rs`, `uk.rs`)

Replace `Label::Tool` with `Label::Tools`, and add the rest:

| Label | English | Ukrainian |
|---|---|---|
| `Tools` | Tools | Інструменти |
| `Rectangle` | Rectangle | Прямокутник |
| `Circle` | Circle | Коло |
| `RectangleHint` | Drag from corner to corner. Esc cancels. | Перетягуйте від кута до кута. Esc скасовує. |
| `CircleHint` | Drag from the center outward. Esc cancels. | Перетягуйте від центру назовні. Esc скасовує. |

### Docs

- `CLAUDE.md`: add `shapes` to the library list and `tools` to the binary modules.
- `docs/project-index.md`: add the new files, update the sizes, `editor.rs` ranges, test counts and the "Add a canvas tool" row, and move the indexed commit forward.

## Verification

```sh
cargo fmt
cargo test --lib --no-default-features -q
cargo check --bin leathercraft --message-format=short
cargo clippy
```

Then a manual pass with `cargo run`, because the binary has no tests:

- The Tools panel appears in 2D mode and is hidden in 3D preview.
- Draw still works as before.
- Rectangle and Circle drag in all four directions, with a live outline and size label, and Esc cancels mid-drag.
- A click with no drag adds nothing.
- Shapes respect snap, appear in the Pieces list and 3D preview, and survive save and load and SVG/DXF export.
- Labels are correct in both languages.
