# Layout panel: tree of pieces with select, delete and lock

## Context

Pieces are listed in a flat list in the left sidebar. Deleting needs the sidebar button or a key press that only works with the Select tool active, and pieces can't be protected from accidental edits. A dedicated Layout panel gives one place to see, select, delete and lock pieces.

## Decisions

- **Tree shape:** the model has no groups, so the tree is one collapsible "Pieces" root with a row per piece. Each row has a name, a lock button and a delete button.
- **Position:** a new right-hand side panel, shown in both 2D and 3D modes, as the current list is. The sidebar keeps the selected piece's details (rename, point count, size).
- **Lock is document data:** it is saved in the `.leathercraft` file, so it goes through `ProjectEvent` and marks the project modified.
- **What lock blocks:** moving, vertex editing and deleting. A locked piece can still be selected and renamed.
- **Format version stays 1:** `locked` is `#[serde(default)]`, so existing files load as unlocked. Bumping would make them unloadable, because the version must match exactly.

## Changes

### Library

- `src/model.rs`: add `#[serde(default)] pub locked: bool` to `PatternPiece`; `PatternPiece::new` sets it to `false`.
- `src/events.rs`:
  - Add `ProjectEvent::SetPieceLocked { id, locked }`, returning `ProjectChange::Edited`.
  - `SetPiecePoints` and `RemovePiece` return `Err(Message::PieceLocked { name })` for a locked piece, so the rule holds in `Project::apply` and not only in the UI.
- `src/i18n/` (`mod.rs`, `en.rs`, `uk.rs`): add `Label::Layout`, `Label::LockPiece`, `Label::UnlockPiece` and `Message::PieceLocked { name }`. Reuse `Label::Pieces` and `Label::DeletePiece`.

### Panels

- `src/panels/mod.rs`: move `icon_button` here from `panels/tools/mod.rs`, generalized to take a size and a paint closure. The Tools panel calls the shared version.
- `src/panels/layout/mod.rs` (new): `show_panel(ui, project, selected: &mut Option<u64>, language) -> Option<ProjectEvent>`.
  - Layout: `CollapsingHeader` root, then a `ScrollArea`, with a `push_id(piece.id)` row for each piece.
  - Row: buttons laid out right to left, so a long name truncates instead of pushing them out.
  - Icons: painted padlock (open or closed) and trash can, in the style of the tool icons, with no dependency on font glyphs.
  - Delete button: disabled on locked pieces.
  - Returning the event follows how `tools::show_panel` stays independent of `EditorState`.

### `src/editor.rs`

- `editor_ui`: add `SidePanel::right("layout")` before the central panel and send the returned event.
- `pieces_list`: remove the list and the delete button, and rename it `selected_piece_details`.
- Delete key:
  - Move it out of `handle_select` into a `handle_delete_key` called from `editor_ui` after the panels, so it works after selecting in the panel whatever the tool or view.
  - Skip it while a text box has focus (existing check) or while a draw or shape draft is in progress.
  - On a locked piece the event is rejected and the status bar shows the "locked" error, which tells the user why nothing happened.
- `on_project_change`: `PieceRemoved(id)` clears `selected` and `drag` only if they refer to `id`. Today it always clears them, which would drop the selection when deleting a different row by its icon.
- `begin_drag`: a locked piece is selected but no drag starts.
- `draw_pieces`: no vertex handles for a selected locked piece.
- `displayed`: keep the piece's other fields when substituting drag points (`PatternPiece { points, ..piece.clone() }`).

### Docs

- `docs/project-index.md`: add the new file, the moved symbols and the new test count.
- `CLAUDE.md`: under the format version invariant, note that additive `#[serde(default)]` fields don't need a bump.

## Tests (`events.rs`, `persistence.rs`)

- Locking a piece marks the project modified.
- `SetPiecePoints` and `RemovePiece` on a locked piece are rejected and leave the project unchanged; renaming is allowed; both work again after unlocking.
- Lock state survives a save and load round trip.
- A file without `locked` loads as unlocked.

## Verification

1. `cargo test --lib --no-default-features -q`
2. `cargo check --bin leathercraft --message-format=short`, `cargo clippy`, `cargo fmt`
3. Manual, with `cargo run`:
   - Add three pieces. Selecting in the panel and on the canvas stays in sync.
   - The delete icon on an unselected row removes that row and keeps the current selection.
   - Select in the panel with the Draw tool active, then press Delete or Backspace: the piece is removed. Doing the same during a draft, or while typing in the rename box, removes nothing.
   - On a locked piece, dragging does nothing, no vertex handles show, the delete icon is disabled and the Delete key shows the error. Rename still works.
   - Save, reopen: the lock is kept. An older file opens with everything unlocked.
   - Long names truncate without pushing the buttons out. Switch to Ukrainian and check the new text.
