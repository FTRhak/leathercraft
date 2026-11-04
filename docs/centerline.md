# Centerlines (`shapes::Line`)

A centerline is an open straight line kept as a child of a piece. It marks the piece, for example its axis or a fold. It is not part of the outline, so it may reach past the piece. A centerline is a `ChildShape` whose shape is `Shape::Line`, so ids, selection, reordering, deleting, moving, aligning, undo and save work for it as for any other child. The geometry lives in the library crate, so it can be tested without Bevy.

| File | Contents |
|---|---|
| `src/shapes/line.rs` | `Line`, `LineStyle`, `MIN_DASH_MM`, `MAX_DASHES_PER_LINE`, `DOT_MM`: `from_ends`, `length`, `angle`, `with_length`, `with_angle`, `handles`, `with_handle_at`, `distance_to`, `dashes`, `dash_count`; tests |
| `src/shapes/mod.rs` | `Shape::Line`, `is_open`, `distance_to`, `line_from_ends`; the `Line` arms of the other methods |
| `src/model.rs` | `validate_shape` (a line's ends and dash count), `BaseMustBeClosed` |
| `src/boolean.rs` | `region` skips open children |
| `src/exports/` | `centerlines`; a `<line>` group in `to_svg`, `LINE` entities in `to_dxf` |
| `src/panels/tools/mod.rs` | `Tool::Centerline`, its icon, `ToolSettings` with the style of new lines |
| `src/editor/canvas.rs` | `commit_shape` (a line is always a child), `shape_hit`, `piece_hit`, `child_at` with a tolerance, `draw_centerlines`, `draw_line`, `draw_line_label` |
| `src/panels/properties/mod.rs` | The `Line` arm of `shape_fields`, `line_style_fields`, `length_field` |
| `src/panels/layout/mod.rs` | Named *Centerline*, with no operation picker |
| `src/settings/hotkeys.rs` | `Action::ToolCenterline`, C by default |

## For users

| Action | How |
|---|---|
| Add a centerline | Select a piece, pick **Centerline** (C), and drag from one end to the other |
| Keep it at an angle step | Hold Shift while dragging: 15° steps |
| Choose its style beforehand | Under the Tools buttons while the tool is active: **Line type**, **Dash**, **Gap** |
| Change it afterwards | Select it, then in Properties: start and end X/Y, **Length**, **Angle**, line type, dash and gap |
| Move or reshape it | Drag the line, or one of its two end handles, with Select |
| Align it | Select it with other children (Shift+click) and use the Align panel, also with **Align to parent** |

- A line is added to the selected piece whatever the operation mode is, because it can't be a piece of its own. With no piece selected, the status bar asks for one.
- It is drawn above every outline and never clipped by it. It is 0.8 px wide, thinner than outlines (1.5 px, or 2.5 px when selected), and 1.5 px in orange when it is the selected child.
- Dash and gap are in mm, so they scale with the zoom and match the export. If a pattern's period is under 3 px on screen, the line is drawn as a faded solid line.
- A click within the edge hit radius (6 px) of a line hits it. Clicking a line that lies outside its piece selects that piece, and the next click selects the line. Among the children under the cursor a line wins, since it counts as having no area.
- Line types are **Solid**, **Dashed** (dash, gap; 5 / 2 mm by default) and **Dash-dot** (dash, gap, 1 mm dot, gap; 10 / 2 mm by default, the default for new lines). Choosing another type starts from its defaults.
- The 3D preview doesn't show lines.

## The model

```rust
pub struct Line {
    pub start: Point,
    pub end: Point,
    #[serde(default)]
    pub style: LineStyle, // Solid | Dashed { dash, gap } | DashDot { dash, gap }, tagged "type"
}
```

- `Shape::ring` of a line is its two ends. So `bounds` works unchanged, and `contains` is always false (fewer than 3 points). Code that treats a ring as a closed outline must check `is_open` first.
- The child's `op` is kept, but ignored for an open shape. New lines get `BooleanOp::default()`.
- `corners` is `None`, so the Edge tool picks no vertices on a line, and **To Polygon** isn't offered.
- The format change is additive (`"kind": "line"`), so `PROJECT_VERSION` stays the same and older files load unchanged. A line saved without `style` loads as solid.

### Validation

`PatternPiece::validate_shapes` checks:

- The base must not be open (`BaseMustBeClosed`).
- A line has a finite length above 1e-6 mm, and dash and gap are at least `MIN_DASH_MM` (0.1). Otherwise it fails with `InvalidShapeSize`.
- Its ends are within `MAX_COORDINATE_MM`.
- `dash_count` is at most `MAX_DASHES_PER_LINE` (4096; `TooManyDashes`), which keeps `dashes` bounded for drawing and DXF. `dashes` also stops at that count by itself.

### Combining and aligning

- `PatternPiece::region` skips open children, so a line never changes the outline, the 3D mesh or the exported cut path.
- `align::child_offsets` measures each child by `shape.bounds()`, which for a line is the box of its ends. A vertical or horizontal line has a zero-width box, which aligns as expected. Pieces are still measured by their outline, so their lines don't move them.

## Export

- **SVG:** a second `<g>` after the outlines, with `stroke-width="0.15"`, holds one `<line data-piece data-child …>` per line. Dashes come from `stroke-dasharray`, built from `LineStyle::pattern`. The drawing's bounds include the lines, so a line that reaches past its piece is not cut off.
- **DXF:** each visible segment of `Line::dashes` is a `LINE` entity on layer `CENTERLINE`, so the pattern survives in any reader without a linetype table. Coordinates are the original ones, as for outlines.

## Extending

- Another open shape, such as a polyline, needs: a `Shape` variant with `is_open` true, its `distance_to`, the drawing in `draw_centerlines`, export output, and validation in `validate_shape`.
- An option to clip lines to the outline would go in `draw_line` and the exports; the model needs no change.
