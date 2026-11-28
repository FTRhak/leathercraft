# Tests

There are 207 tests inline in the library crate, plus 8 in the binary (`settings::hotkeys`, `editor::ruler`, `editor::background` and `panels::seam_link::table`, run with `cargo test --bin leathercraft`, which compiles Bevy):

| Module | Tests |
|---|---:|
| `model` | 9 |
| `background` | 9 |
| `shapes` | 49 |
| `boolean` | 11 |
| `material` | 3 |
| `align` | 14 |
| `snap` | 10 |
| `events` | 22 |
| `history` | 14 |
| `i18n` | 8 |
| `exports` | 11 |
| `persistence` | 8 |
| `seams` | 10 |
| `sewing` | 4 |

Apart from shortcut parsing, `document`, `editor`, `preview`, `settings` and `panels` have no tests, so changes there need `cargo check` and a manual run.
