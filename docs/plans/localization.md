# Localization plan: English and Ukrainian

## Context

All user-facing text is hardcoded English: egui labels and dialogs in `src/editor.rs` and `src/preview.rs`, and error strings in the library (`model`, `events`, `persistence`, `exports`). Library errors are preformatted `String`s, so the UI can't translate them. The default piece name (`Piece {id}`) is also built inside `Project::apply`.

The goal is an app fully usable in English or Ukrainian, with language switching at runtime and a remembered choice. The library must stay serde-only, and geometry and the file format must not change.

First implementation step: save this plan as `docs/localization.md`.

## Decisions

- **Hand-written typed catalog, no i18n crate.** Translations are exhaustive `match`es in Rust, so a missing Ukrainian string is a compile error. This adds no library dependency, and the catalog is tested by `cargo test --lib --no-default-features`. Fluent would add a runtime dependency to the library or move the catalog tests behind the Bevy build.
- **Errors carry structured messages, not text.** The library returns a `Message` value and the UI renders it in the current language.
- **Status is stored untranslated.** `EditorState.status` holds a `Message`, so the status bar follows a language switch immediately.
- **Language is an interface preference.** It lives in `EditorState` and a settings file, never in `Project`. `PROJECT_VERSION` does not change.
- **Default piece names are document content.** They are written once in the language active at creation and are not retranslated afterwards.

## Implementation

### 1. Add the catalog to the library

Add `src/i18n/mod.rs`, `en.rs` and `uk.rs`, and export `pub mod i18n` from `src/lib.rs`.

```rust
pub enum Language { English, Ukrainian }   // Default: English; tag(), from_tag(), native_name(), ALL
pub enum Label { New, Open, Save, /* ... */ }      // Fixed UI text: returns &'static str
pub enum Message {                         // Text with arguments: returns String; Clone + PartialEq
    Ready,
    PieceNotFound { id: u64 },
    CouldNotRead { path: String, detail: String },
    PointCount { count: usize },
    // ...
}

impl Language {
    pub fn label(self, label: Label) -> &'static str;
    pub fn message(self, message: &Message) -> String;
    pub fn unit_symbol(self, units: Units) -> &'static str;   // mm/in, мм/дюйм
}
```

- `Display` and `Error` on `Message` render English, for logs and tests.
- `uk.rs` has a plural helper for the one/few/many forms: 1 точка, 2 точки, 5 точок, 11 точок, 21 точка.
- OS and serde error details pass through as untranslated `detail` arguments.

### 2. Move library errors to `Message`

| File | Change |
|---|---|
| `src/model.rs` | `validate` returns `Result<(), Message>`. Remove `Units::symbol`, since `Language::unit_symbol` replaces it. |
| `src/persistence.rs` | `save_project` and `load_project` return `Result<_, Message>`. |
| `src/events.rs` | `apply`, `edit` and `piece_mut` return `Result<_, Message>`. `AddPiece` becomes `{ points, name }`, and the UI supplies the name. |
| `src/exports.rs` | `ExportError` stays, with `InvalidProject(Message)`. Add `From<ExportError> for Message`. |

Tests that match on English text, such as `contains("duplicate ID")`, should compare `Message` variants instead. `Display` stays available where text matters.

### 3. Translate the UI in the binary

- **`src/editor.rs`:**
  - `status` becomes a `Message`, and `report` and `report_error` take a `Message`.
  - Replace every literal in `file_buttons`, `confirm_discard`, `project_dialog`, `export`, `tool_controls`, `units_control` and `pieces_list`, passing `Language` to the helpers that need it.
  - `format_length` takes `Language` for the unit symbol.
  - `finish_draft` builds the default name from `project.next_piece_id()`.
  - The "piece added" status uses the piece's actual name from the project.
- **`src/preview.rs`:** `show_controls` takes `Language` for Camera, Orbit, Tilt and Zoom.
- **`src/document.rs`:** the logic doesn't change; `report_error` now receives a `Message`.
- **Language selector:** add a `ComboBox` at the right of the `file_bar` panel, listing each language by its native name (English, Українська). A change updates `EditorState.language` and saves the setting. It does not send a `ProjectEvent`.

### 4. Persist and detect the language

- Add `src/settings.rs` to the binary, storing `settings.json` with `{ "language": "uk" }` in the OS config directory under `leathercraft/`.
- Add `dirs` and `sys-locale` to the `desktop` feature only.
- Startup order: saved setting, then OS locale (`uk` or `uk-*` selects Ukrainian), then English.
- A missing or invalid settings file falls back silently. A failed write is logged and doesn't interrupt the user.

### 5. Update the docs

- Update `CLAUDE.md`:
  - Library functions return `Result<_, Message>`.
  - New user-facing text goes through `i18n`.
  - The library's serde-only rule still holds.
  - Language is interface state.
- Add this plan as `docs/localization.md`.

## Out of scope

- Window title (product name), log messages and SVG/DXF output.
- Native Yes/No button labels in dialogs, which `rfd` and the OS control.
- Decimal commas, because egui `DragValue` fields use a dot and the two should stay consistent.
- OS and serde error details.
- CJK and right-to-left languages.

## Risks

- **Translation quality:** the Ukrainian text will be machine-written and should get a review by a native speaker, especially the leathercraft and CAD terms.
- **Font coverage:** Ukrainian needs `і ї є ґ`. egui's default font should include them, which the manual check below confirms. If any are missing, bundle a font with Cyrillic coverage.
- **Layout:** longer Ukrainian labels may overflow the 220 px side panel or the single-row file bar. Adjust the widths or wrapping if they do.

## Verification

```sh
cargo fmt
cargo clippy
cargo test --lib --no-default-features
cargo check --bin leathercraft
```

New tests:
- Ukrainian plural forms for 1, 2, 5, 11, 21 and 22.
- `Language` tag round trip and locale matching (`uk`, `uk-UA`, `en-US`, unknown).
- `validate` returns the expected `Message` variants.
- `AddPiece` uses the supplied name.
- Rejected events still leave the project unchanged.

Manual check with `cargo run`:
1. Switch to Українська. Every panel, tooltip, hint and file dialog filter changes, and `і ї є ґ` display correctly.
2. Trigger an error, such as opening an invalid file, then switch language. The status text follows.
3. Restart. The language is remembered. Delete the settings file and confirm the OS locale is used.
4. Draw a piece in Ukrainian, save, switch to English and reopen. The name is unchanged, and the file has no new fields.
5. Export SVG and DXF in each language and confirm the output is byte-identical.
