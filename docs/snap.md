# Snapping (`snap`, `panels::snap`)

How points and moved shapes snap while drawing and dragging, and where the settings live.

| File | Holds |
|---|---|
| `src/snap.rs` (lib) | `SnapSettings`, `SnapScope`, `SnapTargets`, `Moving`, `Guide`, `SnapFeedback`, `snap_point`, `snap_offset`; tests |
| `src/panels/snap/mod.rs` | The Snap panel: `show_panel` |
| `src/editor/canvas.rs` | `draw_targets`, `snap_draw` (drawing tools), the hover target, `draw_snap_feedback` |
| `src/editor/drag.rs` | `snap_setup` (targets and moved shapes gathered when a drag starts), snapping in `update_drag` |
| `src/editor/grid.rs` | The dots of the snap grid |
| `src/settings/mod.rs` | `load_snap`, `save_snap`: the `snap` section of `settings.json` |

## For users

The **Snap** panel (beside Properties and Align; reopen it from the Window menu) has:

- **Snapping**: turns everything on or off. **F9** toggles it; holding **Alt** turns it off for a moment.
- **To grid** with a step of 1–10 mm, counted from the origin [0, 0]. While the dots are at least 8 px apart, the canvas shows them.
- **To vertices**: the handles of other shapes: corners, ellipse axis ends, line and stitch line points. A child shape snaps only to the vertices of its own piece.
- **Smart guides**: the left, right, top and bottom edges and the center of what moves line up with those of other objects. A dashed magenta line shows what lined up.
- **Snap distance** in screen pixels (2–30, default 8): how close a vertex or guide has to come. It is the same at every zoom.

A magenta square marks the vertex snapped to. Drawing tools show the snapped point before the click.

## Priority

Each snap tries, in order:

1. **Vertex**: the closest pair of a moved handle and a target vertex within reach. It snaps both axes exactly, and nothing else applies.
2. **Guide**, per axis: the closest line within reach among the min, center and max of every target box, against the same three lines of the moved box. A point is a box of size zero.
3. **Grid**, per axis: rounds to the step. A point is rounded itself. A move rounds its **anchor**, the moved handle nearest where the drag started, so that this vertex lands on the grid while the shape keeps its own form.

So x can follow a guide while y follows the grid. With grid snapping off, an axis without a guide stays where the cursor puts it.

`snap_point` is `snap_offset` with a zero offset on a `Moving::point`.

## What snaps to what

| Action | Scope | Removed from the targets |
|---|---|---|
| A drawing tool making a new piece | `Project` | — |
| A drawing tool adding a child (an operation chosen, or a centerline or stitch line) to the selected piece | `Piece` of the selected piece | — |
| The Draw and Stitch holes tools | as above | — (the draft's own points are added) |
| Dragging pieces | `Project`, excluding the dragged pieces | — |
| Dragging children | `Piece`, excluding the dragged children | — |
| Dragging a handle of the base | `Project` | the handle's own point, the piece's box |
| Dragging a handle of a child | `Piece` | the handle's own point, the child's box |

Target points are `Shape::handles`, not ring points, so the 64 points of an ellipse or the arcs of rounded corners don't attract. Target boxes are `PatternPiece::bounds` in project scope; in piece scope they are the base's bounds and each remaining child's bounds.

A drag gathers its targets once, in `begin_drag` or `begin_handle_drag`, since the project doesn't change until it is committed. Drawing tools gather them each frame.

## State

All of it is interface state, never part of a `Project`:

- `EditorState::snap`: the settings. `snap_saved` holds the last saved copy.
- `snap_suspended`: Alt is held, read at the start of `canvas`. `active_snap` combines it with `snap.enabled`.
- `snap_feedback`: what the last snap of the frame lined up with. It is cleared at the start of `canvas` and drawn last.
- `hover_target`: the snapped point under the cursor for drawing tools. `draw_draft` and `draw_stitch_draft` use it.

The tolerance in mm is `distance_px / zoom` (`Viewport::snap_tolerance`).

## Extending

- **A new kind of target**, such as edge midpoints or shape centers: add the points in `SnapTargets::collect`.
- **A new snap mode**: add a flag to `SnapSettings`, with a default, so that older `settings.json` files still load. Give it a step in `snap_offset` and a checkbox in `panels::snap`.
