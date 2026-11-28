# Background images: implementation plan

## Context

Users trace patterns from photos and scans, and the editor can't show them. The design is in `docs/background-image.md`, which is the behavior spec. The mockups are the four *Background Image…* boards in `leathercraft-design/main.bl`. This plan turns that design into ordered, checkable steps. It doesn't restate the spec: where they differ, the spec wins.

There are no images or textures anywhere in the code yet. `image` 0.25 is in `Cargo.lock` only through Bevy. The 2D canvas is drawn entirely with the egui painter (`src/editor/canvas.rs`), which makes a textured quad the natural fit.

## Decisions

- **Stored in the project.** `Project.background_images: Vec<BackgroundImage>` uses `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so `PROJECT_VERSION` stays 2 and older files load unchanged.
- **Linked, not embedded.** The file keeps a path, relative to the project file when possible.
- **Decoding stays out of the library.** The library holds data, validation, path helpers and the frame-method UV math. Decoding and textures live in the binary.
- **One `Update` event carries the whole image.** The panel and the canvas both send a changed copy. History merges consecutive updates of one image.
- **Background edits don't rebuild the 3D preview.** They return a new `ProjectChange::BackgroundEdited`, and `apply_project_events` doesn't call `set_changed` for it. `preview::rebuild_preview` is the only reader of `ActiveProject` change detection; verified with `rg "is_changed\(\)" src`.
- **Images never steal clicks from pieces.** They are picked in the panel, or by clicking inside the frame of an image that is already selected.

## Phase 1: Library (`cargo test --lib --no-default-features`)

### 1.1 `src/background.rs` (new, `pub mod background;` in `src/lib.rs`)

- `BackgroundImage { id, path: PathBuf, visible, locked, opacity: f32, depth: Depth, frame: FrameMethod, size: (f64, f64), offset: Point, rotation: f64, scale: f64, flip_x, flip_y }`. Serde defaults are as in the spec: `visible = true`, `scale = 1.0`, `frame = Fit`.
- `enum Depth { #[default] Back, Front }` and `enum FrameMethod { Stretch, #[default] Fit, Crop }`, both serialized in lowercase.
- `BackgroundImage::new(id, path, picture_px: [u32; 2], view_center: Point, view_size_mm: (f64, f64))`. It fits the picture inside 80 % of the view at its aspect ratio, with opacity 0.5, depth Back and frame Fit.
- `drawn_size(&self) -> (f64, f64)`, which is `size × scale`.
- `corners(&self) -> [Point; 4]` gives the box corners in mm, in the order TL, TR, BR, BL, after rotation about `offset`. The canvas and hit-testing use it.
- `uv_rect(frame, picture_px, box_mm) -> (Rect in 0..1 UV, Rect in 0..1 of the box)`. It is a pure function:
  - `Stretch` maps the full UV to the full box.
  - `Fit` maps the full UV to a centered sub-box.
  - `Crop` maps a centered sub-UV to the full box.
  - Flips swap the UV coordinates at draw time, not here.
- `contains(&self, p: Point) -> bool` tests a point against the rotated box, for clicking inside a selected image.
- Path helpers:
  - `stored_path(image: &Path, project_file: Option<&Path>) -> PathBuf`: relative when the image is under the project's folder, absolute otherwise.
  - `resolve_path(stored: &Path, project_file: Option<&Path>) -> PathBuf`.
- Tests:
  - `uv_rect` for wide and tall pictures in each mode.
  - `corners` at 0° and 90°.
  - `contains` for a rotated image.
  - Path round-trip for relative, absolute, and no project path.

### 1.2 `src/model.rs`

- Add the field to `Project` beside `seam_links` and initialize it in `Project::new`.
- Add `fn validate_background_images(&self) -> Result<(), Message>` and call it from `validate` (around line 364).
  - `opacity` must be finite and within 0..=1.
  - `size.0`, `size.1` and `scale` must be finite and > 0.
  - The drawn size and `offset` must be within `MAX_COORDINATE_MM`.
  - `rotation` must be finite.
  - Ids must be unique.
  - It never touches the file system.
- Add `fn next_background_id(&self) -> u64`, following the pattern of the existing piece and seam-link ids.
- Tests: each rejection, and that an older JSON without the field loads.

### 1.3 `src/events.rs`

- Add the `ProjectEvent` variants:
  - `AddBackgroundImage(BackgroundImage)`, which rejects a duplicate id.
  - `RemoveBackgroundImage { id }`
  - `UpdateBackgroundImage { image: BackgroundImage }`, which finds the image by `image.id` and replaces it.
  - `ReorderBackgroundImages { order: Vec<u64> }`, which must be a permutation of the existing ids.
- Add `ProjectChange::BackgroundEdited`, returned by all four. An unknown id returns `Err(Message::BackgroundImageNotFound)`.
- Tests:
  - Add, update and remove.
  - A bad reorder is rejected and leaves the project unchanged.
  - An update with opacity 2.0 is rejected by validation.

### 1.4 `src/history.rs`

- Add `MergeKey::BackgroundImage(u64)`, given to `UpdateBackgroundImage` only.
- Add `describe` arms with the entry labels from the spec. Remove names the file.
- Tests: two updates of one id share one entry, and updates of two different ids make two.

### 1.5 `src/persistence.rs`

- `save_project`: before serializing, rewrite each image's `path` with `stored_path(.., Some(path))`. Do it on a clone, since the project is borrowed.
- `load_project`: after parsing, resolve each `path` with `resolve_path(.., Some(path))`. In memory, paths are always absolute.
- Tests:
  - A round-trip with two images, one under the project folder and one outside it. The JSON holds a relative and an absolute path, and the reloaded project holds two absolute paths.
  - A missing image file doesn't fail the load.

### 1.6 `src/exports/`

There is no code change: exports only walk `pieces`. Add one test showing that SVG and DXF output is identical with and without background images.

### 1.7 `src/i18n/` (`mod.rs`, `en.rs`, `uk.rs`)

- **Labels:** `BackgroundImages`, `AddBackgroundImage`, `ImageSource`, `ImageDisplay`, `ImagePlacement`, `Opacity`, `Depth`, `DepthBack`, `DepthFront`, `FrameMethod`, and `FrameMethodName(FrameMethod)` with a helper in each catalog, following the `Label::Operation` pattern. Also `Offset`, `Scale`, `Flip`, `Calibrate`, `ImageFileType`, `ImageNotFound`, `BackgroundEmptyHint`, `ShowImage`, `HideImage`, `LockImage`, `UnlockImage`, `RemoveImage`, `Browse`, `Reload`. Reuse `Label::Rotation` (it is at `i18n/mod.rs:146`) and the existing width and height labels.
- **Messages:**
  - `ImageLoadFailed { path, reason }`
  - `BackgroundImageNotFound`
  - `CalibrateFirstPoint`, `CalibrateSecondPoint`, `CalibrateDistance` (status bar)
  - History: `AddedBackgroundImage { name }`, `RemovedBackgroundImage { name }`, `EditedBackgroundImage { name }`, `ReorderedBackgroundImages`
- The Ukrainian texts are the ones on the mockups: Фонові зображення, Додати зображення, Джерело, Відображення, Розміщення, Непрозорість, Глибина, Позаду, Попереду, Вписування, Розтягти, Вписати, Обрізати, Зсув, Масштаб, Віддзеркалити, Калібрувати…, не знайдено.
- `cargo check --lib --no-default-features` lists any missing translation arm.

**Phase 1 is done when** the library tests pass and `cargo clippy` is clean, before any UI is written.

## Phase 2: Binary plumbing

### 2.1 `Cargo.toml`

- Add `image = { version = "0.25", default-features = false, features = ["png", "jpeg", "webp", "tiff"] }` under the `desktop` feature list (lines 18 and 24–34). Match the version already in `Cargo.lock` so no second copy is built.

### 2.2 `src/document.rs`, `apply_project_events` (around line 54)

- Skip `active.set_changed()` for `ProjectChange::BackgroundEdited` as well as `Saved`. Update the comment there: background edits don't touch what the preview builds.
- History restores still call `set_changed`. That is acceptable, since it only marks the preview stale and the preview rebuilds only while the 3D view is shown.

### 2.3 `src/background_textures.rs` (new binary module, registered in `main.rs`)

- `#[derive(Resource, Default)] struct BackgroundTextures { entries: HashMap<PathBuf, Entry> }`, where `enum Entry { Loaded { texture: egui::TextureHandle, size_px: [u32; 2], bytes: u64 }, Failed(Message) }`.
- `fn get(&mut self, ctx: &egui::Context, path: &Path) -> &Entry`. It loads on first use: `image::open`, then downscale so the longer side is at most 4096 px, then `ColorImage::from_rgba_unmultiplied`, then `ctx.load_texture(path, image, TextureOptions::LINEAR)`.
- `fn reload(&mut self, path)` drops the entry, so the next `get` reloads it.
- `fn retain(&mut self, project)` drops textures no image references. It runs once a frame from `editor_ui`.
- Loading is synchronous for the first version: a 12 MP JPEG takes about 100–200 ms once, on Add or Open. Note in the module that it could move to a thread if that stalls too long.

### 2.4 `src/editor/editor_state_struct.rs`

- `selected_background: Option<u64>`. Selecting a piece clears it, and selecting an image clears the piece selection.
- `background_drag: Option<BackgroundDrag { id, kind: Move | Scale(corner) | Rotate, start: BackgroundImage, anchor: Point }>`.
- `calibration: Option<Calibration { id, first: Option<Point>, second: Option<Point>, distance_text: String }>`.
- `on_project_change` / `on_history_restore`: clear `selected_background` and `calibration` if their image no longer exists.

## Phase 3: Canvas (`src/editor/canvas.rs`)

### 3.1 Drawing

- `fn draw_background_images(painter, view: Viewport, project, textures, depth: Depth)`. For each visible image of that depth, in list order:
  1. Get the texture. If loading failed, skip it (the panel shows the warning).
  2. Use `uv_rect` for the sub-box and UV, `corners` for the screen positions through `view.to_screen`, and swap U or V for `flip_x` or `flip_y`.
  3. Build an `egui::Mesh::with_texture(texture.id())` with 4 vertices and 2 triangles, with color `Color32::WHITE.gamma_multiply(opacity)`. That is a tint of document data, which `docs/design.md` allows.
  4. `painter.add(Shape::mesh(mesh))`.
- Call sites in the draw order (around lines 149–169):
  - `Back`: right after `painter.rect_filled(.. surface.canvas)`, before `draw_grid`.
  - `Front`: after `draw_dimensions` and before the drafts, so in-progress drawing stays on top.
- `draw_background_selection(painter, view, image, state)`, after `draw_snap_feedback` and before the rulers:
  - a dashed `accent.base` outline along `corners`
  - four 8 px corner squares filled with `surface.canvas`
  - a stem and a 5 px circle 28 px above the top edge's midpoint, in the image's rotated frame
  - nothing for a locked image

### 3.2 Interaction (Select tool only)

- **Hit order on press:** the handles of the selected image, then the inside of the selected image, then the existing `piece_at` / child hit-testing. Images are never hit-tested unless already selected and unlocked.
- **Move:** the new `offset` is `start.offset + (world − anchor)`, snapped with `snap_offset` against grid targets only.
- **Scale from a corner:** keep the opposite corner fixed. Scale uniformly when `start.size` has its aspect locked (the default); otherwise scale freely, or uniformly with Shift. Write the result to `size`, not `scale`.
- **Rotate:** `rotation = start.rotation + angle(anchor → world about offset)`, in 15° steps with Shift.
- While dragging, draw from the in-progress copy in `EditorState`. On release, send one `UpdateBackgroundImage` (history merges anyway).
- **Esc** cancels a drag and restores the start state. **Delete** with an image selected and no piece selected sends `RemoveBackgroundImage`.

### 3.3 Calibration

- It starts from the panel's **Calibrate…** button: `state.calibration = Some(Calibration { id, .. })`.
- While it is active, canvas clicks don't select anything:
  - The first click sets `first` and shows `CalibrateFirstPoint` in the status bar.
  - The second click sets `second` and opens a small `egui::Window` anchored at the cursor, with a `NumberInput` in project units and OK / Cancel.
- On OK, apply `k = entered_mm / distance_mm(first, second)` to `size` (both axes) and to `offset` about the image center, then send `UpdateBackgroundImage`. Esc or Cancel ends calibration without an event.
- Draw the two points and the line between them in `accent.base`, with the live distance as a label.
- Points snap to vertex targets only when snapping is on, so the user can click a ruler mark in a scan.

## Phase 4: Panel

### 4.1 Shared widgets

- `src/ui_components/slider/mod.rs` (new):
  - `Slider::new(&mut f32, 0.0..=1.0).percent()`, implementing `egui::Widget`.
  - It looks like `NumberInput`: same height, fill and stroke from `palette.input`, with an `accent.container` fill bar up to the value and the value centered in mono.
  - Dragging anywhere sets the value; Ctrl+drag gives fine steps; a double-click turns it into a text field.
  - Colors go in a new `palette/slider.rs`, with values for DARK and LIGHT that match the mockups.
  - Add it to the component table in `docs/ui-components.md`.
- `src/panels/properties/fields.rs`: make `section_header`, `divider` and `angle_field` `pub(crate)`, or move them to `src/panels/fields.rs`, and update the Properties imports.
- `src/ui_components/icons/mod.rs`: add glyphs for image, eye, eye-off, lock, unlock, folder, reload, link and ruler, following `docs/icons.md`. Make `icon_button` non-`dead_code`.

### 4.2 `src/panels/background_images/mod.rs` (new, registered in `src/panels/mod.rs`)

- Signature: `pub fn show_panel(ui, project: &Project, textures: &mut BackgroundTextures, state: &mut EditorState, language) -> Option<ProjectEvent>`.
- The layout follows the dark and light boards from top to bottom.
  - **Add image** (full-width secondary `Button`): `rfd::FileDialog::new().add_filter(Label::ImageFileType, &["png","jpg","jpeg","webp","tif","tiff"]).pick_file()`. Then `textures.get` for the pixel size, `BackgroundImage::new(project.next_background_id(), path, px, view_center, view_size)`, and `AddBackgroundImage`. Select the new image.
  - **Entry per image**, rendered with `ui.push_id(image.id, ..)`.
    - The header row has a chevron, a 22×16 thumbnail (the same texture, drawn through `ui.image`), and the file name, truncated. Right to left: eye, lock and ✕ `icon_button`s. A failed texture shows the warning glyph and the name with `ImageNotFound` in `status.warning` on `status.warning_container`.
    - Clicking the header toggles open; the open state is kept in egui memory, as in `panels/layout/mod.rs` `root_row`. Clicking the name also selects the image on the canvas.
    - The body has three sections with `section_header` and `divider`:
      - **Source:** a read-only path field shortened from the left, then Browse (`rfd` pick, which keeps the placement and sends `Update` with the new path, then `textures.reload`) and Reload (`textures.reload`). Under them, an info line with `W × H px · size`.
      - **Display:** Opacity `Slider`, Depth `GroupButton`, then Frame method on its own row with a full-width `GroupButton`.
      - **Placement:** Calibrate… in the header row, W and H `NumberInput`s in project units with a link toggle between them, Offset X and Y, Rotation `angle_field`, Scale `NumberInput` (0.01–100, 3 decimals), and Flip `Checkbox` X and Y.
    - The aspect-lock toggle is interface state, kept per image id in egui memory and on by default, so it is not in the file.
    - Each widget edits a local clone. If the clone differs from the stored image, return `UpdateBackgroundImage { image: clone }`.
  - **Empty state:** the Add button, then the dashed hint box with `BackgroundEmptyHint`.
- Reordering (drag in the list) isn't in the first version. `ReorderBackgroundImages` exists for the follow-up.

### 4.3 Dock registration

- `src/ui_components/dock/mod.rs`:
  - Add `PanelTab::BackgroundImages` to the enum (lines 14–22), to `CLOSEABLE` and to `label()`.
  - Leave it out of `default_state()`. It opens from the Window menu (`toggle`), so saved layouts keep working.
- `src/editor/panel_viewer.rs`: add `title()` (image glyph), `ui()` (calls `show_panel` and `send`s its result) and `scroll_bars()` (vertical) arms.
- `src/editor/ui.rs` `window_menu` (around line 490): the entry appears through `CLOSEABLE`. Check that it does.

## Phase 5: Docs

- `docs/background-image.md`: change the title to drop "(design, not implemented)", and fix any spec detail the implementation changed.
- `docs/project-index.md`:
  - Rows for `src/background.rs`, `src/background_textures.rs`, `src/panels/background_images/` and `src/ui_components/slider/`.
  - New symbols in the canvas section.
  - Change the Docs table status to Current.
- `CLAUDE.md` Architecture: one bullet for `background` in the library list. Add "background images" to the list of things exports and the 3D preview ignore.
- `docs/history.md`: the new merge key.

## Verification

1. `cargo test --lib --no-default-features`: the new tests in `background`, `model`, `events`, `history`, `persistence` and `exports` pass.
2. `cargo fmt`, `cargo clippy`, then `cargo check --bin leathercraft`.
3. Manual run (`cargo run`), checked in both themes:
   - Window → Background images. The empty state matches the *Empty* board.
   - Add a JPEG. It appears centered and fitted at 50 % behind the grid. The panel matches the *Dark* and *Light* boards.
   - Each field changes the canvas live:
     - Opacity: drag the slider.
     - Depth Front: the image is over the pieces, and clicking a piece through it still selects the piece.
     - Stretch, Fit and Crop on a non-matching W/H.
     - Flips.
     - Rotation.
   - Select the image, then move, scale and rotate it. Each drag is one undo entry.
   - Lock it: the handles disappear and canvas clicks pass through.
   - Calibrate on a scan with a ruler: after entering 100 mm between two marks 100 mm apart, the drawn ruler matches the canvas ruler.
   - Save, move the project folder together with its images, then reopen: the images load (relative path). Rename an image file and reopen: the entry shows "not found" and the project still opens. Browse fixes it.
   - Export SVG and DXF: no images in the output.
   - Switch to the 3D view, then drag an image in 2D and switch back: the preview doesn't rebuild. This is visible in the log or by timing.
   - Undo and redo across add, edit and remove. The History panel shows the entry names in both languages.
