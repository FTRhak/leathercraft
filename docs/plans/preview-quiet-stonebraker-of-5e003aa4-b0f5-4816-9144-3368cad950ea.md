# Sewn 3D preview

## Context

The 3D preview (`src/preview/mod.rs`) now shows each piece flat, extruded to its thickness. The user wants the preview to show the pieces **sewn together** along their seam links, with centerlines acting as lines where the leather folds more easily, and with each piece's stiffness taken from `material.flexibility`.

Decisions (from the user):
- A soft-body simulation (XPBD cloth), animated as it settles.
- Fold direction is automatic. No model or file-format change.
- A **Flat / Sewn** toggle plus a **Re-sew** button in the preview controls. Flat is today's view.
- Unlinked pieces lie flat where they are.

This replaces the CLAUDE.md invariant "the preview shows individual flat pieces… does not infer seams". It also replaces the matching note in `docs/plan.md`.

## Design

### Library: new `src/assembly/` module (std only, tested without Bevy)

Following the crate boundary, the solver lives in the library. It needs no new dependencies: a small `Vec3` type lives in the module.

- **`assembly/mesh.rs`: sim mesh per piece.**
  - Start from `PatternPiece::region()`, not the punched one.
  - Spacing `h = clamp(sqrt(area / 700), 3 mm, 20 mm)`, which caps a piece at about 700 nodes.
  - Points come from:
    - the outline and hole rings, resampled at `h`;
    - an interior grid at `h`, kept more than `h/2` from the boundary;
    - samples of each centerline (`Shape::Line`) at `h`, clipped to the region;
    - the center of every stitch hole that takes part in a link (`StitchLine::holes`).
  - Triangulate with Bowyer–Watson Delaunay. Drop triangles whose centroid is outside the region.
  - Keep a map from `(child id, hole index)` to node.
  - Output: `PieceMesh { nodes_2d, triangles, edges, bend_pairs (opposite vertices across each interior edge, with an is_crease flag), hole_nodes }`.
  - An edge is a crease when its midpoint lies within `0.35·h` of a centerline and it runs roughly parallel to it (|cos| > 0.9).
- **`assembly/stiffness.rs`:** `bend_compliance(Flexibility)` is a table from `VerySoft` (high compliance) to `Rigid` (near 0). The crease multiplier is `CREASE_SOFTNESS ≈ 200`. Stretch compliance is near 0 for every material, since leather barely stretches. The seam gap is `(tA + tB) / 2`.
- **`assembly/placement.rs`: initial pose (the automatic fold).**
  - Build a graph of pieces joined by links. Each connected component starts at its largest piece, which stays at its canvas position, grain side up (y = 0).
  - BFS over the links. For each newly reached piece, fit a 2D Kabsch/Procrustes transform from its paired hole points to the placed piece's points (`SeamLink::pairs` gives the pairs). Then:
    - **Reflection fits** (mirror edges, e.g. wallet front and back): flip the piece about an in-plane axis and stack it under the placed one, on its flesh side, a few thicknesses apart. Grain stays outside.
    - **Rotation fits where the bodies overlap** (lapped seam): stack it on top.
    - **Rotation fits where the bodies lie on opposite sides** (butt seam): keep it in plane, pre-folded 90° toward the flesh side about the seam axis.
  - **Self-links** (a flap folded onto its own piece): for each centerline that separates paired holes, pre-rotate the smaller side 20° toward the flesh side. The seam then pulls the fold closed.
  - Unlinked pieces aren't simulated.
- **`assembly/solver.rs`: `Simulation` (XPBD).**
  - Particle mass comes from the triangle area. Substeps run with damping and no gravity.
  - Constraints:
    - Stretch on every edge.
    - Bending as a distance constraint across each `bend_pair`, with the piece's compliance (×crease softness on creases).
    - Seam: each hole pair is pulled to the gap. Its rest length ramps from the start distance down to the gap over the first ~60% of `SETTLE_STEPS`, so the sewing is visible and stable.
    - A simple repulsion through a spatial hash keeps particles of different layers at least the gap apart.
  - API:
    - `Simulation::new(&Project) -> Simulation`
    - `step(budget: Duration)`, which runs substeps until the time budget is used, so the frame rate holds in dev builds
    - `settled() -> bool`, true when the max speed is under a threshold or the step limit is reached
    - `positions(piece) -> &[Vec3]`
    - `normals(piece)`
    - `locate(piece, Point) -> (triangle, barycentric)`, used to map render vertices
  - Positions are in mm. After placement and solving, each component is shifted so its lowest point is at y = 0.
- **Tests (inline):**
  - The Delaunay mesh covers the region area within 2% and includes hole and centerline nodes.
  - Creases are flagged along a centerline.
  - Kabsch detects a reflection for mirrored seams.
  - Two square pieces with mirrored edge seams settle with paired holes within `gap + ε`.
  - A single piece with a center fold line and a self-link folds, so the two halves end up roughly parallel.
  - A `Rigid` piece bends less than a `VerySoft` one under the same seam pull.
  - Unlinked pieces are left out.

### Binary: `src/preview/`

- Split into `mod.rs` (plugin, camera, theme, controls), `flat_mesh.rs` and `sewn.rs`.
- Refactor `build_piece_mesh` into `piece_surface(region, thickness) -> Surface { points_2d, layer (0 or 1 per vertex), indices }`. It does the same earcut on `punched_region()`, plus the walls.
  - **Flat mode** maps each vertex to `(x, layer·t, y)`, exactly as today.
  - **Sewn mode** binds each surface vertex once to a sim triangle via `Simulation::locate`. Each frame its position is the barycentric sim position plus the normal times `(layer − 0.5)·t`, then the normals are recomputed. The stitch holes and thickness stay visible.
- `PreviewSettings` gains `mode: PreviewMode { Flat, Sewn }` and a `resew` request flag. These are interface state, not saved in the project.
- Systems:
  - **`rebuild_preview`:** on a project change or mode change, despawn the meshes and spawn them again. In Sewn mode it also creates a `SewnScene` resource (the `Simulation` plus each piece's bindings and mesh handle).
  - **`step_sewn`** (Update, Sewn only, until `settled`): `sim.step(6 ms)`, then rewrite the `ATTRIBUTE_POSITION` and normals of each mesh asset.
  - **Re-sew** rebuilds the `SewnScene`.
  - Camera distance and center come from the scene's 3D bounds, recomputed when the scene is rebuilt.
- `show_controls`: a `GroupButton` with Flat and Sewn, a `Button` labelled Re-sew (enabled in Sewn mode), and a "Settling…" note while running. All of these exist in `src/ui_components/`.

### i18n, docs

- `Label::PreviewFlat`, `PreviewSewn`, `Resew`, `Settling` in `src/i18n/en.rs` and `uk.rs`.
- New `docs/assembly.md`: the model, placement rules, stiffness table, limits. The limits: no full self-collision, so layers between particles can pass through each other; folds only where the seams pull.
- Update `docs/project-index.md`, the CLAUDE.md architecture list and invariant, and the `docs/plan.md` note.

## Critical files

- New: `src/assembly/{mod,mesh,stiffness,placement,solver}.rs`, `docs/assembly.md`
- Edit: `src/lib.rs` (the `pub mod assembly`), `src/preview/mod.rs` (split into `flat_mesh.rs` and `sewn.rs`), `src/i18n/{mod,en,uk}.rs`, `CLAUDE.md`, `docs/project-index.md`, `docs/plan.md`
- Reuse:
  - `PatternPiece::region` and `punched_region` (`src/boolean.rs`)
  - `SeamLink::pairs`, `seams::stitch_line` (`src/seams.rs`)
  - `StitchLine::holes` (`src/shapes/stitch.rs`)
  - `Shape::Line` (`src/shapes/line.rs`)
  - `Region::contains`, `signed_area`, `ring_contains` (`src/model.rs`)
  - `Material::flexibility` and `thickness_mm` (`src/material.rs`)
  - `GroupButton` and `Button` (`src/ui_components/`)

## Verification

1. `cargo test --lib --no-default-features -q`: the new assembly tests and the existing ones.
2. `cargo check --bin leathercraft --message-format=short` and `cargo clippy`, then `cargo fmt`.
3. Manually, with `cargo run`:
   - **Wallet:** two rectangles with mirrored side and bottom seams. Sewn should show them stacked and stitched.
   - **Card holder:** one piece with a centerline fold and a self-link. It should fold in half.
   - **Flexibility:** switching a piece's flexibility from Very soft to Rigid changes the curvature.
   - **Unlinked pieces** stay flat.
   - **Flat** looks as it did before.
   - **Re-sew** replays the animation.
   - **Editing** a seam while Sewn rebuilds the scene.
