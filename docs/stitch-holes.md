# Stitch holes (`shapes::StitchLine`)

A stitch line is an open polyline kept as a child of a piece. It marks where the piece is punched for a hand (saddle) stitch. The user draws the line, sets the hole type, the pitch (distance between holes) and the hole size, and the editor shows the holes instead of the line. The holes are computed from the line and its parameters every time they are needed, never stored.

A stitch line is a `ChildShape` whose shape is `Shape::StitchLine`, so ids, selection, reordering, deleting, moving, aligning, undo and save work for it as for any other child. Like a centerline (`docs/centerline.md`), it is open: it takes no part in `PatternPiece::region`, so it never changes the outline, the validation of the result, or the exported cut path. The holes go to their own export layer, and only the 3D preview punches them through the piece. The geometry lives in the library crate, so it can be tested without Bevy.

| File | Contents |
|---|---|
| `src/shapes/stitch.rs` | `StitchLine`, `HoleKind`, `Hole`, `Run`, `StitchBand`, the constants below: `new`, `snap`, `path`, `length`, `runs`, `holes`, `hole_count`, `hole_outline`, `actual_pitch`, `distance_to`, `handles`, `with_handle_at`, `translate`, `has_valid_size`, `has_valid_parameters`, `holes_overlap`; tests, including the validation rules |
| `src/shapes/corners.rs` | `fillet` and `push_point`, now `pub(crate)` so the path can round its inner vertices |
| `src/shapes/mod.rs` | `Shape::StitchLine`; its arms in `is_open`, `distance_to`, `ring`, `corners`, `radii`, `with_corner_radius`, `center`, `translate`, `handles`, `with_handle_at`, `has_valid_size`; `can_round` keeps the ends sharp; `contains` is false for any open shape; `to_polygon` returns it unchanged |
| `src/model.rs` | `validate_stitch_line`, called from `validate_shape` |
| `src/boolean.rs` | `region` already skips open children; `PatternPiece::punched_region` for the preview |
| `src/exports/` | `stitch_lines`, `HoleOutput`, `stitch_holes` beside `centerlines`; a hole group in `to_svg`, entities on layer `STITCH_HOLES` in `to_dxf` |
| `src/i18n/` | The labels and messages under [Text](#text) |
| `src/panels/tools/mod.rs` | `Tool::StitchHoles`, its icon, `StitchSettings` (the kind, pitch and size of new lines) in `ToolSettings` |
| `src/editor/canvas.rs` | `handle_stitch`, `stitch_target`, `stitch_line`, `finish_stitch`: the click-by-click draft; `commit_shape` (a stitch line is always a child); `draw_stitch_holes`, `draw_stitch_line`, `draw_stitch_draft` |
| `src/editor/editor_state_struct.rs` | `stitch` (the tool's settings) and `draft_tool`, the tool that owns `draft` |
| `src/panels/properties/mod.rs` | `stitch_fields` (shared with the Tools panel), `bounded_length_field`, `stitch_details` |
| `src/panels/layout/mod.rs` | Named *Stitch holes (N)*, with no operation picker |
| `src/preview/mod.rs` | `rebuild_preview` builds the mesh from `punched_region` |
| `src/settings/hotkeys.rs` | `Action::ToolStitchHoles`, H by default |

## For users

| Action | How |
|---|---|
| Add a stitch line | Select a piece, pick **Stitch holes** (H), and click each vertex. Double-click or Enter finishes, Backspace drops the last vertex, Esc cancels |
| Keep a segment at an angle step | Hold Shift while placing a vertex: 15° steps from the previous vertex |
| Choose the parameters beforehand | Under the Tools buttons while the tool is active: **Hole type**, **Pitch** with the **Iron** presets, **Hole size** |
| Change them afterwards | Select the line, then in Properties: the same fields |
| See the result | Properties shows the vertex count, the line's length, the number of holes, the actual pitch, and a thread hint for the pitch |
| Move or reshape it | Drag the line, or one of its vertex handles, with Select |
| Round a corner of the path | With the **Edge** tool, pick an inner vertex and set **Corner radius**, as for polygons (`docs/round-vertex.md`). The ends have no radius field |
| Align it | Select it with other children (Shift+click) and use the Align panel, also with **Align to parent** |

- A stitch line is added to the selected piece whatever the operation mode is. With no piece selected, the status bar asks for one.
- It needs at least two vertices. Finishing with fewer drops the draft.
- While drafting, the canvas shows the vertices, the segment to the cursor, the holes the line would get, and its length and hole count next to the cursor.
- The canvas shows the holes, not the line. The path is drawn faintly only while the line is selected or under the cursor, so it can be grabbed.
- A click within the edge hit radius (6 px) of the path hits it. Among the children under the cursor a stitch line wins, like a centerline, since it has no area.
- Pitch presets are the usual iron sizes, in mm whatever the display units: 2.7, 3.0, 3.38, 3.85, 4.0 and 5.0. Any value in range can be typed.
- Choosing another hole type resets the size to that type's default.
- Hints never block an edit: the recommended thread and leather thickness for the pitch, and a warning when the piece's material thickness falls outside that range. See the [reference data](#appendix-reference-data).
- The 3D preview shows the holes punched through the piece.

## The model

```rust
pub struct StitchLine {
    pub points: Vec<Point>,  // the open path, at least 2 points
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radii: Vec<f64>,     // per vertex, as in docs/round-vertex.md; the ends stay sharp
    pub hole: HoleKind,
    pub pitch: f64,          // mm, center to center along the path
    pub size: f64,           // mm: the diameter of a round hole, the long side of the others
}

#[serde(rename_all = "snake_case")]
pub enum HoleKind {
    French,   // a thin slanted slot
    Diamond,  // a slanted rhombus; the default
    Round,    // a circle, no slant
}
```

The kind is stored as `hole`, not `kind`, because `Shape` is internally tagged with `"kind"` and the two keys would collide in JSON.

| Kind | Default size | Width | Slant to the path | Outline |
|---|---|---|---|---|
| French | 1.2 mm | `size × FRENCH_WIDTH_RATIO` (0.25) | `HOLE_SLANT` (45°) | Rectangle, 4 points |
| Diamond | 1.6 mm | `size × DIAMOND_WIDTH_RATIO` (0.45) | `HOLE_SLANT` (45°) | Rhombus, 4 points |
| Round | 1.2 mm | = size | none | Circle, `HOLE_SEGMENTS` (16) points |

New lines default to Diamond with a pitch of `DEFAULT_PITCH_MM` (3.85 mm), the most common iron for wallets and small bags.

- `Shape::ring` of a stitch line is its path, with rounded vertices replaced by arcs from `corners::fillet`. Code that treats a ring as a closed outline must check `is_open` first.
- `Shape::contains` is false for every open shape. Before stitch lines it held for a `Line` only because its ring has two points.
- `corners` is `Some(points)`, so the Edge tool can pick a vertex. `corner_radius` and `with_corner_radius` return `None` for the two ends. `to_polygon` returns the line unchanged, and **To Polygon** isn't offered for any open shape.
- The child's `op` is kept, but ignored for an open shape. New lines get the operation mode's op, or `BooleanOp::default()`.
- The format change is additive (`"kind": "stitch_line"`), so `PROJECT_VERSION` stays the same and older files load unchanged. A line with sharp vertices saves without a `radii` key.

### Validation

`PatternPiece::validate_shape` checks `has_valid_size`, then `validate_stitch_line` checks the rest, in this order:

| Rule | Error |
|---|---|
| At least 2 points; `radii` finite and ≥ 0 | `InvalidShapeSize` |
| Every point within `MAX_COORDINATE_MM` | `CoordinateOutOfRange` |
| Neighboring vertices at least 1e-6 mm apart | `CoincidentPoints` |
| The path within `MAX_POINTS_PER_PIECE` points | `PointCountOutOfRange` |
| `MIN_HOLE_MM` (0.3) ≤ size ≤ `MAX_HOLE_MM` (10); `MIN_PITCH_MM` (1.0) ≤ pitch ≤ `MAX_PITCH_MM` (50) | `InvalidHoleParameters` |
| `hole_count` ≤ `MAX_HOLES_PER_LINE` (4096) | `TooManyHoles`, which also bounds drawing, export and the preview |
| The hole's footprint along the path is under the smallest actual pitch, so neighboring holes don't merge | `HolesOverlap` |

`HoleKind::footprint` is the size for a round hole; for a slanted hole it is the extent of its outline along the path: `size·cos + width·sin` for a French slot, `max(size·cos, width·sin)` for a diamond. A very short run between two sharp vertices has a small actual pitch, so it can trigger `HolesOverlap` too.

A stitch line can't be a base (`BaseMustBeClosed`). Holes outside the piece's outline are not an error: a line can be dragged across the edge, like any child.

## Hole placement

`StitchLine::holes` returns `Vec<Hole>`, where `Hole { center: Point, angle: f64 }`.

1. **Runs.** The path is split at its anchors: the two ends, and every inner vertex that is sharp (radius 0, or a corner `fillet` can't round). A rounded vertex belongs to the run around it, since its arc is part of a smooth curve. Each anchor gets a hole, so a corner is always stitched through, as when walking an iron into a corner.
2. **Spacing.** For a run of length L, the number of intervals is n = max(1, round(L / pitch)), and the holes sit at k · L / n for k = 0..n. The actual pitch of the run is L / n. A hole shared by two runs is placed once.
3. **Angle.** The direction of the path at the hole: its segment's direction, or, at an anchor vertex, the bisector of its two segments. `HoleKind::slant` is added: `HOLE_SLANT` for French and Diamond, 0 for Round.
4. **Outline.** `hole_outline(&Hole)` returns a closed ring in project coordinates, without a repeated closing point, centered on the hole and turned by its angle.

`hole_count` and `actual_pitch` work from `runs` without building the holes. `actual_pitch` returns the smallest and largest run pitch, shown in Properties next to the pitch, so the user sees how far the spacing drifts from the iron.

## Canvas

- `draw_stitch_holes` runs after `draw_centerlines`, so holes are drawn above every outline and never clipped.
- Each hole is filled with the canvas background and outlined 1 px, so it reads as punched. Round holes are circles; the others are their outlines. The holes of the selected line are orange.
- If the pitch is under `MIN_HOLE_PITCH_PX` (3 px) on screen, the path is drawn as a faded dotted line instead of the holes.
- The path itself is drawn faintly, `CENTERLINE_WIDTH_PX` wide, while the line is selected or within the hit tolerance of the cursor. Its vertex handles are shown while it is the selected child of an unlocked piece.
- `draw_stitch_draft` replaces `draw_draft` while the tool is active.

## Export

Holes are exported for every stitch line, whether or not they fall inside the outline. `stitch_holes` builds them once from `holes` and `hole_outline`, and both exports use it.

- **SVG:** a `<g>` after the centerlines, with `fill="none"` and `stroke-width="0.1"`. Each hole is a `<circle>` for a round hole or a `<polygon>` for the others, with `data-piece` and `data-child`. The drawing's bounds include the holes.
- **DXF:** layer `STITCH_HOLES`. A round hole is a `CIRCLE`; the others are closed `LWPOLYLINE`s, as outlines are on `PATTERN`. Coordinates are the original ones.

A laser or plotter can then cut the outline layer and the hole layer separately, or the hole layer can be printed as a punching template.

## 3D preview

`PatternPiece::punched_region` returns `region` minus every stitch hole's outline, with `i_overlay` (`OverlayRule::Difference`, non-zero fill). `rebuild_preview` builds the mesh from it, so holes show through the extruded piece.

- `region` itself never changes, so validation, the editor's flags and exports are unaffected.
- Holes outside the outline have no effect. One that crosses the edge leaves a notch.
- If `region` fails, so does `punched_region`, and the preview skips the piece as before. If punching leaves anything but one outline, the unpunched region is used.

## Message flow in the binary

The feature has no `ProjectEvent` of its own:

1. `handle_stitch` collects clicks into `EditorState::draft`, the same list the Draw tool uses. `draft_tool` records which tool owns it, so switching tools drops it. Nothing reaches the project while drafting.
2. Finishing calls `finish_stitch`, which builds the line from `EditorState::stitch` and calls `commit_shape`. That sends `AddChild` to the selected piece. An invalid line is rejected by `Project::apply` and its message shown.
3. Edits in Properties, handle drags and vertex rounding send `SetChildShape`. `Project::apply` validates the copy, and `History` merges consecutive edits of the same child (`MergeKey::Child`).
4. `handle_hotkeys` runs before the canvas and is skipped while a draft exists, so Backspace drops a vertex instead of deleting the selection.

## Text

`Label`s: `StitchHoles`, `StitchHolesHint`, `HoleType`, `HoleKind(HoleKind)`, `Pitch`, `PitchPresets`, `HoleSize`, `StitchLength`, `HoleCount`, `ActualPitch`. `Message`s: `InvalidHoleParameters`, `HolesOverlap`, `TooManyHoles`, `StitchHolesRow { count }` (the Layout name), `StitchVertices { count }`, `ThreadHint { band }`, `LeatherOutsideBand { band }`. Both catalogs have a `hole_kind` and a `range` helper; Ukrainian ranges use a decimal comma.

### Hints

`StitchBand::from_pitch` groups pitches as the reference table does: under 3.2 mm `Fine`, under 3.7 mm `Classic`, under 4.5 mm `Medium`, otherwise `Heavy`. `thread_mm` and `leather_mm` give the ranges from the table, and `suits_leather` compares the piece's `material.thickness_mm` with the leather range.

## Tests

| Module | Covers |
|---|---|
| `shapes::stitch` | Even spacing and actual pitch on a straight run; holes at both ends and at sharp vertices; rounded vertices don't anchor; slant per kind and the bisector at a vertex; outline points per kind; overlap and parameter limits; `distance_to`, handles, `translate`; Shift snapping; bands; every validation error, and a stitch line as a base |
| `shapes` | A stitch line is open, `contains` is false, `to_polygon` keeps it, only inner vertices round |
| `boolean` | `region` ignores stitch lines; `punched_region` cuts the holes inside the outline and ignores those outside |
| `events` | Adding through `AddChild`; an invalid pitch is rejected and leaves the project unchanged |
| `exports` | SVG circles and polygons in their group with offset coordinates; DXF `CIRCLE` and `LWPOLYLINE` on `STITCH_HOLES` |
| `persistence` | Round-trip with sharp and rounded vertices; the `stitch_line` and `hole` keys; no `radii` key when sharp |

Drawing, the draft and the Properties fields have no tests. Check them with `cargo check` and a manual run.

## Extending

- **Offset from the edge:** build the path as an offset of a chosen range of the outline, at the recommended edge distance for the pitch (below). The model needs no change; it is a way to produce `points`.
- **Slant per line:** a `slant` field with `#[serde(default)]`, for left- and right-handed irons, read by `HoleKind::slant` and `footprint`.
- **SPI input:** a pitch field in stitches per inch, converted with 25.4 / SPI.
- **A second row:** a stitch line offset from the first, for double stitching.
- **Flagging stray holes:** mark holes outside the outline on the canvas, using `Region::contains`.
- **A closed stitch line:** around a whole outline, the first and last anchors would merge; `path_with_anchors` would need to wrap like `round_corners`.

## Appendix: reference data

These values drive the presets, the default sizes and the hints.

### Pitch, thread and leather

| Iron pitch | Thread | Total leather thickness | Typical items |
|---|---|---|---|
| 2.7–3.0 mm | 0.35–0.45 mm | 1.0–2.0 mm | Watch straps, small card holders, thin premium wallets |
| 3.38–3.5 mm | 0.45–0.55 mm | 1.5–2.5 mm | Classic wallets, card holders, passport covers, inner pockets |
| 3.85–4.0 mm | 0.55–0.65 mm | 2.5–4.0 mm | Long wallets, clutches, small bags, knife sheaths |
| 5.0 mm and up | 0.8–1.0 mm and up | 4.0 mm and up | Large bags, backpacks, belts, travel bags, footwear |

The smaller and finer the item, the smaller the pitch and the thinner the thread.

### Hole types

| Type | Shape | Use |
|---|---|---|
| French (pricking iron) | Thin, long slanted slits | The finest classic slanted stitch. Usual pitches 2.7, 3.0, 3.38, 3.85 mm |
| Diamond (Japanese) | Rhombus, wider than French | Pierces thick leather more easily, forgives small mistakes. Usual pitches 3, 4, 5 mm |
| Round (hole cutters) | Circle, usually 1.0–1.5 mm | Lacing, or a straight stitch without slant |

### Edge distance

| Pitch | Distance from the edge |
|---|---|
| About 3.0 mm (small items) | 2.5–3.0 mm |
| 4.0–5.0 mm (medium and large items) | 3.5–4.5 mm |

### Needles

The needle must pass freely through the hole with the thread folded double in its eye. Blunt harness needles are used: about No. 004 for thin thread, No. 002 and No. 1 for thicker thread.

### Stitches per inch

| SPI | Pitch |
|---|---|
| 9 | ≈ 2.8 mm |
| 8 | ≈ 3.15 mm |
| 7 | ≈ 3.6 mm |
| 6 | ≈ 4.2 mm |
