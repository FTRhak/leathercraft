# Project index

A map for finding code without reading whole files. It complements `CLAUDE.md`, which holds the architecture rules and invariants and isn't repeated here.

Indexed at commit `12d841a` plus the change to pieces built from a base shape and boolean child shapes. Line numbers drift, so treat them as hints and search for the symbol name if one doesn't match.

## How to read this repo cheaply

- **Start with this index, then read a line range.** All of `src/` is about 50k tokens, and `i18n/uk.rs` alone is about 6k.
- **Never open or sweep these:**
  - `target/` (11 GB, about 7,700 files)
  - `.git/`
  - `Cargo.lock` (165 KB, about 41k tokens, more than all of `src/`)
- **Scope searches.** Pass `path: src` to Grep and use `src/**/*.rs` with Glob. An unscoped `**/*` Glob returned `.git` and `target` entries and hit its 100-result limit, with no source files beyond `src/document.rs`.
- **Skip test modules unless you're changing tests.** The ranges are listed under [Library crate](#library-crate-core-no-bevy).
- **Keep build output short:**
  - `cargo test --lib --no-default-features -q` doesn't compile Bevy.
  - `cargo check --bin leathercraft --message-format=short` is the desktop type-check.
  - `cargo run` opens a GUI and can't be verified from a shell, so don't use it as a check.
- **Treat the plan docs as history.** See [Docs](#docs).

## File sizes

Tokens are estimated as bytes / 4. The estimate is rougher for `uk.rs`, because Cyrillic takes two bytes per character.

| File | Lines | ~Tokens | Crate | Role |
|---|---:|---:|---|---|
| `src/editor/mod.rs` | 86 | 600 | bin | `ViewMode`, `ShapePart` (edge or vertex picked with the Edge tool), `send`, `export`, `format_length` |
| `src/editor/editor_state_struct.rs` | 440 | 3,700 | bin | `EditorState` and its methods (primary piece plus multi-selection, primary child plus multi-selection of children in that piece, selected edge or vertex; snap settings, Alt suspension, snap feedback and hover target: `active_snap`, `snap_point`, `save_snap_when_settled`), `document_name` |
| `src/editor/drag_state_struct.rs` | 60 | 480 | bin | `ShapeRef`, `DragTarget`, `DragState` (with the `group` of pieces or `child_group` of children moved along, and the snap `targets` and `moving` shapes gathered at the start) |
| `src/editor/shape_draft_struct.rs` | 9 | 60 | bin | `ShapeDraft` |
| `src/editor/canvas.rs` | 1,290 | 10,400 | bin | `canvas`, `Viewport` (`hit_tolerance`), tool behavior (Shift+click toggles a child of the selected piece, else a piece; the Edge tool picks a vertex or a polygon edge; a line or stitch line is always added as a child; the Stitch holes tool clicks a draft), hit testing (pieces and their lines, children with a line tolerance, handles, vertices, polygon edges), painting (centerlines in their dash pattern and stitch holes above all outlines); snapping of drawing tools (`draw_targets`, `snap_draw`, the hover target) and `draw_snap_feedback` |
| `src/editor/confirm_discard.rs` | 14 | 100 | bin | `confirm_discard`: unsaved-changes prompt |
| `src/editor/drag.rs` | 309 | 2,100 | bin | `begin_drag`, `begin_handle_drag`, `snap_setup` (what a drag snaps to), `update_drag` (snapped offset or handle; stops before a stitch line leaves its piece), `finish_drag`: dragging pieces and children (the whole selection when a selected piece or child is grabbed), children and shape handles with the Select tool, and handles with the Edge tool without changing the selection |
| `src/editor/grid.rs` | 73 | 600 | bin | `draw_grid`: canvas grid and axes, grid steps; `draw_snap_dots`: the snap grid |
| `src/editor/ui.rs` | 401 | 3,700 | bin | `editor_ui` (menu bar, tool bar, status bar, dock area), `action_button`, `file_menu`, `file_command`, `edit_menu`, `handle_hotkeys`, `window_menu`, `delete_selection`, `select_all`, `language_control`, `project_dialog`, `save_as`: top and bottom bars, dock area, menus with configured shortcuts, hotkey dispatch, new-project and Settings dialogs, language picker |
| `src/editor/panel_viewer.rs` | 193 | 1,300 | bin | `PanelViewer`: the `TabViewer` that draws each dock tab; the History, Layout and Properties tab titles have icons; the Align tab targets the selected children when there are any, else the selected pieces |
| `src/ui_components/icons/mod.rs` | 113 | 1,000 | bin | Icon font (`src/res/leathercrafticons.ttf`): `install_icon_font`, `glyph` codes, `paint_glyph`, `icon_text` (glyph plus text); shared `icon_button` |
| `src/panels/mod.rs` | 6 | 50 | bin | Panel module declarations |
| `src/ui_components/mod.rs` | 12 | 250 | bin | Reusable UI component module declarations (`input` allows dead code until `TextInput` has a caller, `radio` since the theme became a group button) |
| `src/ui_components/button/mod.rs` | 98 | 800 | bin | `Button`: the design-system text button (`primary`, `secondary`, `outline`), an `egui::Widget` drawn in its default, hover or disabled state (disabled through `ui.add_enabled`), with a focus ring |
| `src/ui_components/checkbox/mod.rs` | 178 | 1,450 | bin | `Checkbox`: the design-system checkbox (from the Brilliant `Checkbox`), an `egui::Widget` bound to a `bool` with an optional label and `mixed` (a dash; a click checks it), drawn checked, unchecked or mixed in its default, hover or disabled state, with a focus ring and focus-colored edge; `paint_mark` (check or dash) |
| `src/ui_components/checkbox/style.rs` | 61 | 450 | bin | `CheckboxValue` (on, off, mixed), `CheckboxState`, `paint`: picks the `Palette::checkbox` entry for a value and state |
| `src/ui_components/button/style.rs` | 37 | 300 | bin | `ButtonKind`, `ButtonState`, `paint`: picks the `Palette::button` entry for a kind and state |
| `src/ui_components/dropdown/mod.rs` | 370 | 3,000 | bin | `Dropdown`: the design-system dropdown (from the Brilliant `Select`), a field with the chosen value, an optional leading slot and a painted caret, drawn default, hover, focus (focused or open) or disabled, opening a themed `egui::Popup` list with `show_ui`; `DropdownItem` (an `egui::Widget` with an optional leading slot, drawn selected, hover or disabled), `dropdown_value` (the item counterpart of `selectable_value`), `group_label` |
| `src/ui_components/dropdown/style.rs` | 39 | 300 | bin | `DropdownState`, `paint`: picks the `Palette::dropdown` entry for a state |
| `src/ui_components/group_button/mod.rs` | 165 | 1,450 | bin | `GroupButton`: the design-system group button (from the Brilliant `Button Group` and `Segment`), an `egui::Widget` bound to a `&mut V` with one joined segment per `option`, drawn default, hover, selected, focus, disabled or disabled-selected, with dividers, a focus ring and rounded outer corners; the response is marked changed when a click picks another option |
| `src/ui_components/group_button/style.rs` | 55 | 400 | bin | `SegmentState`, `paint`: picks the `Palette::group_button` entry for a state |
| `src/ui_components/input/mod.rs` | 185 | 1,650 | bin | The design-system input fields (from the Brilliant `Number Input` and `Search Field`): re-exports `TextInput`, `NumberInput`, `SearchInput`; shared `Field` frame (allocate, state, paint, click-to-focus), `show_edit` (a frameless `TextEdit` centered in a rect), `escaped` (Esc leaves a field without closing the dialog), painted search, clear and stepper icons |
| `src/ui_components/input/text.rs` | 71 | 500 | bin | `TextInput`: single-line text field with an optional hint, an `egui::Widget` |
| `src/ui_components/input/number.rs` | 270 | 2,400 | bin | `NumberInput`: any `Numeric` with range, step, decimals, unit and accent; typed text committed on Enter or focus loss (Esc drops it, a comma reads as a point), stepper clicks and Up/Down keys (Shift for ten steps), always clamped |
| `src/ui_components/input/search.rs` | 136 | 1,100 | bin | `SearchInput`: text field with a search icon (dim while empty and idle) and a clear button while it has text |
| `src/ui_components/input/style.rs` | 44 | 350 | bin | `InputState` (default, hover, focus, accent, disabled), `paint`: picks the `Palette::input` entry for a state |
| `src/ui_components/dock/mod.rs` | 185 | 1,550 | bin | `PanelTab` (including `Snap`; the tools are a fixed tool bar, not a tab), `PanelDock`: panel arrangement, default layout, saving and restoring it |
| `src/ui_components/palette/mod.rs` | 136 | 850 | bin | `Palette` (`DARK`, `LIGHT`, `of`: picked from `visuals.dark_mode`), values from the Brilliant palette boards: canvas colors (grid, outline, label, hole, centerline, edge) and one field per color group, each group in its own submodule with its `DARK` and `LIGHT` values; `hex` and `hex_opt` (`0xRRGGBB` to `Color32`); re-exports the component paint types; used by `canvas`, `grid`, `button`, `tabs`, `dropdown`, `input`, `checkbox`, `radio`, `group_button` and `title_bar` |
| `src/ui_components/palette/surfaces.rs` | 43 | 250 | bin | `Surfaces`: backgrounds from the canvas to chips |
| `src/ui_components/palette/borders.rs` | 35 | 200 | bin | `Borders`: divider, panel, control, field and chip lines |
| `src/ui_components/palette/text.rs` | 39 | 250 | bin | `TextColors`: six steps of text brightness |
| `src/ui_components/palette/accent.rs` | 35 | 250 | bin | `AccentColors`: the single accent (base, fill, border, container, selected row) |
| `src/ui_components/palette/status.rs` | 31 | 200 | bin | `StatusColors`: warning and the gizmo axes |
| `src/ui_components/palette/button.rs` | 76 | 550 | bin | `ButtonColors` (`ButtonStates`, `ButtonPaint`: fill, stroke and text per kind and state, plus the focus ring) |
| `src/ui_components/palette/tabs.rs` | 60 | 450 | bin | `TabColors` (`TabPaint`: text, icon and underline per tab state, the strip background and border, the focus ring) |
| `src/ui_components/palette/dropdown.rs` | 83 | 600 | bin | `DropdownColors` (`FieldPaint`: fill, stroke, text and caret per state; `MenuColors`: the open list) |
| `src/ui_components/palette/input.rs` | 65 | 500 | bin | `InputColors` (`InputPaint`: fill, stroke, text, placeholder, unit and icon per state, including the accent state) |
| `src/ui_components/palette/checkbox.rs` | 72 | 550 | bin | `CheckboxColors` (shared by the checkbox and radio; `CheckboxStates`, `CheckboxPaint`: fill, stroke, mark or dot and label per checked/unchecked value and state, plus the focus ring; light values derived, the design has dark boards only) |
| `src/ui_components/palette/group_button.rs` | 63 | 450 | bin | `GroupButtonColors` (`SegmentPaint`: fill and text per segment state, plus the border, disabled border and focus ring; light values derived) |
| `src/ui_components/palette/title_bar.rs` | 153 | 1,050 | bin | `TitleBarColors` (`MenuTitleStates`, `MenuTitlePaint`, `ModeSwitchColors`, `ModePaint`, `WindowButtonColors`: bar, logo, app name, menu titles, mode switch and window buttons; light values derived) |
| `src/ui_components/radio/mod.rs` | 127 | 1,050 | bin | `Radio`: the design-system radio button (from the Brilliant `Radio`), an `egui::Widget` with an optional label, drawn on (a dot) or off in its default, hover or disabled state, with a focus ring and focus-colored edge; the checkbox sizes and `Palette::checkbox` colors; `radio_value` (the counterpart of `ui.radio_value`); no caller yet |
| `src/ui_components/radio/style.rs` | 36 | 300 | bin | `RadioState`, `paint`: picks the `Palette::checkbox` entry for a selected or unselected button and state |
| `src/ui_components/tabs/mod.rs` | 182 | 1,500 | bin | `Tab`: the design-system tab (from the Brilliant `Dialog Tab`), an `egui::Widget` with a label and optional icon glyph, drawn active, inactive, hover, disabled or disabled-active, with an underline and a focus ring; `tab_bar` (the strip), `tab_value` (the tab counterpart of `selectable_value`) |
| `src/ui_components/tabs/style.rs` | 42 | 300 | bin | `TabState`, `paint`: picks the `Palette::tabs` entry for a state |
| `src/panels/tools/mod.rs` | 333 | 1,650 | bin | `Tool` (with `Centerline` and `StitchHoles`), `StitchSettings`, `ToolSettings`, `ZoomRequest` (All or Selection), `Tool::needs_piece` (Centerline and Stitch holes are enabled, and reachable by hotkey, only while a piece is selected; otherwise the tool falls back to Select), tool, new-piece and operation icons from font glyphs; `show_toolbar`: the full-width tool bar under the menu bar (drawn by `editor_ui`, not a dock tab), one scrolling line of tool buttons, the new-piece and boolean operation modes as one dropdown (`op_mode_dropdown`), then the Centerline and Stitch holes buttons, then Zoom to fit and Zoom to selection (`paint_fit_icon`, `paint_fit_selection_icon`: painted corner brackets, the second with a square inside and enabled only with a selection; they set `ToolSettings::zoom_request`), then the current tool's settings (regular polygon sides, centerline style, stitch holes) |
| `src/panels/layout/mod.rs` | 273 | 2,800 | bin | Layout panel: tree of pieces with collapsible children, lock, delete, reorder and operation controls (none for a centerline); Shift+click toggles a piece or child |
| `src/panels/align/mod.rs` | 248 | 2,000 | bin | Align panel: `Reference` (Align To; `Parent` only for children), `Targets` (pieces or children of one piece), `AlignOptions` (kept in `EditorState::align`), align, distribute and spacing icons from font glyphs (`arrange_glyph`), `show_panel` |
| `src/panels/snap/mod.rs` | 44 | 400 | bin | Snap panel: `show_panel` edits `SnapSettings` (on/off, grid and step, vertices, smart guides, distance in px) with the design-system `Checkbox` |
| `src/panels/history/mod.rs` | 39 | 300 | bin | History panel: list of actions, click to jump |
| `src/panels/material/mod.rs` | 241 | 2,100 | bin | Material form, Material panel, new-project dialog |
| `src/panels/settings/mod.rs` | 106 | 850 | bin | Settings window: `SettingsDraft` (with the Hotkeys filter text), `SettingsResult`, `settings_dialog` with Default material, Hotkeys and Appearance tabs in a `tab_bar`, and a right-aligned footer of `Button`s (primary OK, secondary Cancel) |
| `src/panels/settings/appearance.rs` | 26 | 200 | bin | Appearance tab: `appearance_tab`, the theme as a design-system `GroupButton` |
| `src/panels/settings/material.rs` | 186 | 1,550 | bin | Default material tab: `material_tab`, its own material form of `Dropdown`s (the color one with swatches as leading slots and grouped items) and a thickness `NumberInput` in mm, with an outline Reset to defaults `Button` |
| `src/panels/settings/hotkeys.rs` | 229 | 2,050 | bin | Hotkeys tab: `Capture`, `hotkeys_tab` (a `SearchInput` filtering the list), `matches_filter` (by action name or shortcut text), `shortcut_button`, key capture; an outline Reset to defaults `Button` |
| `src/panels/properties/mod.rs` | 81 | 650 | bin | Properties panel: `EditorView`, `ViewControls`, `Selection`, view or preview controls (tool and navigation hints), display units, `show_panel`; re-exports `line_style_fields` and `stitch_fields` for the tool bar |
| `src/panels/properties/piece.rs` | 95 | 800 | bin | Selected piece's name, size or result error, title and fields of its base or child, To Polygon button |
| `src/panels/properties/shape.rs` | 56 | 500 | bin | `shape_fields`: dispatch to the file of each shape type; the selected vertex's corner radius |
| `src/panels/properties/fields.rs` | 80 | 600 | bin | Shared inputs: `MIN_SIZE_MM`, `length_field`, `bounded_length_field`, `angle_field`, `center_fields` |
| `src/panels/properties/polygon.rs` | 38 | 300 | bin | Polygon's point count and the selected edge's length |
| `src/panels/properties/rectangle.rs` | 25 | 200 | bin | Rectangle's center, width, height and rotation |
| `src/panels/properties/ellipse.rs` | 24 | 200 | bin | Ellipse's center, radii and rotation |
| `src/panels/properties/regular_polygon.rs` | 28 | 250 | bin | Regular polygon's center, radius, sides and rotation |
| `src/panels/properties/line.rs` | 69 | 700 | bin | Centerline's ends, length, angle and style (`line_style_fields`, shared with the tool bar) |
| `src/panels/properties/stitch.rs` | 129 | 1,000 | bin | Stitch line's hole type, pitch and size (`stitch_fields`, shared with the tool bar), and its length, hole count, actual pitch and thread hint (`stitch_details`) |
| `src/preview/mod.rs` | 302 | 2,600 | bin | 3D scene, camera, mesh extrusion from each piece's `punched_region`; `apply_theme` sets the background and ground plane colors for the theme |
| `src/model.rs` | 594 | 4,500 | lib | `Project`, `PatternPiece`, `ChildShape`, `BooleanOp`, `Point`, `Units`, validation (a base must be closed; a line's ends and dash count; a stitch line's points, hole parameters, hole count and overlap in `validate_stitch_line`) |
| `src/boolean.rs` | 290 | 2,500 | lib | `Region`, `PatternPiece::region`: combining a piece's shapes, skipping open lines; `punched_region`: the region with stitch holes cut, for the preview; `stitch_area`, `stray_stitch_lines`: the leather a stitch line must stay on |
| `src/material.rs` | 462 | 2,800 | lib | `Material` and the scales of its properties |
| `src/events.rs` | 980 | 6,650 | lib | `ProjectEvent` (including the group `MovePieces`, `RemovePieces`, `ArrangePieces`, `MoveChildren`, `ArrangeChildren`, `RemoveChildren`), `ProjectChange`, `Project::apply` |
| `src/snap.rs` | 558 | 4,300 | lib | `SnapSettings` (with `clamped`), `SnapScope` (`Project` or one `Piece`), `SnapTargets` (`collect`, `with_points`, `without_point`, `without_box`), `Moving`, `Guide`, `GuideAxis`, `SnapFeedback`, `snap_point`, `snap_offset` (vertex, then guides per axis, then grid per axis); tests in the second half |
| `src/align/model.rs` | 108 | 850 | lib | `Axis`, `Edge`, `Arrange` (Illustrator's align, distribute and spacing operations), `AlignTo` (including `Parent`, the base shape of the children's piece), `Spacing`, re-exported from `align`; the module-private `Item` (an id, its span on one axis, and whether it stays fixed) |
| `src/align/mod.rs` | 468 | 3,450 | lib | `piece_box`, `offsets` (pieces), `child_offsets` (children of one piece), shared `arrange_items` that dispatches to the submodules below; tests |
| `src/align/align_edges.rs` | 15 | 130 | lib | `align`: moves each item's edge or center onto the reference's |
| `src/align/distribute.rs` | 35 | 300 | lib | `distribute`: spaces edges or centers evenly, between the outer items or across the project or parent |
| `src/align/space.rs` | 55 | 500 | lib | `space` (equal automatic gaps), `pack` (fixed gap away from the key object) |
| `src/i18n/uk.rs` | 587 | 6,700 | lib | Ukrainian catalog, `plural` helper |
| `src/i18n/mod.rs` | 349 | 2,350 | lib | `Language`, `Label`, `Message` |
| `src/exports/mod.rs` | 323 | 2,300 | lib | `ExportError`, shared helpers (`validate_export`, `centerlines`, `stitch_lines`, `HoleOutput`, `stitch_holes`, `number`), tests |
| `src/exports/svg.rs` | 104 | 1,100 | lib | `to_svg`: outlines, then a group of centerlines with their dash pattern, then a group of stitch holes |
| `src/exports/dxf.rs` | 60 | 550 | lib | `to_dxf`: outlines on `PATTERN`, each centerline dash as a `LINE` on `CENTERLINE`, stitch holes as `CIRCLE` or `LWPOLYLINE` on `STITCH_HOLES` |
| `src/shapes/mod.rs` | 424 | 3,400 | lib | `Shape`: dispatch to the shape structs (including `StitchLine`), constructors from a drag, `is_open`, `distance_to`, `corners`, `corner_radius`, `with_corner_radius` (never a stitch line's ends), `contains` (false when open) |
| `src/shapes/line.rs` | 305 | 2,300 | lib | `Line` (open, a centerline), `LineStyle` (solid, dashed, dash-dot), `MIN_DASH_MM`, `MAX_DASHES_PER_LINE`, `DOT_MM`: `from_ends`, `with_length`, `with_angle`, `distance_to`, `dashes`, `dash_count`; tests |
| `src/shapes/stitch.rs` | 699 | 6,000 | lib | `StitchLine` (open path of stitch holes), `HoleKind` (French, Diamond, Round), `Hole`, `Run`, `StitchBand` (thread and leather hints by pitch), limits and `PITCH_PRESETS`: `path`, `runs`, `holes`, `hole_count`, `actual_pitch`, `hole_outline`, `holes_overlap`; tests in the second half |
| `src/shapes/corners.rs` | 243 | 1,900 | lib | Rounded corners: `round_corners` (fillet arcs), `radius_at`, `set_radius`, `valid_radii`; `fillet` and `push_point` also serve stitch lines |
| `src/shapes/rectangle.rs` | 128 | 900 | lib | `Rectangle` |
| `src/shapes/ellipse.rs` | 123 | 900 | lib | `Ellipse`, `CIRCLE_SEGMENTS` |
| `src/shapes/regular_polygon.rs` | 154 | 1,150 | lib | `RegularPolygon`, `REGULAR_POLYGON_SIDES` |
| `src/shapes/polygon.rs` | 116 | 850 | lib | `Polygon`, its edges and their lengths |
| `src/i18n/en.rs` | 532 | 5,100 | lib | English catalog |
| `src/history.rs` | 725 | 4,950 | lib | `History`, `HistoryCommand`: snapshots for undo and redo |
| `src/persistence.rs` | 230 | 1,750 | lib | `save_project`, `load_project`, `write_export` |
| `src/settings/mod.rs` | 165 | 1,350 | bin | `settings.json` in the OS config directory: language, panel layout, snap settings (`load_snap`, `save_snap`), and `Preferences` (default material, hotkeys, theme) with `load_preferences`, `save_preferences` |
| `src/settings/theme.rs` | 45 | 350 | bin | `Theme` (Dark, Light; `ALL`, `label`), `apply_theme`: sets egui's theme when `Preferences::theme` changes |
| `src/settings/hotkeys.rs` | 340 | 2,700 | bin | `Action` (bindable commands, default shortcuts), `Hotkeys` (`from_json`, `to_json`, `first`, conflicts, `triggered`), `format_shortcut`, `parse_shortcut`; tests |
| `src/document.rs` | 70 | 550 | bin | `ActiveProject` (`new`, project and history), `ProjectMessage`, `HistoryMessage`, `apply_project_events` |
| `src/main.rs` | 52 | 360 | bin | App wiring and system schedule (including `settings::apply_theme`); loads `Preferences` and starts the project with their default material; the editor UI starts one pass after the icon font is installed |
| `src/lib.rs` | 11 | 45 | lib | Module declarations |

## Library crate (core, no Bevy)

| File | Lines | Contents |
|---|---:|---|
| `material.rs` | 3-4 | `MIN_THICKNESS_MM`, `MAX_THICKNESS_MM` |
| | 8-104 | `Origin`, `Treatment`, `ThicknessCategory` (`from_millimeters`) |
| | 106-255 | `ColorGroup`, `LeatherColor` (`group`, `rgb`) |
| | 258-380 | `Texture`, `Strength`, `Flexibility`, `WaterResistance`, `HeatResistance` |
| | 383-420 | `Material`: `thickness_category`, `has_valid_thickness` |
| | 421-462 | Tests |
| `model.rs` | 8-13 | `PROJECT_VERSION` (2) and validation limits (`MAX_PIECES`, `MAX_POINTS_PER_PIECE`, `MAX_CHILDREN_PER_PIECE`, `MAX_COORDINATE_MM`, `MIN_POLYGON_AREA_MM2`) |
| | 17-70 | `Units`, `Point`, `Bounds` (`union`) |
| | 71-117 | Ring helpers: `ring_bounds`, `ring_contains`, `signed_area` |
| | 119-140 | `BooleanOp`, `ChildShape` |
| | 143-262 | `PatternPiece`: `bounds`, `child`, `child_index`, `next_child_id`, `translate`, `validate_shape`, `validate_ring`, `validate_shapes` |
| | 265-353 | `Project`: `bounds`, `validate` (301), `next_piece_id`, `add_piece`, `remove_piece` |
| | 355-476 | Tests |
| `shapes/mod.rs` | 1-40 | Re-exports, `Shape` enum (`Polygon`, `Rectangle`, `Ellipse`, `RegularPolygon`, `Line` wrapping the structs of the same name), `to_world`, `to_local`, `positive` |
| | 43-200 | `rectangle_from_corners`, `ellipse_from_center`, `regular_polygon_from_center`, `ring`, `to_polygon` (keeps corners and radii), `corners`, `corner_radius`, `with_corner_radius`, `bounds`, `contains`, `center`, `translate`, `handles` (corners, not ring points), `with_handle_at`, `has_valid_size` |
| | 202-346 | `test_support` (`validate`, `assert_near`), tests |
| `shapes/corners.rs` | 14-128 | `radius_at`, `set_radius` (keeps the list to the vertex count, empties it when all are 0), `valid_radii`, `round_corners`, `fillet` (radius reduced so an arc reaches at most halfway along each edge); tests from 130 |
| `shapes/rectangle.rs` | 5-68 | `Rectangle` (with `radii`): `from_corners`, `ring`, `corners`, `with_handle_at`, `has_valid_size`; tests from 70 |
| `shapes/ellipse.rs` | 6-79 | `CIRCLE_SEGMENTS`, `Ellipse`: `from_center`, `ring`, `handles`, `with_handle_at`, `has_valid_size`; tests from 81 |
| `shapes/regular_polygon.rs` | 7-85 | `REGULAR_POLYGON_SIDES` (3..=64), `ROTATION_SNAP`, `RegularPolygon` (with `radii`): `from_center`, `ring`, `corners`, `with_handle_at`, `has_valid_size`; tests from 87 |
| `shapes/line.rs` | 1-208 | `LineStyle` (`dashed_default`, `dash_dot_default`, `pattern`), `Line`: `from_ends` (Shift snaps to 15°), `length`, `angle`, `midpoint`, `with_length`, `with_angle`, `translate`, `handles`, `with_handle_at`, `distance_to`, `dash_count`, `dashes`, `has_valid_size`; tests from 210 |
| `shapes/polygon.rs` | 4-67 | `Polygon` (`points`, `radii`): `new`, `ring` (rounded), `has_valid_size`, `translate`, `with_handle_at`, `edge`, `edge_length`, `with_edge_length` (moves the edge's end along it); tests from 69 |
| `boolean.rs` | 11-53 | `ROUND_PRECISION_MM`, `Region`: `rings`, `vertices`, `hole_indices`, `bounds`, `contains`, `contains_path` |
| | 67-96 | `PatternPiece::region` |
| | 98-221 | Tests |
| `events.rs` | 10-128 | `ProjectEvent`, `ProjectChange` |
| | 130-314 | `Project::apply` |
| | 316-358 | `edit` (validates a copy), `piece_mut`, `unlocked_piece_mut`, `child_mut`, `child_index` |
| | 360-980 | Tests |
| `history.rs` | 6-48 | `MAX_HISTORY`, `HistoryCommand`, `MergeKey`, `Entry`, `History` |
| | 50-268 | `new`, `record`, `apply`, `actions`, `position`, `can_undo`, `can_redo`, `describe` |
| | 270-725 | Tests |
| `persistence.rs` | 7-34 | `save_project`, `load_project`, `write_export` |
| | 36-169 | Tests |
| `exports/mod.rs` | 14-38 | `ExportError`, conversion to `Message` |
| | 41-66 | `validate_export` (returns each piece's `Region`), `number` |
| | 68-187 | Tests |
| `exports/svg.rs` | 4 | `to_svg` |
| `exports/dxf.rs` | 5 | `to_dxf` |
| `i18n/mod.rs` | 14-66 | `Language`: `tag`, `from_tag`, `native_name`, `label`, `message`, `unit_symbol` |
| | 70-165 | `Label`, with nested variants for material values and `Operation(BooleanOp)` |
| | 169-226 | `Message` |
| | 235-279 | Tests |

## Binary crate (desktop)

### `editor/mod.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 27-40 | Types | `ViewMode`, `ShapePart` (`Edge` or `Vertex`, by index into `Shape::corners`) |
| 34-36 | Events | `send` |
| 38-70 | Menu bar helpers | `export` |
| 72-78 | Formatting | `format_length`, also used by the Properties panel |

### `editor/editor_state_struct.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 12-48 | Type | `EditorState`, with `op_mode`, `selected` (primary piece), `pieces` (every selected piece), `selected_child` (primary child), `children` (every selected child of that piece) and `selected_part` (edge or vertex) |
| 39-62 | Defaults | `Default` impl |
| 67-242 | Methods | `new` (language and snap settings), `report`, `report_error`, `active_snap`, `snap_point`, `save_snap_when_settled`, `reset_document`, `on_project_change`, `on_history_restore`, `selected_child_id`, `selected_children`, `clear_children`, `select`, `toggle_child`, `focus_child`, `select_all_children`, `is_selected`, `toggle_piece`, `focus`, `select_all`, `prune_selection`, `select_part`, `selected_part` (only while its shape is the one shown in Properties), `clear_released_drag`, `displayed` |
| 211-216 | Helpers | `document_name` |

### `editor/canvas.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 18-32 | Constants | Zoom limits, hit radii, edge label limit, `SNAP_COLOR`, `SELECTED_COLOR` |
| 21-36 | Viewport | `Viewport` (`to_screen`, `to_world`) |
| 61-253 | Canvas | `canvas` (runs `EditorState::zoom_request`), `selection_bounds` (selected children, else selected pieces), `fit_view` |
| 114-195 | Select and Edge tools | `handle_select`, `handle_edge` (click picks a vertex of the selected piece, else a polygon edge; drag moves a vertex; Esc clears), `child_of_selected_at`, `select_at` |
| 152-262 | Drawing tools | `handle_draw`, `finish_draft`, `commit_shape` (new piece or `AddChild`), `handle_shape`, `add_piece` |
| 300-440 | Hit testing | `piece_hit`, `piece_at`, `child_at`, `handle_at`, `segment_distance`, `edge_at`; `vertex_at` sits after `draw_edges` |
| 403-702 | Painting | `op_color`, `draft_color`, `dashed_ring`, `draw_handles`, `draw_pieces`, `draw_edges` (selected and hovered edge, selected vertex), `draw_edge_labels` (by corners, so arcs get no labels), `fill_polygon`, `draw_draft`, `draw_shape_draft` |

### `editor/ui.rs`

| Lines | Area | Symbols |
|---:|---|---|
| 21-160 | Top-level layout | `editor_ui`, which also shows the dock area, the new-project dialog and the Settings window (OK saves `Preferences`) |
| 163-266 | Menus | `action_button` (label plus the action's first shortcut), `file_menu`, `file_command` (New, Open, Save, Save As; shared with hotkeys), `edit_menu` (with Settings…), `window_menu` |
| 268-372 | Input | `handle_hotkeys` (dispatches `Hotkeys::triggered`, skipped while drawing, in a dialog or in a text field), `delete_selection` (several children go as `RemoveChildren`, several pieces as `RemovePieces`, locked ones stay), `select_all` (all children of the selected piece while one is selected) |
| 374-388 | Language | `language_control` |
| 390-401 | File dialog | `project_dialog`, `save_as` |

### `ui_components/`

| File | Lines | Symbols |
|---|---:|---|
| `mod.rs` | 1-12 | Module declarations |
| `button/mod.rs` | 19-48 | `Button`: `new`, `primary`, `secondary`, `outline` |
| | 50-98 | `impl egui::Widget for Button`: size from the label, state from enabled and hover, paint, focus ring |
| `button/style.rs` | 8-24 | `ButtonKind`, `ButtonState` |
| | 26-37 | `paint`: the palette entry of a kind and state |
| | 82-88 | `focus_color` |
| `checkbox/mod.rs` | 31-54 | `Checkbox`: `new`, `mixed` |
| | 56-148 | `impl egui::Widget for Checkbox`: size from box and label, toggle on click, value and state, focus ring, box, mark, label |
| | 152 | `paint_mark`: the check or the dash |
| `checkbox/style.rs` | 9-43 | `CheckboxValue`, `CheckboxState`, their `new` |
| | 47-61 | `paint`: the palette entry of a value and state |
| `radio/mod.rs` | 32-44 | `Radio`: `new` |
| | 46-111 | `impl egui::Widget for Radio`: size from circle and label, state, focus ring, circle, dot, label |
| | 115-127 | `radio_value`: select on click |
| `radio/style.rs` | 11-26 | `RadioState`, its `new` |
| | 29-36 | `paint`: the palette entry of a selection and state |
| `tabs/mod.rs` | 39-67 | `Tab`: `new`, `icon`, `selected` |
| | 69-137 | `impl egui::Widget for Tab`: size from icon and label, state, icon, label, underline, focus ring |
| | 141-164 | `tab_bar`: strip background and borders, a left-to-right child `Ui` for the tabs |
| | 169-182 | `tab_value`: select on click |
| `tabs/style.rs` | 9-31 | `TabState`, `TabState::new` |
| | 34-42 | `paint`: the palette entry of a state |
| `dropdown/mod.rs` | 47-64 | `Leading`: the slot before the text, `paint_at` |
| | 69-117 | `Dropdown`: `new`, `width`, `max_height`, `leading` |
| | 118-210 | `Dropdown::show_ui`: field size and truncated text, state, paint, caret, the popup list in a palette frame with a scroll area |
| | 213-220 | `paint_caret` |
| | 224-258 | `DropdownItem`: `new`, `selected`, `leading` |
| | 260-324 | `impl egui::Widget for DropdownItem`: fill for selected or hover, focus ring, leading slot, text |
| | 328-341 | `dropdown_value`: select on click |
| | 344-370 | `group_label` |
| `dropdown/style.rs` | 8-29 | `DropdownState`, `DropdownState::new` |
| | 32-39 | `paint`: the palette entry of a state |
| `input/mod.rs` | 37-50 | `Metrics`, `Field` |
| | 52-107 | `Field`: `allocate`, `edit_id`, `state`, `paint_frame`, `forward_focus` |
| | 110-133 | `show_edit`, `escaped` |
| | 136-185 | `paint_search_icon`, `paint_clear_icon`, `paint_stepper_icon` |
| `input/text.rs` | 17-51 | `TextInput`: `new`, `hint`, `width`, `id_salt` |
| | 53-71 | `impl egui::Widget for TextInput` |
| `input/number.rs` | 26-131 | `NumberInput`: `new`, `range`, `step`, `decimals`, `unit`, `width`, `accent`, `id_salt`; `value`, `current_text`, `format`, `clamp`, `set` |
| | 135-154 | `parse`, `arrow_steps` |
| | 156-270 | `impl egui::Widget for NumberInput`: arrow keys, stepper, unit after the text, select all on focus, commit on focus loss |
| `input/search.rs` | 19-53 | `SearchInput`: `new`, `hint`, `width`, `id_salt` |
| | 55-131 | `impl egui::Widget for SearchInput`: clear button, search icon, text |
| | 134 | `icon_color` |
| `input/style.rs` | 8-33 | `InputState`, `InputState::new` |
| | 36-44 | `paint`: the palette entry of a state |
| `dock/mod.rs` | 11-32 | `PanelTab`: `CLOSEABLE`, `label` |
| | 35-121 | `PanelDock`: `load`, `is_open`, `toggle`, `reset`, `set_language`, `save_on_release` |
| | 123-163 | `window_id`, `is_usable`, `restore_windows`, `default_state`, `translations` |

### `panels/`

| File | Lines | Symbols |
|---|---:|---|
| `mod.rs` | 1-8 | Module declarations |
| `tools/mod.rs` | 8 | Icon button size |
| | 10-98 | `Tool` (Select, Edge, Draw, Rectangle, Ellipse, RegularPolygon, Centerline, StitchHoles): `ALL`, `label`, `hint`, `shape`, `glyph`, `paint_icon` |
| | 100 | `operation_glyph`: font glyph of a boolean operation |
| | 205 | `ZoomRequest`: zoom to every piece or to the selection |
| | 213-221 | `paint_fit_selection_icon`, `paint_fit_icon`: icons of Zoom to selection and Zoom to fit |
| | 244 | `show_toolbar`: tools, the new-piece and operation modes, Zoom to fit and Zoom to selection, then the sides field while the Regular polygon tool is active (`EditorState::polygon_sides`) |
| `layout/mod.rs` | 9-10 | Constants |
| | 14 | `expand_toggle`: expands or collapses a piece's children |
| | 31-52 | `paint_trash` (icon font), `paint_arrow`, `shape_label`; the lock button uses the font's lock and unlock glyphs |
| | 54 | `Selection` (read-only), `Pick` (the clicked row, returned to `PanelViewer`) |
| | 60 | `child_row`: delete, reorder, operation picker, select |
| | 135 | `piece_row` |
| | 195 | `show_panel`, which returns the `ProjectEvent` to send and the `Pick` |
| `history/mod.rs` | 6 | `show_panel`, which returns the `HistoryCommand` to send |
| `material/mod.rs` | 15-103 | `DialogResult`, `combo`, `swatch`, `color_combo`, `thickness_row` |
| | 105 | `material_fields`, shared by the dialog and the Material panel |
| | 189 | `show_panel`, which returns the `ProjectEvent` to send |
| | 214 | `new_project_dialog` |
| `settings/mod.rs` | 14-44 | `DIALOG_WIDTH`, `SettingsTab`, `SettingsDraft` (edited copy of `Preferences`), `SettingsResult` |
| | 47 | `settings_dialog`: modal with Default material, Hotkeys and Appearance tabs; OK is disabled while shortcuts conflict |
| `settings/material.rs` | 16 | `material_tab`: its own form on the draft's default material, independent of `material_fields`, with Reset to defaults |
| | 37-187 | `basic_rows`, `physical_rows`, `combo`, `thickness_row`, `color_row`, `swatch` |
| `settings/hotkeys.rs` | 8-21 | `HOTKEYS_HEIGHT`, `Capture`, `HotkeyChange` |
| | 23 | `hotkeys_tab`, `shortcut_button` |
| | 140 | `capture_keys` (next key press becomes the shortcut, Esc cancels), `normalize` (Ctrl saved as Command) |
| `properties/mod.rs` | 4-15 | Submodules; re-exports `line_style_fields`, `stitch_fields` |
| | 24-47 | `EditorView`, `ViewControls` (editor or preview), `Selection` (displayed piece, child, selected edge or vertex, rename buffer) |
| | 49-77 | `view_controls`, `units_control` |
| | 80 | `show_panel`, which returns the `ProjectEvent` to send |
| `properties/piece.rs` | 13 | `selected_piece_details`: name, size or result error, fields of the selected base or child, stitch details, To Polygon button for a primitive |
| `properties/shape.rs` | 16 | `shape_fields`: calls `polygon_fields`, `rectangle_fields`, `ellipse_fields`, `regular_polygon_fields`, `line_fields` or `stitch_line_fields` |
| | 36 | `corner_radius_field`: the selected vertex's corner radius |
| `properties/fields.rs` | 6-80 | `MIN_SIZE_MM`, `length_field`, `bounded_length_field`, `angle_field` (degrees shown, radians kept), `center_fields` |
| `properties/line.rs` | 8 | `line_fields`: ends, length and angle around the start, style |
| | 35 | `line_style_fields` |
| `properties/stitch.rs` | 12-30 | `stitch_line_fields`, `stitch_fields` |
| | 90 | `stitch_details` |

### `preview/mod.rs`

| Lines | Symbols |
|---:|---|
| 12-51 | Scene constants, `PreviewPlugin`, `PreviewSettings`, `Ground` |
| 53-76 | `setup_scene` |
| 78-112 | `clear_color`, `ground_color`, `apply_theme` (follows `Preferences::theme`) |
| 114-164 | `rebuild_preview`, which skips pieces without a valid region |
| 166-183 | `update_camera` |
| 185-198 | `show_controls` |
| 200-302 | `build_piece_mesh` (from a `Region`) |

## Where to look for common tasks

| Task | Read | Change |
|---|---|---|
| Add a project edit | `events.rs` 10-229, `history.rs` `describe` | Add a `ProjectEvent` variant and handle it in `apply`, give it an entry label in `History::describe` (an `Action…` `Message`), then return it from a panel or send it from `editor/` with `send` |
| Add a validation rule | `model.rs` 8-13, 200-262, 301-333 | `validate`, plus a `Message` variant and both catalogs |
| Add UI text | `i18n/mod.rs` 70-166 | A `Label` or `Message` variant and an arm in `en.rs` and `uk.rs`; the compiler flags missing arms |
| Add a language | `i18n/mod.rs` 14-66 | `Language`, `ALL`, `tag`, `from_tag`, `native_name`, and a new catalog file |
| Add a canvas tool | `panels/tools/mod.rs`, `editor/canvas.rs` 38-93, 152-262, `settings/hotkeys.rs` | `Tool` and its `ALL`, `label`, `hint`, `glyph` (plus a glyph in the icon font), a dispatch arm in `canvas`, an `Action::Tool…` hotkey; a new primitive also goes in `Shape` (`ring`, `handles`, `with_handle_at`, `has_valid_size`), `shape_label` and `shape_fields` (with its own file in `panels/properties/`); an open one also in `is_open`, `distance_to`, its own drawing and the exports (see `docs/centerline.md` and `docs/stitch-holes.md`). A click-by-click tool shares `EditorState::draft`, owned through `draft_tool` |
| Add a boolean operation | `model.rs` `BooleanOp`, `boolean.rs` `region` | A `BooleanOp` variant and its `ALL`, its `OverlayRule`, catalog arms, `op_color`, `operation_glyph` and a glyph in the icon font |
| Add or change a material value | `material.rs`, `i18n/en.rs`, `i18n/uk.rs` | The enum and its `ALL`, plus catalog arms; a new property also goes in `Material`, `material_fields` and the Settings form in `panels/settings/material.rs` |
| Change the file format | `model.rs` 119-160, 265-278, `shapes/mod.rs` 14-23 and the shape structs, `persistence.rs` 7-30 | Bump `PROJECT_VERSION` and decide how older files are handled |
| Change SVG or DXF output | `exports/svg.rs`, `exports/dxf.rs` | `to_svg` or `to_dxf` |
| Change the 3D look or mesh | `preview/mod.rs` 49-123, 159-261 | `setup_scene`, `rebuild_preview`, `build_piece_mesh` |
| Change snapping | `snap.rs`, `editor/canvas.rs` `draw_targets`, `editor/drag.rs` `snap_setup`, `docs/snap.md` | The rules live in `snap_offset`; what snaps to what is chosen by `draw_targets` (drawing tools) and `snap_setup` (drags); settings in `SnapSettings` and the Snap panel |
| Change align or distribute | `align/mod.rs` `arrange_items`, `align/align_edges.rs`, `align/distribute.rs`, `align/space.rs`, `panels/align/mod.rs` | `arrange_items` measures the items and picks the operation's submodule; it is shared by `offsets` (pieces) and `child_offsets` (children); the panel only builds an `ArrangePieces` or `ArrangeChildren` event |
| Add a dockable panel | `ui_components/dock/mod.rs`, `editor/panel_viewer.rs` | A `PanelTab` variant, its `CLOSEABLE` and `label` entries, `default_state`, and arms in `PanelViewer::ui` and `scroll_bars` |
| Change how the app is compiled or linked | `Cargo.toml`, `.cargo/config.toml` | See [Build configuration](#build-configuration) |
| Change system scheduling | `main.rs`, `document.rs` | Keep the ordering rule in `CLAUDE.md` |
| Add a hotkey or preference | `settings/hotkeys.rs`, `settings/mod.rs`, `editor/ui.rs` `handle_hotkeys` | A hotkey: an `Action` variant with `ALL`, `label`, `default_shortcuts`, and a dispatch arm. A preference: a field of `Preferences`, a raw-JSON section in `Settings` read in `load_preferences` and written in `save_preferences`, and a tab or field in `panels/settings` |

## Build configuration

| File | Holds |
|---|---|
| `Cargo.toml` | The `dynamic_linking` feature (on by default) builds Bevy into a separate `bevy_dylib` DLL, so a code change relinks only the small `leathercraft.exe`. `[profile.dev.package."*"]` sets `opt-level = 3` for dependencies, which also keeps that DLL under the 65,535-export limit of Windows. |
| `.cargo/config.toml` | `rust-lld` as the Windows linker; the `cargo release` alias, a static build without `dynamic_linking`. |

## Tests

There are 144 tests inline in the library crate, plus 4 in the binary (`settings::hotkeys`, run with `cargo test --bin leathercraft`, which compiles Bevy):

| Module | Tests |
|---|---:|
| `model` | 8 |
| `shapes` | 46 |
| `boolean` | 9 |
| `material` | 3 |
| `align` | 14 |
| `snap` | 10 |
| `events` | 18 |
| `history` | 13 |
| `i18n` | 5 |
| `exports` | 11 |
| `persistence` | 7 |

Apart from shortcut parsing, `document`, `editor`, `preview`, `settings` and `panels` have no tests, so changes there need `cargo check` and a manual run.

## Docs

| File | Status |
|---|---|
| `CLAUDE.md` | Current. Loaded automatically in every session, so keep it short. |
| `docs/design.md` | Current. Spec for colors and components: every color comes from `Palette` with a dark and a light value, naming by role, nested per-component color structs, how to write a reusable component, and the colors not yet migrated. |
| `docs/ui-components.md` | Current. Usage spec for the `ui_components` widgets: common rules (adding, translated text, bound values, disabling, focus), which component to pick, and the API of `Button`, `Checkbox`, `Radio`, `GroupButton`, `Dropdown`, the tabs, the inputs, the icons and `PanelDock`, with a checklist for new panels. |
| `docs/icons.md` | Current. Spec for the icon font, `paint_glyph` and `icon_button`: adding glyphs, sizes, toggles, disabled state. |
| `docs/history.md` | Current. Spec for undo, redo and the History panel: user controls, snapshots and merging, the message flow in the binary, adding a new edit. |
| `docs/settings.md` | Current. Spec for `settings.json` and the Settings window: sections and their fallbacks, `Action` and `Hotkeys`, shortcut text format, the draft and key capture, adding a hotkey or preference. |
| `docs/centerline.md` | Current. Spec for centerlines: `Shape::Line` as an open child, `LineStyle` and its dash pattern in mm, validation limits, hit testing and drawing, SVG and DXF output, extending to other open shapes. |
| `docs/stitch-holes.md` | Current. Spec for stitch holes: `Shape::StitchLine` as an open polyline child, hole kinds, pitch and size, placement by runs between sharp vertices, validation limits, the click-by-click draft and canvas drawing, the `STITCH_HOLES` export layer, punching in the 3D preview, thread hints, and the saddle-stitch reference data behind the presets. |
| `docs/snap.md` | Current. Spec for snapping: the Snap panel, priority (vertex, guide, grid), scopes (a child only within its piece), the drag anchor, tolerance in px, Alt and F9, state and persistence. |
| `docs/round-vertex.md` | Current. Spec for rounded vertices: per-vertex `radii`, `Shape::corners` vs `ring`, the fillet construction and its clamping, selecting a vertex with the Edge tool, the Corner radius field, extending to new shapes. |
| `docs/i18n.md` | Current. Spec for the localization module: `Label` vs `Message`, rendering, language selection, adding text and languages, translation conventions. |
| `docs/plan.md` | Historical MVP plan. It still says the repo contains only a license. Its todo 6 asks for build and run docs, and there is no README; `CLAUDE.md` only lists the commands. |


## Refreshing this index

To regenerate the symbol list and line numbers:

```sh
rg -n "^\s*(pub(\(\w+\))? )?(fn|struct|enum|trait|impl|mod|const) " src
```

Update it when a file is added, moved or split, or when `editor/mod.rs` or `editor/canvas.rs` is restructured. Small line drift doesn't need a refresh.
