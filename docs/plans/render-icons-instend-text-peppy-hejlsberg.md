# Icons in the Tools panel

## Context

The Tools panel shows each tool as a full-width text button. Replace the text with icons and show the tool's translated name in a tooltip on hover.

## Approach

Draw the icons with egui's painter, in `src/panels/tools/mod.rs` only. This needs no new dependency, image assets or icon font. The icons stay sharp at any scale and take their color from the current egui theme.

`Tool::label` and the existing `Label` variants supply the tooltip text, so `i18n` doesn't change. `Tool`, `hint` and `shape_points` also stay as they are, and `editor.rs` still calls `tools::show_panel` with the same signature.

## Changes to `src/panels/tools/mod.rs`

1. **Add `const ICON_BUTTON_SIZE: f32 = 32.0`.**

2. **Add `Tool::paint_icon(self, painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke)`**, drawing inside `rect` shrunk by a margin:
   - `Select`: arrow pointer outline, using `egui::Shape::closed_line` because the shape is concave
   - `Draw`: open polyline with small filled circles at its vertices, matching how the tool places points
   - `Rectangle`: `painter.rect_stroke` with `egui::StrokeKind::Middle`
   - `Circle`: `painter.circle_stroke`

3. **Add `fn icon_button(ui, tool: Tool, selected: bool, name: &str) -> egui::Response`:**
   - Allocate the button with `ui.allocate_exact_size(egui::Vec2::splat(ICON_BUTTON_SIZE), egui::Sense::click())`.
   - Get colors from `ui.style().interact_selectable(&response, selected)`, so hover and selected states match the other selectable widgets. Paint the background with `visuals.weak_bg_fill` and `visuals.bg_stroke`, and the icon with `visuals.fg_stroke`.
   - Report the name to accessibility tools through `response.widget_info` with `egui::WidgetInfo::selected(egui::WidgetType::Button, ...)`, because a painted button has no text of its own.
   - Return `response.on_hover_text(name)`.

4. **Rewrite the loop in `show_panel`:**

   ```rust
   ui.label(language.label(Label::Tools));
   ui.horizontal_wrapped(|ui| {
       for option in Tool::ALL {
           let name = language.label(option.label());
           if icon_button(ui, option, *tool == option, name).clicked() {
               *tool = option;
           }
       }
   });
   ```

   The panel keeps its `Tools` heading and 110 px default width. With wrapping, the icons sit in two rows of two and reflow if the panel is resized.

## Documentation

- Copy this plan to `docs/plans/tools-panel-icons.md`, as `CLAUDE.md` requires, and list it in the Docs table of `docs/project-index.md`.
- In `docs/project-index.md`, update the `panels/tools/mod.rs` size row and symbol table.

## Verification

- `cargo check --bin leathercraft --message-format=short`
- `cargo clippy`
- `cargo fmt`
- `cargo test --lib --no-default-features -q`
- Run `cargo run` and check by hand:
  - All four icons render and are recognizable.
  - Clicking an icon switches the tool, and the selected icon is highlighted.
  - Hovering shows the tool's name, in both English and Ukrainian.
  - The icons wrap when the panel is resized.
  - The panel is hidden in 3D view, as it is now.
