# Rounded vertices (`shapes::corners`)

How a corner of a shape is rounded by a radius. The radius is a stored parameter of the vertex, not geometry baked into the shape. The outline is rebuilt from the sharp vertices and their radii every time it is needed, so the rounding survives moving vertices, resizing, undo and save/load, and the value can be changed at any time. The geometry lives in the library crate, so it can be tested without Bevy.

| File | Contents |
|---|---|
| `src/shapes/corners.rs` | `round_corners`, `fillet`, `radius_at`, `set_radius`, `valid_radii`, tests |
| `src/shapes/polygon.rs`, `rectangle.rs`, `regular_polygon.rs` | The `radii` field; `ring` (rounded) and `corners` (sharp) |
| `src/shapes/mod.rs` | `Shape::corners`, `corner_radius`, `with_corner_radius`; `handles` and `to_polygon` |
| `src/editor/mod.rs` | `ShapePart` (`Edge` or `Vertex`) |
| `src/editor/editor_state_struct.rs` | `EditorState::selected_part`, `select_part`, `selected_part()` |
| `src/editor/canvas.rs` | `handle_edge`, `vertex_at`, the vertex highlight in `draw_edges`, edge labels by corners |
| `src/panels/properties/mod.rs` | The Corner radius field in `shape_fields` |

## For users

| Action | How |
|---|---|
| Select a vertex | With the **Edge** tool, click a vertex handle of the selected piece |
| Round it | Set **Corner radius** in the Properties panel, under *Vertex N* |
| Make it sharp again | Set the radius to 0 |
| Clear the selection | Esc |

- Rounding works on polygons, rectangles and regular polygons, for the base shape and for children. An ellipse has no vertices.
- Only vertices of the selected, unlocked piece can be picked: handles of the selected child are tried first, then those of the base. For a locked piece the field is disabled.
- A drag on a vertex still moves it, and a click on a polygon edge still selects the edge for its length. A click on nothing else selects as the Select tool does.
- The selected vertex is marked with a ring on the canvas. Handles stay on the sharp corner, not on the arc.
- The radius is kept as typed. If the corner's edges are too short for it, the arc is made smaller on the canvas, in exports and in the 3D preview, and grows back to the full radius when the edges are lengthened again.
- Dragging the radius field makes one History entry, like other shape fields.
- **To Polygon** keeps the corners and their radii, so a rounded rectangle becomes a rounded polygon with the same outline.
- Edge length labels measure between the sharp vertices, so an arc doesn't get labels of its own. The exception is a piece with children: its labels follow the combined outline, where each arc segment can get a small label, and no labels are shown past 64 points.

## The model

`Polygon`, `Rectangle` and `RegularPolygon` each have:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub radii: Vec<f64>,
```

| Rule | Meaning |
|---|---|
| Index | `radii[i]` rounds vertex `i` of `Shape::corners` |
| Missing entries | Count as 0, so an empty list means every corner is sharp |
| Extra entries | Ignored, for example after lowering a regular polygon's `sides` |
| Valid values | Finite and ≥ 0; checked by `has_valid_size` through `valid_radii`, so `Project::validate` rejects anything else |
| Sharp shapes | Save without a `radii` key, exactly as before the feature |

The field is `#[serde(default)]`, so older files load unchanged and `PROJECT_VERSION` was not bumped.

### API

| Item | Purpose |
|---|---|
| `Shape::corners()` | The sharp vertices, or `None` for an ellipse. A polygon's `points`, the rectangle's four corners, or the regular polygon's vertices |
| `Shape::ring()` | The outline, with each rounded corner replaced by an arc; what boolean operations, exports, hit testing and the preview use |
| `Shape::handles()` | The sharp corners for all three shapes, never ring points |
| `Shape::corner_radius(i)` | The radius of vertex `i`, 0 if unset; `None` if there is no such vertex |
| `Shape::with_corner_radius(i, r)` | A copy with vertex `i` set to `r`; `None` for a missing vertex, an ellipse, or a negative or non-finite `r` |
| `Shape::to_polygon()` | Keeps `points` and `radii`; a polygon is returned unchanged; an ellipse becomes the points of its ring |
| `Polygon::new(points)` | A polygon with sharp corners |
| `corners::set_radius(radii, i, count, r)` | Resizes the list to `count`, sets entry `i`, and empties the list when every radius is 0 |

`with_handle_at` keeps `radii` for every shape, and `Rectangle::with_handle_at` takes the fixed corner from `corners()`, so resizing never reads arc points.

## Building the arc

`round_corners(corners, radii)` walks the vertices. A vertex with radius ≤ 0, or one that `fillet` can't round, stays as a single point. Otherwise it is replaced by the points of its arc. For vertex P with neighbors A (previous) and B (next):

1. u = unit(A − P), v = unit(B − P), and the corner angle θ = acos(u·v).
2. Skip the corner if either edge is shorter than `MERGE_DISTANCE_MM` (1e-6 mm), or θ is within `MIN_ANGLE` (1e-6 rad) of 0 or π, which is folded back or flat.
3. Tangent distance t = r / tan(θ/2), clamped to `min(|PA|, |PB|) / 2`. The effective radius is r' = t · tan(θ/2). Each arc reaches at most halfway along an edge, so neighboring arcs never overlap.
4. The arc runs from P + u·t to P + v·t around the center P + unit(u + v) · r' / sin(θ/2). Its sweep is π − θ, and its direction comes from the sign of the cross product, so concave corners round correctly too.
5. Segments: `ceil(sweep / (TAU / CIRCLE_SEGMENTS))`, at least 1, which is the same density as an ellipse: 18 segments for a right angle.

Points closer than `MERGE_DISTANCE_MM` are merged, including across the wrap from last to first. Without this, two arcs that both reach an edge's midpoint would leave a repeated point and fail validation with `CoincidentPoints`.

A rounded corner adds about one point per 5° of sweep. `MAX_POINTS_PER_PIECE` (2,048) applies to each shape's ring, so an edit that would exceed it is rejected like any other invalid edit.

## Message flow in the binary

The feature has no `ProjectEvent` of its own. A radius change is an ordinary shape edit:

1. `canvas::handle_edge`: a click calls `vertex_at`, which runs `handle_at` on the selected, unlocked piece and requires the shape to have `corners()`. A hit calls `state.select_part(piece, shape, ShapePart::Vertex(i))`. Otherwise `edge_at` is tried and selects `ShapePart::Edge`, and then `select_at`.
2. `EditorState::selected_part()` returns the part only while its piece is selected and its shape is the one shown in Properties. The index can be out of range after an undo or a change of sides, so readers check it against the shape.
3. `panel_viewer` passes the part to `properties::Selection.part`. `shape_fields` shows the field when `corner_radius(i)` is `Some`, and applies `with_corner_radius` to its edited copy.
4. `selected_piece_details` sends `SetBaseShape` or `SetChildShape`. `Project::apply` validates it, and `History` merges consecutive edits of the same shape (`MergeKey::Base` or `MergeKey::Child`).

## Tests

| Module | Covers |
|---|---|
| `shapes::corners` | Zero radii leave corners unchanged; arc points lie on the circle and touch both edges; a large radius is clamped to halfway (a square becomes a circle) with no repeated points; flat corners stay sharp; concave corners; `set_radius` and `valid_radii` |
| `shapes` | Handles and corners don't move when rounded; resizing keeps radii; radius 0 restores the original shape; invalid radii and ellipses are rejected; `to_polygon` keeps radii and the ring; fewer sides ignore extra radii |
| `events` | Rounding through `SetBaseShape`; a negative radius is rejected and leaves the project unchanged; returning to sharp |
| `persistence` | Radii round-trip; sharp shapes save without a `radii` key |

The selection and the Properties field have no tests. Check them with `cargo check` and a manual run.

## Extending

- **A new shape with vertices:** add a `radii` field with the same serde attributes, build `ring` with `corners::round_corners(&self.corners(), &self.radii)`, check `valid_radii` in `has_valid_size`, keep `radii` in `with_handle_at`, and add its arm to `Shape::corners`, `radii`, `with_corner_radius` and `to_polygon`.
- **A different arc density:** change the segment count in `fillet`. Exports and the preview follow, since they read `ring`.
- **Rounding several vertices at once:** today only one vertex is selected. A multi-selection would replace `ShapePart::Vertex(usize)` and apply `with_corner_radius` for each vertex in one `SetBaseShape` or `SetChildShape`.
