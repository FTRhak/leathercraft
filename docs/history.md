# Undo, redo and history (`history`)

How edits are recorded, undone, redone and restored. The model lives in the library crate, in `src/history.rs`, so it can be tested without Bevy. The binary owns it through `document::ActiveProject` and shows it in the History panel.

| File | Contents |
|---|---|
| `src/history.rs` | `History`, `HistoryCommand`, `MAX_HISTORY`, tests |
| `src/document.rs` | `ActiveProject` (owns the `History`), `HistoryMessage`, `apply_project_events` |
| `src/panels/history/mod.rs` | History panel: `show_panel` |
| `src/editor/ui.rs` | Edit menu (`edit_menu`) and keyboard shortcuts (`handle_history_keys`) |
| `src/editor/editor_state_struct.rs` | `EditorState::on_history_restore` |

## For users

| Action | How |
|---|---|
| Undo | **Edit → Undo**, or Ctrl+Z (Cmd+Z on macOS) |
| Redo | **Edit → Redo**, or Ctrl+Shift+Z or Ctrl+Y |
| Go to any point | Click an entry in the **History** panel |

- The History panel lists every edit of the current document, oldest first. The highlighted entry is the current state. Entries below it have been undone; they are dimmed and can still be redone.
- Making a new edit after undoing discards the dimmed entries.
- The status bar names what was undone, redone or returned to, for example *Undone: Added 'Flap'.*
- Shortcuts don't act while a text field has focus, which keeps its own undo, while a shape is being drawn, or while the new-project dialog is open. The Edit menu still works then.
- Typing a name, or dragging a value or handle of the same shape, makes one entry, not one per keystroke or frame.
- Saving doesn't add an entry. It marks the current entry as the saved one: undoing or redoing back to it clears the unsaved-changes mark.
- **New** and **Open** start a fresh history, so a previous document's edits can't be undone into the new one.
- Only the last 100 entries are kept.
- The history isn't saved in the `.leathercraft` file and is lost when the app closes.

## The model

`History` is a list of entries plus a position. Each entry holds:

| Field | Meaning |
|---|---|
| `action` | A `Message` describing the edit, shown in the panel and status bar |
| `merge` | An optional `MergeKey` that lets the next edit replace this entry |
| `snapshot` | A full clone of the `Project` right after the edit |

Entries are full snapshots, not diffs, so restoring any entry is a single clone and needs no inverse operations. The first entry is the document's starting state (`ActionNewProject` or `ActionOpened`) and can't be undone past.

| Field of `History` | Meaning |
|---|---|
| `position` | Index of the entry that matches the current project |
| `saved` | Index of the entry that matches the file on disk, or `None` once that entry is discarded or dropped |
| `mergeable` | Whether the top entry may absorb the next edit; cleared by saving and by any undo, redo or jump |

### API

| Item | Purpose |
|---|---|
| `History::new(&project, action)` | Starts a history whose only entry is `project`, marked as saved |
| `record(&event, change, &project)` | Records an event that `Project::apply` accepted; `project` is the result |
| `apply(command, &mut project)` | Restores an entry, returning the status `Message`, or `None` if nothing moved |
| `actions()` | The entry labels, oldest first, for the panel |
| `position()`, `can_undo()`, `can_redo()` | State for the panel, menu and shortcuts |
| `HistoryCommand` | `Undo`, `Redo`, or `GoTo(index)`, where `index` is a position in `actions()` |
| `MAX_HISTORY` | 100; the oldest entry is dropped beyond it |

### Recording

`record` branches on the `ProjectChange` returned by `Project::apply`:

| Change | Effect |
|---|---|
| `Created` | Replaces the history with `History::new(project, ActionNewProject)` |
| `Opened` | Replaces the history with `History::new(project, ActionOpened { name })`, where `name` is the file name |
| `Saved` | Sets `saved` to the current position and clears `mergeable`; adds no entry |
| Anything else | Truncates the redo branch, then merges into the top entry or pushes a new one |

For an ordinary edit:

1. Entries after `position` are dropped. If `saved` pointed into them, it becomes `None`, so the project stays modified until the next save.
2. If `mergeable` is set and the top entry has the same `MergeKey` as this edit, the top entry's label and snapshot are replaced and nothing else changes.
3. Otherwise a new entry is pushed, `position` moves to it, and `mergeable` is set.
4. If there are more than `MAX_HISTORY` entries, the oldest is removed and `position` and `saved` shift down.

### Merging

`describe` gives each event its label and merge key. The label's piece name is looked up in the snapshot *before* the event, so a deleted piece is still named.

| Event | Label | Merges with |
|---|---|---|
| `RenamePiece` | `ActionRenamePiece` (new name) | Renames of the same piece |
| `SetPieceMaterial` | `ActionMaterial` | Material changes of the same piece |
| `SetBaseShape` | `ActionEditBase` | Base edits of the same piece |
| `SetChildShape` | `ActionEditChild` | Edits of the same child of the same piece |
| `SetUnits`, `AddPiece`, `MovePiece`, `MovePieces`, `ArrangePieces` (align or distribute), `SetPieceLocked`, `RemovePiece`, `RemovePieces`, `AddChild`, `SetChildOp`, `ReorderChild`, `RemoveChild` | Their own `Action…` message; the group events count their pieces | Never |

A merge chain ends at any other edit, at a save, or at an undo, redo or jump, so the next edit always gets its own entry after one of those.

### Restoring

`apply` resolves the target entry (`position - 1`, `position + 1`, or the given index). It returns `None` and changes nothing when the target is the current entry or out of range, including undo at the first entry and redo at the last. Otherwise it:

1. Builds the status: `Undone` with the label of the entry being left, or `Redone` / `HistoryRestored` with the label of the target.
2. Moves `position` to the target and clears `mergeable`.
3. Replaces the project with the target's snapshot, **keeping the current `path`**, so restoring an entry recorded before a Save As doesn't point the document at the old file.
4. Sets `project.modified` to whether the target differs from the saved entry.

## How the binary uses it

The flow matches the one for project edits in `CLAUDE.md`: the UI never touches the `History` directly.

```text
History panel / Edit menu / shortcuts
        │  HistoryMessage(HistoryCommand)
        ▼
document::apply_project_events
        ├─ ProjectMessages first: Project::apply, then History::record
        └─ HistoryMessages next:  History::apply, set_changed, EditorState::on_history_restore
```

- **Sending a command.** Write `HistoryMessage(command)` with a `MessageWriter<HistoryMessage>`. `panels::history::show_panel` returns `Option<HistoryCommand>` and `PanelViewer` sends it; `edit_menu` and `handle_history_keys` send theirs directly.
- **Reading state.** `ActiveProject::history()` gives a read-only `&History`. Use `can_undo` and `can_redo` to enable menu items.
- **Change detection.** `apply_project_events` calls `set_changed` only when `History::apply` returned a status, so a no-op undo doesn't rebuild the 3D preview.
- **Interface state after a restore.** `on_history_restore` drops in-progress work (polygon draft, shape draft, drag, rename buffer), clears a selection whose piece or child no longer exists, and reports the status.
- **Shortcuts.** Redo is matched before undo, because egui's matching ignores an extra Shift and Ctrl+Z would otherwise also take Ctrl+Shift+Z.

### History panel

`show_panel(ui, history, language)` draws one design-system `ListItem` per entry (`ui_components::list`, from the Brilliant `History List` board) in a vertical scroll area:

- Each row starts with an icon chosen from the entry's `Message` by `action_glyph`. A new `Action…` message gets no icon until it has an arm there.
- The current entry is selected (accent fill and border) and says *Current* at its end. Later entries are dimmed, struck through and say *Undone*.
- Entries have no time yet: the design shows one at the end of each done row, but `History` doesn't record it.
- Clicking a row returns `HistoryCommand::GoTo(index)`. Clicking the current row has no effect, because `History::apply` returns `None`.
- The list scrolls to the current entry only when the position changes, tracked in egui temp data, so the user can scroll freely otherwise.
- Labels are rendered with `language.message` at draw time, so they follow a language change.

## Adding a new edit

Every `ProjectEvent` must have an arm in `History::describe`; the `match` has no wildcard, so the compiler flags a missing one.

1. Add the `ProjectEvent` variant and handle it in `Project::apply` (`src/events.rs`).
2. Add an `Action…` variant to `Message` with the piece name as an argument, and its text in `i18n/en.rs` and `i18n/uk.rs` (see `docs/i18n.md`).
3. Add the arm in `describe`, looking the name up with the `name(id)` helper.
4. Decide on merging. Give it a `MergeKey` only if the UI sends it continuously for one gesture (typing, dragging a field or handle). Add a `MergeKey` variant if none fits, keyed narrowly enough that edits of different objects never merge.
5. Add a test in `history.rs`, using the `run` helper, for the label and, if it merges, for the merge.

An event that doesn't change content, like `Save`, needs its own `ProjectChange` handled in `record` before `describe` is called.

## Rules

- **Only `apply_project_events` calls `record` and `apply`.** Everything else sends a `HistoryMessage` and reads through `ActiveProject::history()`.
- **Record only accepted events.** `record` assumes `project` is the valid result of `Project::apply`; rejected events leave the history untouched.
- **Snapshots are whole projects.** Keep `Project` cheap to clone; anything large added to it is multiplied by up to 100 entries.
- **`path` and `modified` belong to the document, not the entry.** `apply` keeps the path and recomputes `modified` from `saved`; don't restore them from a snapshot.
- **Labels stay `Message`s**, never pre-rendered `String`s, so the panel follows a language change.
- **In-progress work isn't history.** Drafts, drags and a name being typed stay in `EditorState` until an event commits them, and a restore discards them.

## Tests

`src/history.rs` has 10 tests. Run them with:

```sh
cargo test --lib --no-default-features history
```

They cover undo and redo, dropping the redo branch, merging renames and child edits, `modified` following the saved entry, keeping the path, the size limit, `GoTo`, naming a removed piece, and New starting a fresh history. The panel, menu and shortcuts have no tests and need a manual run.
