# Sewn preview (`drape::Drape`)

The 3D preview can show the pieces sewn together along their seam links. Each piece becomes a triangle mesh that keeps its size and bends as easily as its material allows, and bends most easily along its centerlines, which mark its folds. The paired holes of each link are drawn together the way the link's assembly lays the pieces, with the slack of its stitch. The simulation lives in the library crate, so it is tested without Bevy. The preview steps it once per frame until it settles and redraws the meshes.

| File | Contents |
|---|---|
| `src/drape/mod.rs` | `Drape` (`new`, `step`, `is_settled`, `positions`, `pieces`, `bounds`, `vertex_normals`, `threads`, `thread_segments`), `DrapedPiece`, `SeamThread` (the link's stitch, `Thread` and lock holes), `ThreadSegment` (its seam, two ends and the normal of the face it lies on), `Join` and `join_of` (the assembly as geometry), `slack` (by stitch), `attach` (a hole as a point of a triangle); tests |
| `src/drape/sheet.rs` | `Sheet`, `build`: a piece's mesh (resampled outline and holes, points along centerlines, a hexagonal grid inside) and its crease edges; tests |
| `src/drape/delaunay.rs` | `triangulate`: Bowyer–Watson with neighbor links; tests |
| `src/drape/placement.rs` | `Iso`, `Pose`, `fit` (best rigid move, turned over or not), `prefold`, `preroll`, `place`; tests |
| `src/drape/solver.rs` | `Solver` (XPBD), `Distance`, `Bend` (quadratic bending), `Seam`, `SeamKind`, `Anchor`, and how the material sets the compliances; tests |
| `src/drape/vec3.rs` | `Vec3` |
| `src/preview/mod.rs` | `PreviewMode` (Sewn, Flat), `PreviewSettings`, `rebuild_preview` (only while the preview is shown), `step_sewn` |
| `src/preview/controls.rs` | `show_controls` (the View switch and Sew again) |
| `src/preview/sewn.rs` | `SewnModel`: the `Drape` and its mesh handles, `piece_mesh` (a shell of the leather's thickness), `thread_mesh` (each seam a mesh of tubes as wide and high as its thread), `thread_material` (its color, with a finish by kind) |
| `src/preview/flat.rs` | `build_piece_mesh`: the flat preview, unchanged |

## For users

| Action | How |
|---|---|
| See the item sewn | Switch to 3D; **View** is **Sewn** by default. The pieces are laid onto each other and drawn together over about a second and a half |
| See the pieces flat | **View → Flat**: each piece on its own, with its holes cut through, as before |
| Start the sewing over | **Sew again** |
| Make a piece fold in a place | Draw a centerline there. The piece folds there much more easily than anywhere else |
| Make a piece bend more or less | Its material's **Flexibility** and **Thickness** |

- Pieces with no seam links lie flat where they are drawn. Pieces whose shapes don't combine into one outline are left out, as in the flat preview.
- Stitch holes aren't cut in the sewn model. The thread is drawn over the seams instead: along both faces for a saddle stitch, every other stitch on each face for a running stitch (both faces over its lock holes, where it is sewn back), and crossing between the two seams for the baseball, whip and cross stitches. Each link's thread is drawn as chosen in the Link seams dialog: in its color, as thick as its diameter, resting on the leather's face, and by its kind round and glossy (bonded nylon), round with a soft sheen (waxed polyester), round and matte (waxed linen), or a flat tape 1.6 times as wide as its diameter and half as high (artificial sinew).
- The model is built when the preview is shown, not on every edit in 2D. Each change to the project builds it again and starts the sewing over.
- There is no gravity and no collision. Pieces that would have to pass through each other to reach their seams can end up crossed. For example, a gusset that the simulation bends to the wrong side stays there.

## How it works

1. **Meshes.** Every piece gets the same spacing, chosen so that the whole project has about 2,500 vertices, and never finer than 1 mm. Each ring of `PatternPiece::region` is resampled at that spacing. Corners that turn by more than 20° are kept, and so are the points where a centerline crosses the edge. Points along each centerline and a hexagonal grid go inside, at least 0.6 of the spacing from the edge and from the lines. Delaunay triangulation follows, and triangles off the leather are dropped (their centroid and edge midpoints must be on it). Mesh edges between two points of one centerline are **creases**.
2. **Holes.** Each paired hole of each link becomes an `Anchor`: its triangle and barycentric weights (the nearest triangle if it falls just outside).
3. **Self-links first.** A piece linked to itself is shaped by its first such link:
   - **Folded or flesh to flesh: `prefold`.** The B side folds over onto the flesh side of the A side, around a half circle as wide as the leather's thickness, so it keeps its size. The fold runs along a centerline that has the two sides on either side of it and lies within about 20° of the pairs' axis. Without one, it runs along the line the pairs are symmetric about.
   - **Butted, turned or lapped: `preroll`.** The piece rolls into a tube around its flesh side, whose circumference is the distance from the A holes to the B holes plus the gap.
4. **Placement.** Pieces sewn together form groups. Each group's root is its piece with the most links, which stays where it is drawn. Every other piece is laid onto the piece it is linked to by `fit`, the best rigid move from its holes onto its partner's. It is turned over for the assemblies where the two pieces face each other. Stacked pieces are lifted by the gap; butted ones are moved apart by it. The root lies grain up, unless its flesh-to-flesh partner is the one **On top**. A small noise off the plane lets a piece that has to curve leave it.
5. **Simulation.** XPBD, 8 substeps a frame, velocities damped to 0.9 a substep, no gravity:
   - each mesh edge keeps its length (stretch compliance, by flexibility);
   - each inner edge keeps the leather flat across it, in the quadratic bending model of Bergou et al. (2006). The bend is a cotangent-weighted sum of the edge's ends and the two vertices across it, zero while flat. It is linear in the curvature and keeps the same stiffness at any mesh spacing;
   - each pair of holes is a `Seam`. Its distance from the start shrinks to its target over `RAMP_FRAMES` (90), so the seams draw the pieces in rather than snapping them.
6. **Settling.** The model is settled once the seams are closed and nothing moves more than 0.002 mm a substep, or after 20 seconds. The preview then stops updating its meshes.

### Assemblies

| Assembly | `Join` | Seam target |
|---|---|---|
| Flesh to flesh, Folded | Stacked, B turned over, on A's flesh side | B's hole at the mean thickness along A's normal, within the stitch's slack |
| Lapped | Stacked, B the same way up, on A's grain side when B is on top, else its flesh side | The same |
| Butted | Beside | The holes as far apart as each hole is from its piece's edge, so the edges touch |
| Turned | Beside | The holes one thickness apart: sewn grain to grain and opened |

Stitch slack, in mm: saddle 0, baseball 0.15, running 0.3, whip 0.4, cross 0.5.

### Material

| Flexibility | Bending compliance | Stretch compliance |
|---|---:|---:|
| Very soft | 20 | 2e-3 |
| Soft | 8 | 1e-3 |
| Semi-soft | 3 | 5e-4 |
| Medium | 1 | 2e-4 |
| Stiff | 0.3 | 1e-4 |
| Very stiff | 0.08 | 5e-5 |
| Rigid | 0.02 | 2e-5 |

Bending compliance is divided by (thickness / 1.2 mm)², clamped to 0.25–6, so thick leather is stiffer. A crease is 400 times softer than its leather, and never stiffer than 200. That ratio is what makes a centerline the fold: stiff leather folds sharply at the line and stays flat beside it, while soft leather rounds the fold off. Each vertex's mass is its share of the area times the thickness, so thick pieces give way less at a seam.

Without gravity, the shape of an item made of one material doesn't depend on how stiff the material is, only on where it may fold. Flexibility shows where pieces of different materials meet, and in how sharp the centerline folds are.

## Model coordinates

Millimeters, with the pattern's `x` along X, its `y` along Z, and Y up. Triangles are stored clockwise in the pattern's plane, so that `(b - a) × (c - a)` points out of the grain side. A piece turned over (`Iso::mirror`) has its grain side down. The preview centers the model over the origin, stands it on the ground (its lowest point lifted by half the thickest leather) and scales it by `MM_TO_SCENE`.

## Tests

`cargo test --lib --no-default-features drape`:

| Module | Covers |
|---|---|
| `drape::delaunay` | A grid is covered once by counterclockwise triangles; every triangle of random points is Delaunay |
| `drape::sheet` | A rectangle's mesh covers its area; a hole stays open; a centerline becomes creases from edge to edge |
| `drape::placement` | `fit` with and without turning over; `prefold` lays the B side under the A side and keeps its size |
| `drape::solver` | Stiffer and thicker leather bends less, and a crease far less; a flat hinge has no bend; a seam closes to its gap |
| `drape` | A strip folds on its centerline into a flat card holder with its seams closed; two panels sew flesh to flesh into one footprint; butted panels lie edge to edge; a strip butted to itself rolls into a cuff; an unlinked piece stays flat and still |

The meshes, the thread and the controls are checked by hand in the app.
