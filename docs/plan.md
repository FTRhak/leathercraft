# Leathercraft pattern editor plan

## Problem and approach

The repository currently contains only a license. Build a cross-platform desktop MVP in Rust for drafting leather patterns, saving projects, exporting cut patterns, and previewing pattern pieces in 3D. Use Bevy with `bevy_egui` as the integrated desktop UI and renderer.

## Todos

1. Bootstrap the Rust desktop application and establish its Bevy/egui application shell.
2. Define a unit-aware pattern and project model, including serializable geometry and validation.
3. Implement the CAD-style 2D editor with selection, drawing, editing, zoom/pan, and measurement-aware tools.
4. Add project save/load and SVG/DXF exports for the pattern geometry.
5. Add a 3D leather-look preview of pattern pieces with camera navigation.
6. Document how to build/run the app and verify core behavior with tests.

## Notes and considerations

- The 3D MVP previews individual pattern pieces with a leather-like material and camera controls; it does not infer sewn seams or automatically assemble a finished wallet/bag.
- Store native `.leathercraft` project data in a versioned, human-readable format. Keep internal geometry in real-world units and make display/export unit conversion explicit.
- SVG and DXF are exports of the 2D pattern geometry; project round-tripping uses `.leathercraft`.
- Keep geometry and file-format logic independent of UI/rendering so they can be tested without launching the desktop application.
- Confirm crate APIs and compatibility when implementing because the repository has no existing Rust project or dependency choices.
