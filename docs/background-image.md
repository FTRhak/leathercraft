# Background images

A background image is a photo or scan shown on the 2D canvas to trace a pattern from: an existing product, a hand drawing, a marked hide. A project can hold several. Each has its own position, size in mm, opacity and depth. They are saved in the `.leathercraft` file but are never part of the pattern: exports and the 3D preview ignore them.

The panel is modeled on Blender's Background Images panel. It differs in one way: Blender fits an image to the camera, while a pattern needs true scale, so here an image has a size in mm and a **Calibrate** step.

The mockups are in `leathercraft-design/main.bl`:

| Board | Shows |
|---|---|
| *Background Images Panel — Фонові зображення (Dark)* | The panel with one expanded entry, one hidden and locked entry, and one whose file is missing |
| *Background Images Panel — Фонові зображення (Light)* | The same panel in the light theme |
| *Background Image — Canvas (Dark)* | An image under the grid at 50 %, a piece traced over it, the selection frame with scale and rotate handles, and notes on behavior |
| *Background Images Panel — Empty (Dark)* | The panel with no images |

## For users

| Action | How |
|---|---|
| Add an image | **Window → Background images**, then **Add image**, and pick a PNG, JPEG, WebP or TIFF file. It lands centered on the view, fitted to it, at 50 % opacity, behind everything. |
| Show or hide it | The eye in the entry header |
| Lock it | The lock in the entry header. A locked image can't be selected or moved on the canvas, only edited in the panel. |
| Remove it | The ✕ in the entry header. Undo brings it back. |
| Set true scale | **Calibrate…**, click two points on the image whose distance you know, then enter that distance in mm. The size is recomputed and the aspect ratio is kept. |
| Select it | Click its name in the panel. A click on the canvas inside the selected image keeps it; a click on a piece or on empty canvas selects that instead. |
| Move, scale or rotate it on the canvas | With the Select tool, on a selected, unlocked image: drag the body to move it, a corner to scale it (in proportion; Shift scales freely), or the circle above the top edge to rotate it (Shift: 15° steps). Esc cancels a drag, else deselects the image. |
| Delete it | Del with the image selected and no piece selected, or the ✕ |
| Put it over the pieces | **Depth → Front** |
| Mirror it | **Flip X / Y**, for example a scan taken from the back of the leather |

- The entry header shows a caret, a thumbnail, the file name, and the eye, lock and ✕ buttons. The caret (or a double click on the name) collapses or expands the entry; the state is kept per image in egui memory.
- A missing or unreadable file keeps its entry, shown in the warning color with "— не знайдено" / "— not found" after the name. **Reload** retries, and **Browse** picks a new file while keeping the placement.
- Images never catch clicks meant for pieces. With **Depth → Front** the image is drawn over the pieces, but clicks still go to the pieces under it. A drag starting on a piece moves the piece even inside the selected image; only the image's handles come before pieces.
- Phone photos are turned upright by their EXIF orientation when loaded.
- Calibration snaps its two points to the pieces' vertices and guides while snapping is on, and scales the image about the first point, so that point stays where it was clicked. Hotkeys are off while it runs.

## Panel fields

| Field | Widget | Stored as |
|---|---|---|
| File | Path field (read-only, shortened from the left), **Browse** and **Reload** icon buttons, and an info line with pixel size and file size | `path` |
| Opacity | New `Slider`: a number field with a fill bar, 0–100 % | `opacity` 0.0–1.0 |
| Depth | `GroupButton`: **Back** / **Front** | `depth` |
| Frame method | `GroupButton`: **Stretch** / **Fit** / **Crop**, on its own row under its caption because the Ukrainian labels are long | `frame` |
| Width / Height | `NumberInput`s in project units, with a link toggle that keeps the proportions (on by default; kept in egui memory, not in the file) | `width`, `height` (mm) |
| Calibrate… | Outline button in the *Placement* section header, disabled while the image is locked | changes `width`, `height`, `offset` |
| Offset X / Y | `NumberInput`s in project units | `offset` (mm) |
| Rotation | `angle_field` (degrees shown, radians stored) | `rotation` |
| Scale | `NumberInput`, ×, 0.01–100 | `scale` |
| Flip | Two `Checkbox`es, X and Y | `flip_x`, `flip_y` |

The frame method decides how the picture fills its W × H box when the box's aspect ratio differs from the picture's (`background::frame_rects`). **Stretch** distorts the picture to fill the box. **Fit** shows the whole picture inside the box. **Crop** fills the box and cuts off the overflow. The drawn size is `width × scale` by `height × scale`. `scale` is there so that a calibrated size can be adjusted temporarily without being lost.

## The model

```rust
pub struct BackgroundImage {
    pub id: u64,                 // unique within the project
    pub path: PathBuf,           // relative to the project file when it is under the same folder
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    pub opacity: f32,            // 0.0..=1.0
    #[serde(default)]
    pub depth: Depth,            // Back | Front
    #[serde(default)]
    pub frame: FrameMethod,      // Stretch | Fit (default) | Crop
    pub width: f64,              // mm, the box before `scale`
    pub height: f64,
    pub offset: Point,           // mm, center of the box
    #[serde(default)]
    pub rotation: f64,           // radians, about the center
    #[serde(default = "one")]
    pub scale: f64,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
}
```

- It lives in `src/background.rs` in the library, with no image decoding, beside `Depth`, `FrameMethod`, `frame_rects`, `stored_path` and `resolve_path`. `BackgroundImage` has `new` (fitted to the view), `name`, `drawn_size`, `corners`, `contains`, `quad` (the textured quad after frame method and flips) and `scaled_about` (calibration). `Project` gets `#[serde(default, skip_serializing_if = "Vec::is_empty")] background_images: Vec<BackgroundImage>`, the same way `seam_links` was added. Older files load unchanged, so `PROJECT_VERSION` stays the same. The list order is the draw order within a depth.
- All geometry is in mm, like everything else. `Project.units` only changes how the fields display.
- `path` is saved relative to the project file when the image is in the project's folder or below it, and absolute otherwise. `save_project` writes the paths through `Project::with_stored_image_paths`, and `load_project` resolves them with `Project::resolve_image_paths`, so in memory they are always as given. Before a project's first save, paths are absolute, and they become relative on save when they can.
- Images are linked, not embedded. Embedding them would make files large and diffs useless. A later "Pack images" could add an embedded variant.

### Validation

`Project::validate` checks each image:

- `opacity` is finite and within 0..=1.
- `width`, `height` and `scale` are finite and > 0, and the drawn size is within `MAX_COORDINATE_MM`.
- `offset` is within `MAX_COORDINATE_MM`, and `rotation` is finite.
- Ids are nonzero and unique (`InvalidBackgroundImageId`; the others fail with `InvalidBackgroundImage`).

It does **not** check that the file exists or decodes. A project with a moved photo is still a valid project, so `load_project` must not fail on it. The missing file is a display state in the binary.

### Events

| Event | History entry | Merges |
|---|---|---|
| `AddBackgroundImage { image }` | "Added background image 'name'" | no. The image's `id` is ignored and a new one given; returns `ProjectChange::BackgroundAdded(id)`, which selects it |
| `RemoveBackgroundImage { id }` | "Removed background image 'name'" | no |
| `UpdateBackgroundImage { image }` | "Changed background image 'name'" | yes, with consecutive updates of the same id (`MergeKey::BackgroundImage(id)`), so dragging the slider is one entry |
| `ReorderBackgroundImages { order }` | "Reordered background images" | no. `order` must hold each id once, or `BackgroundImageNotFound` |

A single `Update` carrying the whole image keeps the event list short. The panel and the canvas both build the changed copy and send it; a canvas drag sends one `Update` on release. Calibration is an `Update` of `width`, `height` and `offset`. An unknown id fails with `BackgroundImageNotFound`.

Every background event returns `BackgroundAdded` or `BackgroundEdited`, for which `ProjectChange::affects_pattern` is false. `apply_project_events` then doesn't call `set_changed` on `ActiveProject`, so the 3D preview, its only change-detection reader, doesn't rebuild. Undo and redo still mark it changed, which only rebuilds while the 3D view is shown.

### Exports and preview

- `to_svg` and `to_dxf` ignore background images.
- The 3D preview ignores them.
- They round-trip through `.leathercraft` only.

## In the binary

| Place | Work |
|---|---|
| `Cargo.toml` | `image` as a direct dependency under `desktop` (it is already in the lock file through Bevy), with `png`, `jpeg`, `webp`, `tiff` |
| `src/background_textures.rs` | `BackgroundTextures`, mapping a path to `Picture::Loaded { texture, size_px, kilobytes }` or `Picture::Failed(Message)`. It decodes on first use (synchronously, turned upright by EXIF), again after **Reload**, and `retain_used` drops pictures no image uses. Pictures are downscaled once to fit 4096 px and the graphics device's texture limit (2048 px on some GPUs); one that still doesn't fit becomes `Failed`, shown as a warning, rather than a panic. It is kept in `EditorState.background_textures`, being interface state. |
| `src/editor/background.rs` | `draw_images(painter, view, project, state, depth)`: a textured `egui::Mesh` quad per visible image from `BackgroundImage::quad`, tinted `WHITE.gamma_multiply(opacity)`. `canvas` calls it with `Back` right after the canvas fill and with `Front` after the dimensions. `draw_selection` (dashed frame, corner handles, rotate handle), `begin_drag` / `update_drag` / `finish_drag` / `cancel`, and calibration: `handle_calibration`, `draw_calibration`, `calibration_window`. |
| `src/editor/canvas.rs` | Calls the above: `handle_select` lets the selected image's handles take a press first, then pieces, then the image's body; while `calibration` is set, canvas clicks go to calibration whatever the tool. It also records `view_area`, the view a new image is fitted to. |
| `src/editor/editor_state_struct.rs` | `selected_background`, `background_drag`, `calibration`, `background_textures`, `view_area`; `select_background`, `start_calibration`. Selecting a piece clears `selected_background`, and edits or undo drop a selection, drag or calibration of an image that is gone. |
| `src/editor/ui.rs` | Del removes the selected image; hotkeys are off during a calibration or an image drag; `retain_used` once a frame |
| `src/panels/background_images/mod.rs` | `show_panel(ui, project, PanelState { selected, textures, view_area }, language) -> (Option<ProjectEvent>, Option<Request>)`, where `Request` is `Select`, `Calibrate` or `Report` (a file that couldn't be decoded isn't added) |
| `src/ui_components/dock/mod.rs` | `PanelTab::BackgroundImages`, closeable, not in the default layout: it is opened from the Window menu |
| `src/editor/panel_viewer.rs` | `title`, `ui` (disabled in the 3D view, like Align) and `scroll_bars` arms |
| `src/ui_components/slider/` | `Slider::new(&mut f32, range).percent()`: a field as tall as a number field, filled up to its value; a click or drag sets the value, Ctrl drags ten times slower. Typing a value isn't supported yet. |
| `src/panels/properties/fields.rs` | `fields` is `pub(crate)`, with `param_row`, `section_header`, `divider`, `note`, `angle_field`, `ROW_GAP` and `FIELD_WIDTH`, so this panel reuses them |
| `src/ui_components/icons/mod.rs` | Painted icons `paint_eye`, `paint_image`, `paint_folder`, beside the existing `paint_link`, `paint_reset`, `paint_warning`, `paint_close` and the lock glyphs; `icon_button` is in use |

### Palette

`Palette.background` (`palette/background.rs`) holds the colors this feature adds, with DARK and LIGHT values from the mockups: `entry_fill`, `entry_border`, `entry_divider`, `slider_fill`, `slider_edge`, `frame` (the selection frame, handles and calibration marks, the accent hue), `handle_fill`, `missing_border` and `empty_border`. A missing file also uses `status.warning` and `status.warning_container`; the rest reuses `input`, `text`, `border` and `group_button`.

### i18n

- **Labels:** `BackgroundImages` (panel title and Window menu), `AddBackgroundImage`, `ImageSource`, `ImageDisplay`, `ImagePlacement`, `Opacity`, `ImageDepth`, `ImageDepthOption(Depth)`, `FrameMethodField`, `FrameMethod(FrameMethod)`, `Offset`, `Scale`, `Flip`, `Calibrate`, `CalibrateHint`, `CalibrateDistance`, `ImageFileType` (dialog filter), `ImageNotFound`, `BackgroundEmptyHint`, `ShowImage`, `HideImage`, `LockImage`, `UnlockImage`, `RemoveImage`, `BrowseImage`, `ReloadImage`, `KeepAspect`. The existing `Rotation`, `Width`, `Height`, `Ok` and `Cancel` are reused.
- **Messages:** the four history entries, `BackgroundImageNotFound`, `InvalidBackgroundImage { name }`, `InvalidBackgroundImageId { name }`, `ImageLoadFailed { path, detail }`, `ImageInfo { width, height, kilobytes }` (shown in KB, or MB from 1 MB), `CalibrateFirstPoint` and `CalibrateSecondPoint` (status bar).

## Tests

Library tests, inline, run without Bevy:

- `background`: fitting a new image to the view, `frame_rects` for wide and tall pictures in each mode, `corners` with rotation and scale, `contains` on a rotated box, flips in `quad`, `scaled_about`, stored and resolved paths, validation (opacity 1.5, scale 0, an offset past `MAX_COORDINATE_MM`, a repeated id), and that a project without images omits the field.
- `events`: add, update, a rejected update leaving the project as it was, bad reorders, reorder, remove.
- `history`: updates of one image merge, of two images don't; the removed image is named; undo.
- `persistence`: a relative and an absolute path in the saved JSON, both resolved on load, and missing image files don't fail the load.
- `exports`: SVG and DXF are the same with and without images.

In the binary, `editor::background` tests that scaling from a corner keeps the opposite corner and the proportions.

## Open questions

- Embed images in the file (a "Pack" action) or keep linking only? Linking only for now.
- Typing a value into the slider, and reordering images by dragging their cards (`ReorderBackgroundImages` exists for it).
- Decoding on a thread, if large photos stall the frame they are added in.
- Should a PDF page be allowed as a source? It would need a rasterizer dependency.
- Should an image be tied to a piece so that it moves with it? Not for now: images live in project space.
