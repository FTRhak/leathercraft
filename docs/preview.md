# 3D preview (`src/preview`)

Spec and code index for the binary's 3D preview module: the Bevy scene, the Flat and Sewn meshes, the camera and its navigation, and the controls in the Properties panel. The simulation behind the Sewn mode lives in the library and has its own spec, `docs/sewn-preview.md`. This file covers only how the binary turns a `Project` (Flat) or a `drape::Drape` (Sewn) into meshes and lets the user look at them.

Indexed at commit `db27c30`. Line numbers drift, so search for the symbol name if one doesn't match.

## Scope

- **Display only.** The preview reads `ActiveProject` and never sends a `ProjectEvent`. Nothing it computes is written back to the project.
- **Two modes**, `PreviewMode`:
  - **Sewn** (the default): pieces assembled from the project's seam links by `Drape` and stepped once a frame until the simulation settles. Thread is drawn along each seam.
  - **Flat**: each piece extruded on its own from `PatternPiece::punched_region`, holes cut through.
- **Shown only in `ViewMode::Preview3d`.** In 2D the scene isn't rebuilt or stepped. Edits made in 2D mark it stale, and it is rebuilt when the 3D view is opened.
- **Out of scope:** background images (never drawn), pieces whose result is empty or split (skipped), gravity and collision (see `docs/sewn-preview.md`).

## Module map

| File | Lines | Visibility | Role |
|---|---:|---|---|
| `mod.rs` | 225 | `PreviewPlugin` public, the rest private | Plugin, scene setup, theme, `rebuild_preview`, `spawn_flat`, `step_sewn`; re-exports the public surface |
| `mode.rs` | 9 | public | `PreviewMode` |
| `settings.rs` | 33 | public type, `pub(super)` fields | `PreviewSettings` resource: camera, mode, resew flag |
| `constants.rs` | 13 | `pub(super)` | Scale, camera limits and defaults, ground opacity |
| `navigation.rs` | 91 | `navigate` public | Mouse navigation, camera placement |
| `controls.rs` | 41 | `show_controls` public | Properties panel controls |
| `flat.rs` | 112 | `pub(super)` | `build_piece_mesh`: Flat mode extrusion |
| `sewn.rs` | 269 | `pub(super)` | `SewnModel`: Sewn mode meshes from `Drape` |

### Public surface

The rest of the binary uses only these:

| Item | Used by | Purpose |
|---|---|---|
| `PreviewPlugin` | `main.rs:36` | Registers resources and systems |
| `PreviewSettings` | `editor/ui.rs:42`, `editor/panel_viewer.rs:26`, `panels/properties/mod.rs:38` | Borrowed mutably by the dock to pass to `navigate` and `show_controls` |
| `navigate` | `editor/panel_viewer.rs:73` | Takes the mouse over the canvas tab while in 3D |
| `show_controls` | `panels/properties/mod.rs:149` | Draws the controls in place of the canvas hint box while in 3D |
| `PreviewMode` | (re-exported, only used inside the module today) | |

`PreviewSettings` fields are `pub(super)`, so code outside `preview` can't read or set the camera or mode directly. Changes go through `navigate` and `show_controls`.

## Scene

Spawned once by `setup_scene` (Startup):

| Entity | Components | Notes |
|---|---|---|
| Camera | `Camera3d`, `Transform`, `PreviewCamera` | Renders to the whole window. The egui central panel is transparent in 3D, so the scene shows through the canvas tab. The initial transform is replaced by `update_camera` on the first frame. |
| Light | `PointLight { shadows_enabled }` at (2.5, 5, 3) | Fixed in the scene and doesn't follow the model's size. |
| Ground | 24 × 24 `Plane3d` at y = −0.02, alpha-blended `StandardMaterial`, `Ground` | Colored by `ground_color(theme)` at `GROUND_OPACITY` (5%). |

Spawned by every rebuild, all tagged `PreviewPiece` and despawned by the next rebuild:

| Mode | Entities |
|---|---|
| Flat | One mesh per piece with a valid `punched_region`. Leather material: the piece's `material.color`, roughness 0.88. |
| Sewn | One mesh per `DrapedPiece` (double-sided, no culling) and one per seam thread. All have `NoFrustumCulling`, because the meshes move past the bounds they were spawned with. |

### Resources

| Resource | Owner | Holds |
|---|---|---|
| `PreviewSettings` | `settings.rs` | `azimuth`, `elevation`, `distance`, pan `target`, `fit_distance` (the framing distance a reset returns to), `mode`, `resew` |
| `SewnModel` | `sewn.rs` | The `Drape`, the model point shown at the origin (`center`), mesh handles for the pieces and threads (in `Drape::threads` order) |
| `ClearColor` | Bevy | Set by `apply_theme` |

## Systems and schedule

| System | Schedule | Runs when | Reads | Writes |
|---|---|---|---|---|
| `setup_scene` | `Startup` | once | | spawns camera, light, ground |
| `rebuild_preview` | `Update`, chained before `step_sewn` | the scene is stale, the mode changed or `resew` is set, **and** `ViewMode::Preview3d` | `ActiveProject`, `ViewMode` | `PreviewPiece` entities, `SewnModel`, `PreviewSettings` (`fit_distance`, `distance`, `target`, `resew`) |
| `step_sewn` | `Update` | `Preview3d` and `PreviewMode::Sewn` | `ViewMode`, `PreviewSettings` | `SewnModel`, mesh assets |
| `update_camera` | `Update` | `PreviewSettings` changed | `PreviewSettings` | `PreviewCamera` transform |
| `apply_theme` | `Update` | `Preferences::theme` differs from the last applied one | `Preferences` | `ClearColor`, the ground material |

`navigate` and `show_controls` aren't systems. The dock calls them from `editor_ui` in `EguiPrimaryContextPass`, which runs after `Update`. Input written there reaches the camera on the next frame's `update_camera`.

## Rebuild lifecycle

`rebuild_preview` keeps a `Local<Built { stale, mode }>`:

1. `ActiveProject::is_changed()` sets `stale`. `apply_project_events` touches the resource only for edits that changed content, so selection and background image edits don't mark it.
2. The scene is current when `!stale && built.mode == settings.mode && !settings.resew`. If it is current, or the view isn't 3D, the system returns. A stale scene therefore waits, without cost, until the 3D view opens.
3. Otherwise it clears `resew` (with `bypass_change_detection`), records the mode, despawns every `PreviewPiece`, clears `SewnModel`, and builds the mode's scene:
   - Flat: `spawn_flat` returns `max(width, height)` of `Project::bounds`.
   - Sewn: `SewnModel::rebuild` returns the largest extent of `Drape::bounds`.
4. **Framing.** `fit_distance = clamp(size_mm × MM_TO_SCENE × 2.7, MIN_DISTANCE, MAX_DISTANCE)`, or `DEFAULT_DISTANCE` with no pieces. `distance` is set to it and `target` to the origin, because the scene is always rebuilt around the origin. Azimuth and elevation are kept.

"Sew again" (`show_controls`) sets `resew`, which starts the simulation over from `Drape::new`.

## Coordinates and units

- Project geometry is mm. The scene is mm × `MM_TO_SCENE` (0.01), so 1 scene unit = 100 mm. This is a display scale, not a conversion to meters.
- Y is up. The pattern's `x` maps to X and its `y` to Z, in both modes.
- **Flat:** centered on the middle of `Project::bounds` in X and Z. The flesh side is at y = 0 and the grain side at y = thickness.
- **Sewn:** `Drape` positions are mm in its own frame (`docs/sewn-preview.md#model-coordinates`). `SewnModel::rebuild` picks `center` = the middle of the bounds in X and Z, and in Y the lowest point minus half the thickest material. `to_scene` subtracts `center` and scales, so the model stands on y ≈ 0.

## Flat mode: `flat::build_piece_mesh`

Input: a piece's `Region` (outline ring followed by hole rings, with no repeated closing point), its thickness in mm, and the scene center.

1. **Triangulate** the region in the XZ plane with `earcutr::earcut`, holes from `Region::hole_indices`. A failure is returned as `Err(String)`. `spawn_flat` logs it in English and skips the piece.
2. **Caps:** two copies of every vertex, at y = 0 (normal −Y) and y = thickness (normal +Y). For each triangle, the Y component of `(b − a) × (c − a)` picks the winding, so the bottom faces down and the top faces up whatever order earcut returned.
3. **Walls:** a quad of 4 unshared vertices per ring edge, with a flat normal `(dz, 0, −dx) / len × outward`. `outward` is +1 when `signed_area(ring) ≥ 0` (counterclockwise) for the outline, or the reverse for holes, so a hole's walls face its own center. The quad's winding follows `outward`.

Pieces that `punched_region` rejects (`ResultEmpty`, `ResultSplit`) are skipped without a log. The editor already flags them.

## Sewn mode: `sewn.rs`

### Leather: `piece_mesh`

A shell around the simulated middle surface:

- **Grain and flesh faces:** each vertex offset by ±`thickness / 2` along `Drape::vertex_normals`. The flesh copy reverses the triangle winding and the normal.
- **Walls:** one quad per `DrapedPiece::boundary` edge, normal `edge × (nₐ + n_b)`.
- The material is `double_sided` with `cull_mode: None`, so the shell renders whichever way its boundary is wound.

### Thread: `thread_mesh`, `thread_meshes`

- One mesh per seam, in `Drape::threads` order. Each mesh is built from the `Drape::thread_segments` whose `seam` matches.
- Each segment is an open tube of `THREAD_SIDES` (8) sides with an elliptical cross-section. Its width and height are `Thread::profile_mm`, so flat threads are wider than high. Its height lies along the face normal, made square to the segment axis, and falls back to a world axis when the two line up. Normals follow the ellipse's gradient.
- Segments under 1e-9 mm are skipped. Tubes have no end caps and aren't joined to each other.
- `thread_material`: `Thread::color`, with roughness and reflectance from `thread_finish(ThreadKind)`:

| `ThreadKind` | Roughness | Reflectance |
|---|---:|---:|
| `WaxedPolyester` | 0.5 | 0.5 |
| `WaxedLinen` | 0.85 | 0.35 |
| `BondedNylon` | 0.3 | 0.6 |
| `ArtificialSinew` | 0.45 | 0.5 |

### Stepping: `SewnModel::step`

It does nothing without a drape or once `Drape::is_settled`. Otherwise it calls `Drape::step` once, then replaces every piece mesh and thread mesh asset with a freshly built one.

## Camera and navigation

The camera orbits `target` on a sphere. `camera_offset`:

```
horizontal = distance · cos(elevation)
offset     = (horizontal · cos(azimuth), distance · sin(elevation), horizontal · sin(azimuth))
camera     = target + offset, looking at target, up = +Y
```

| Input over the canvas tab | Effect | Constant |
|---|---|---|
| Primary drag | `azimuth += dx · k` (wrapped to 0..τ), `elevation += dy · k` (clamped) | `ORBIT_SPEED` 0.008 rad/pt |
| Secondary or middle drag | Moves `target` in the camera plane so the scene follows the cursor. Scale: `2 · distance · tan(fov / 2) / window height`. | `FIELD_OF_VIEW` π/4 (Bevy's default) |
| Wheel (while hovered) | `distance *= exp(−scroll · k)`, clamped | `ZOOM_SPEED` 0.002 (same as the 2D canvas) |
| Double click | `reset_view`: default azimuth and elevation, `distance = fit_distance`, `target = 0` | |

| Constant (`constants.rs`) | Value |
|---|---:|
| `MM_TO_SCENE` | 0.01 |
| `MIN_DISTANCE` / `MAX_DISTANCE` | 1.0 / 60.0 |
| `DEFAULT_DISTANCE` | 3.2 |
| `DEFAULT_AZIMUTH` | 0.72 rad |
| `DEFAULT_ELEVATION` | 0.78 rad |
| `MIN_ELEVATION` / `MAX_ELEVATION` | 0.12 / 1.48 rad |
| `GROUND_OPACITY` | 0.05 |

## Controls: `show_controls`

Drawn in the Properties panel while in 3D, with text from `i18n::Label`:

1. `PreviewShows`, then a `GroupButton` for Sewn and Flat (`PreviewSewn`, `PreviewFlat`). While Sewn is chosen, an outline `Button` for `SewAgain` with the `SewAgainHint` tooltip.
2. `Camera`, then sliders for `Orbit` (azimuth 0..τ), `Tilt` (elevation) and `Zoom` (distance), within the camera limits.
3. `PreviewNavigationHint` as weak text.

## Theme

`apply_theme` follows `Preferences::theme` and sets the clear color and the ground's base color. The values are 3D scene colors, not egui colors, so they're written in `mod.rs` and not in `ui_components::palette::Palette`.

| | Dark | Light |
|---|---|---|
| Clear color | `#2B2C2F` | `#D6D9DE` |
| Ground (before `GROUND_OPACITY`) | (0.13, 0.15, 0.18) | (0.72, 0.74, 0.77) |

## Invariants

- The preview never writes to `Project`. Its only writes are its own entities, mesh and material assets, `SewnModel`, `PreviewSettings` and `ClearColor`.
- Rebuilds happen only in `ViewMode::Preview3d`, and only on a content change, a mode change or Sew again.
- Every rebuild centers the scene on the origin and resets `target` there. Panning moves `target` and never the scene.
- The Sewn mode never infers seams: it shows exactly what `Drape::new` builds from `Project.seam_links`.
- Pieces whose result is invalid are skipped in both modes and never cause an error.
- Mesh building is in mm until `to_scene` (Sewn) or the `MM_TO_SCENE` products (Flat). No other scale is applied.

## Review findings

Found while reviewing for this spec. None are fixed yet.

| # | Severity | Where | Finding |
|---|---|---|---|
| 1 | Likely bug | `sewn.rs:146-150` | **The thread tube is wound inward.** With `side = axis × up`, the quad `[a, b, d]` has a geometric normal of `(b − a) × (d − a) = L · (up × axis) = −side` at angle 0, pointing into the tube. Checked by hand with axis = X and up = Y: a = (0, 0, r), b = (0, r, 0), d = (L, r, 0) give a normal of (0, −rL, −rL). `thread_material` keeps Bevy's default back-face culling, so the near wall is culled and the viewer sees the inside of the far wall, lit by vertex normals that point away from the camera. The thread then looks darker or flatter than its material. Fix: emit `[a, d, b, a, c, d]`, or set `cull_mode: None` on the thread material. Confirm in the app. A unit test on `thread_mesh` comparing the winding normal with the vertex normal would catch this. |
| 2 | Minor | `editor/panel_viewer.rs:118` → `controls.rs` | `ViewControls::Preview(self.settings)` turns `&mut ResMut<PreviewSettings>` into `&mut PreviewSettings` through `DerefMut`, which marks the resource changed on every frame the Properties panel is drawn in 3D. `update_camera` then rewrites the transform every frame. That's cheap, but it contradicts the `navigate` doc comment ("only written on input"). Pass the `ResMut` and deref only when a control reports a change. |
| 3 | Minor | `mod.rs:169-174` | Every rebuild resets `distance` and `target`, including Sew again, a Flat ↔ Sewn switch and an undo made in 3D. The user's zoom and pan are lost each time. This may be intended for content changes, but Sew again and a mode switch could keep the camera. |
| 4 | Perf | `sewn.rs:247-268` | `step` rebuilds and replaces every mesh asset every frame until settled. That reallocates all buffers, re-uploads them in full, and recomputes vertex normals. `thread_meshes` also filters all segments once per seam (O(seams × segments)). Updating the position and normal attributes in place and grouping segments once would cut the cost. |
| 5 | Nit | `flat.rs:46-48` | Three `Vec` allocations per triangle (`to_vec()`) just to read two coordinates. Index `flat_points` directly. |
| 6 | Nit | `flat.rs` | Flat pieces all lie at y = 0. Pieces that overlap in the pattern z-fight on their caps. |
| 7 | Nit | `sewn.rs:132-150` | Thread tubes have no end caps and separate segments aren't joined, so gaps can show at sharp bends and at the seam ends. |
| 8 | Nit | `navigation.rs:15` | `FIELD_OF_VIEW` duplicates Bevy's default FOV. If the camera's `Projection` ever changes, panning stops tracking the cursor. Read the FOV from the camera's `Projection` instead. |
| 9 | Nit | `mod.rs:59` | The startup camera transform (2.2, 2.8, 3.4) doesn't match the defaults (about (1.71, 2.25, 1.50)). It's harmless, because `update_camera` overwrites it on the first frame, but it's a second set of magic numbers. Use `PreviewSettings::default()` with `camera_offset`. |
| 10 | Nit | `mod.rs:62-68` | The single point light uses Bevy's default range and doesn't follow `fit_distance`, so very large projects can fall partly outside it. |

## Tests

The module has no tests, and the meshes and controls are checked by hand in the app (`docs/tests.md`). Several pure functions could be tested without a window:

- `flat::build_piece_mesh`: the cap and wall normals agree with the winding, and a hole's walls face inward.
- `sewn::thread_mesh`: the winding normal agrees with the vertex normal (would catch finding 1), and the width and height match `profile_mm`.
- `PreviewSettings::camera_offset`: its length equals `distance`, and elevation is respected.

## Code index

| Symbol | Location | Notes |
|---|---|---|
| `PreviewPlugin`, `Plugin::build` | `mod.rs:25-41` | Resources and systems |
| `PreviewPiece`, `PreviewCamera`, `Ground` | `mod.rs:43-50` | Marker components (`PreviewPiece` is `Clone` so `SewnModel::rebuild` can take it as a tag) |
| `setup_scene` | `mod.rs:52-79` | Camera, light, ground |
| `clear_color`, `ground_color` | `mod.rs:81-96` | Theme colors |
| `apply_theme` | `mod.rs:98-117` | `Local<Option<Theme>>` guard |
| `Built` | `mod.rs:119-125` | Rebuild bookkeeping |
| `rebuild_preview` | `mod.rs:127-175` | Lifecycle and framing |
| `spawn_flat` | `mod.rs:177-213` | Flat entities, returns the size in mm |
| `step_sewn` | `mod.rs:215-225` | |
| `PreviewMode` | `mode.rs:1-9` | `Flat`, `Sewn` (default) |
| `PreviewSettings`, `Default` | `settings.rs:7-33` | |
| constants | `constants.rs:3-13` | |
| `ORBIT_SPEED`, `ZOOM_SPEED`, `FIELD_OF_VIEW` | `navigation.rs:10-15` | |
| `PreviewSettings::reset_view`, `camera_offset` | `navigation.rs:17-35` | |
| `update_camera` | `navigation.rs:37-49` | |
| `navigate` | `navigation.rs:51-91` | |
| `show_controls` | `controls.rs:10-41` | |
| `build_piece_mesh` | `flat.rs:10-112` | Triangulation 16-24, caps 26-65, walls 67-102 |
| `THREAD_SIDES` | `sewn.rs:13` | |
| `SewnModel` | `sewn.rs:15-24` | |
| `to_scene` | `sewn.rs:26-32` | |
| `piece_mesh` | `sewn.rs:34-81` | Faces 45-59, walls 60-71 |
| `thread_finish`, `thread_material` | `sewn.rs:83-103` | |
| `thread_mesh` | `sewn.rs:105-160` | Frame 115-130, ring 132-144, quads 145-150 |
| `thread_meshes` | `sewn.rs:162-174` | |
| `SewnModel::rebuild` | `sewn.rs:177-240` | Center 190-202, pieces 204-222, threads 223-235 |
| `SewnModel::clear` | `sewn.rs:242-244` | |
| `SewnModel::step` | `sewn.rs:246-268` | |

## Where to change what

| Task | Change |
|---|---|
| Add a preview mode | A `PreviewMode` variant, an arm in `rebuild_preview` returning the size in mm, an option in `show_controls` with a `Label` in both catalogs, and a `step_sewn`-style system if it animates |
| Change how leather or thread looks | `spawn_flat` and `SewnModel::rebuild` materials, `thread_finish` |
| Change a mesh | `flat::build_piece_mesh`, `sewn::piece_mesh`, `sewn::thread_mesh` |
| Change camera behavior or limits | `navigation.rs` and `constants.rs`. Keep the sliders in `show_controls` on the same ranges. |
| Change what triggers a rebuild | `Built` and `rebuild_preview`. Keep the rule that it runs only in 3D. |
| Change the simulation | The library's `drape` module, `docs/sewn-preview.md` |
