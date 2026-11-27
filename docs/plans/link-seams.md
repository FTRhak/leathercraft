# Зв'язати шви (`SeamLink`)

## Контекст
Шов у застосунку — це `shapes::StitchLine`, дочірня відкрита лінія деталі. Її отвори обчислюються з `pitch`/`size` і ніде не зберігаються (`docs/stitch-holes.md`). Потрібна функція **«Зв'язати шви»**: користувач вибирає два шви (можна той самий двічі, наприклад для загину клапана) і співставляє їхні отвори для зшивання. Так видно, який отвір із яким зшивається, скільки отворів лишилося без пари і наскільки розходяться довжини.

Дизайн готовий у Brilliant: кадр `Stitch Link Dialog — Зв'язати шви` (`0f4a53ae18e2d8f3`) у `leathercraft-design/main.bl`, поруч із головним вікном `aba8d9c9e24dcb43`. Узгоджені рішення:
- **Різна кількість отворів:** показати попередження, підсвітити отвори без пари й запропонувати «Вирівняти крок» для A або B.
- **Як користувач керує співставленням:** реверс напрямку, зсув початку, діапазон отворів для кожного шва, ручні пари (клік по отвору A, потім по отвору B; Alt+клік прибирає пару).
- **Що в тілі діалогу:** схема зліва, таблиця пар і підсумок справа.

## Рішення
- **Зв'язок — це частина документа.** Він лежить у `Project.seam_links` і зберігається у `.leathercraft`. Поле додається з `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, тому `PROJECT_VERSION` не змінюється, і старі файли відкриваються як раніше.
- **Пари обчислюються, а не зберігаються.** Як і отвори, їх щоразу виводять із параметрів зв'язку. Зберігаються тільки параметри (діапазони, реверс, зсув) і ручні пари. Якщо шов змінили, пари перераховуються самі.
- **Діапазони не валідуються проти кількості отворів.** Інакше редагування шва, що скорочує лінію, відхилялося б. Під час обчислення діапазони обрізаються, а зв'язок, якому довелося обрізати діапазон, позначається як «застарілий» (за аналогією з `ResultEmpty`: проєкт валідний, але редактор показує позначку).
- **Видалення каскадне.** `RemoveChild`, `RemoveChildren`, `RemovePiece` і `RemovePieces` прибирають зв'язки, що посилаються на видалене, в межах тієї самої події. Тому один крок Undo повертає і шов, і його зв'язки.
- **«Вирівняти крок» — окреме редагування.** Кнопка одразу надсилає `SetChildShape` з новим `pitch`. Це звичайний крок історії, а діалог після нього перераховує пари.
- Усе, що стосується геометрії й правил, лежить у бібліотеці і тестується без Bevy. Діалог і малювання — у бінарнику.

## 1. Бібліотека: модель (`src/seams.rs`, новий модуль)

Експортувати `pub mod seams;` з `src/lib.rs`.

```rust
/// One side of a link: a stitch line and the stretch of its holes that takes part.
pub struct SeamSide {
    pub piece: u64,
    pub child: u64,          // a ChildShape whose shape is Shape::StitchLine
    pub from: usize,         // 0-based hole index, inclusive
    pub to: usize,           // inclusive; clamped to hole_count - 1 when pairs are computed
    #[serde(default)]
    pub reversed: bool,
}

pub struct ManualPair { pub a: usize, pub b: usize }   // hole indices on each line

pub struct SeamLink {
    pub id: u64,
    pub a: SeamSide,
    pub b: SeamSide,
    #[serde(default)]
    pub offset: i64,         // shift of B's sequence against A's, in holes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual: Vec<ManualPair>,
}

pub struct HolePair { pub a: usize, pub b: usize, pub manual: bool, pub delta_mm: f64 }

pub struct LinkPairs {
    pub pairs: Vec<HolePair>,
    pub free_a: Vec<usize>,
    pub free_b: Vec<usize>,
    pub length_a: f64,       // path length of the used range, mm
    pub length_b: f64,
    pub clamped: bool,       // a range ran past the line's holes
}
```

`Project` отримує `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub seam_links: Vec<SeamLink>`. Нові id видаються так само, як для дітей і деталей.

### Обчислення пар: `SeamLink::pairs(&self, project: &Project) -> Result<LinkPairs, Message>`
1. Знайти обидві `StitchLine` і взяти `holes()`. Обрізати `from..=to` до наявних отворів; якщо довелося обрізати, `clamped = true`.
2. Скласти послідовності індексів A і B, розвернувши їх, якщо стоїть `reversed`. Послідовність B зсунути на `offset` (при додатному зсуві пропускаються перші отвори B, при від'ємному — перші отвори A).
3. Автопари утворюються, коли обидві послідовності йдуть одночасно (zip).
4. Ручні пари накладаються зверху: спершу видаляються автопари, що використовують хоча б один із двох отворів ручної пари, потім додається сама ручна пара. Ручна пара поза діапазоном ігнорується, і зв'язок позначається як `clamped`.
5. `free_a` і `free_b` — отвори з діапазону, що не потрапили в жодну пару. `delta_mm` — це різниця відстаней уздовж шляху від початку діапазону до кожного з двох отворів пари. Вона показує, як «повзе» шов, і саме це значення стоїть у колонці «Δ мм».

Для відстаней уздовж шляху додати у `src/shapes/stitch.rs` функцію `pub fn hole_offsets(&self) -> Vec<f64>`. Вона обчислюється з того самого обходу runs, що й `holes()`.

### Вирівнювання кроку: `StitchLine::pitch_for_count(&self, count: usize) -> Option<f64>`
Функція шукає крок, при якому `hole_count() == count`. Для одного run це просто `length / (count - 1)`. Для кількох runs використовується бісекція по `pitch` у межах `MIN_PITCH_MM..=MAX_PITCH_MM`, бо `hole_count` монотонно не зростає зі збільшенням кроку. Результат округлюється до 0.01 мм. Якщо підібраний крок порушує `has_valid_parameters` або `holes_overlap`, функція повертає `None`.

### Валідація (`Project::validate`, новий `validate_seam_links`)
| Правило | Помилка (`Message`) |
|---|---|
| `piece`/`child` обох сторін існують, а child — це `StitchLine` | `SeamNotFound` |
| `from <= to` | `InvalidSeamRange` |
| Якщо обидві сторони — той самий шов, діапазони не перетинаються | `SeamRangesOverlap` |
| Ручні пари не повторюють отвір на одному боці | `DuplicateManualPair` |
| id зв'язків унікальні | `DuplicateSeamLink` |

Перевищення кількості отворів **не** вважається помилкою (див. «Рішення»).

### Події (`src/events.rs`)
- `AddSeamLink { link: SeamLink }` (ignore `link.id`, assign a new one) → `ProjectChange::SeamLinkAdded(u64)`.
- `SetSeamLink { link: SeamLink }` → `Edited`.
- `RemoveSeamLink { id: u64 }` → `Edited`.
- У гілках `RemoveChild`, `RemoveChildren`, `RemovePiece` і `RemovePieces` додатково прибрати пов'язані `seam_links`.
- `MoveChildren`, `SetChildShape` та інші зміни геометрії зв'язків не торкаються, бо пари перераховуються самі.

### Історія (`src/history.rs`)
Підписи в `History::describe`: «Зв'язано шви …», «Змінено зв'язок швів …», «Видалено зв'язок швів …» з назвами деталей. Для `SetSeamLink` того самого id — `MergeKey`, щоб кілька змін зсуву підряд злилися в один запис.

## 2. i18n (`src/i18n/{mod,en,uk}.rs`)

- **`Label`:** `LinkSeams` («Зв'язати шви…»), `LinkSeamsTitle`, `LinkSeamsSubtitle`, `SeamA`, `SeamB`, `HolesFrom`, `HolesTo`, `Reverse`, `StartOffset`, `LinkSchematic`, `HolePairs`, `PairAuto`, `PairManual`, `PairFree`, `LinkHint`, `ResetManualPairs`, `Link`, `LengthDifference`, `FreeInB`, `PairsCount`, `SeamLinks` (розділ у Properties), `EditLink`, `RemoveLink`.
- **`Message`:** `SeamNotFound`, `InvalidSeamRange`, `SeamRangesOverlap`, `DuplicateManualPair`, `DuplicateSeamLink`, `HoleCountMismatch { a, b, free }`, `FitPitch { side, pitch }` («Крок A → 3.55 мм»), `LinkPairsButton { count }` («Зв'язати 48 пар»), `SeamOption { piece, index, holes }` («Деталь 1 · Клапан → Шов 1 (48 отв.)»), `SeamLinkStale`, а також підписи історії: `SeamsLinked`, `SeamLinkChanged`, `SeamLinkRemoved`.
- Відмінювання «отвір/отвори/отворів» зробити через наявний механізм множини в `uk.rs`, якщо він є; якщо немає, додати хелпер поруч із `range`.

## 3. Бінарник

### Стан і точка входу
- `EditorState.seam_link_dialog: Option<SeamLinkDraft>`, за зразком `settings_dialog` у `src/editor/editor_state_struct.rs`.
- Меню **Викрійка** (`title_bar::menu(... "pattern" ...)` у `src/editor/ui.rs`, зараз порожнє) отримує пункт «Зв'язати шви…». Він активний, коли в проєкті є хоча б одна `StitchLine`. Якщо зараз вибрано шов, він підставляється як A, і як B теж (той самий двічі; користувач змінить B сам).
- У `dialog_open` (`src/editor/ui.rs:425`) додати `state.seam_link_dialog.is_some()`, щоб полотно й гарячі клавіші не реагували, поки відкрито модальне вікно.
- Гаряча клавіша — поза цим планом. Додати її можна пізніше як `settings::Action::LinkSeams`.

### Діалог: `src/panels/seam_link/mod.rs` (новий модуль панелей)
`pub fn seam_link_dialog(ctx, draft: &mut SeamLinkDraft, project: &Project, language) -> Option<SeamLinkResult>`. Він побудований на `egui::Modal`, як `material::new_project_dialog`, і розмічений за кадром Brilliant:

```rust
pub struct SeamLinkDraft {
    pub editing: Option<u64>,        // Some(id) when opened from an existing link
    pub link: SeamLink,              // the working copy
    pub pending_a: Option<usize>,    // first click of a manual pair
}
pub enum SeamLinkResult { Commit(ProjectEvent), FitPitch(ProjectEvent), Cancel }
```

| Зона дизайну | Реалізація |
|---|---|
| Header | Заголовок, підзаголовок, кнопка ✕ (= Cancel) |
| Картки «Шов A/B» | `ui_components::dropdown` зі списком усіх `StitchLine` (`SeamOption`). Мітки: тип отвору, `pitch`, `length()`. Діапазон «від/до» через `ui_components::input` (UI 1-based, модель 0-based). `Реверс` — toggle-кнопка з `ui_components::button` |
| Зсув початку | Числове поле `offset` (цілі числа, зі знаком) |
| Схема | `seam_schematic(ui, &pairs, …)`: `ui.allocate_painter`. Два шви розгорнуто в горизонталі, отвори — ромби, пари — лінії. Колір лінії залежить від типу пари: авто, ручна, без пари. Номери підписано через кожні 5 отворів і на кінцях. Якщо отворів більше, ніж уміщається (крок < ~6 px), показується зведений вигляд: кожен n-й отвір і всі отвори без пари. Клік по ромбу A ставить `pending_a`, наступний клік по B створює `ManualPair`, Alt+клік по отвору з ручною парою її прибирає |
| Таблиця пар | `egui::ScrollArea` з рядками `#, A, B, Δ мм` і бейджами «Ручна» та «Без пари». Отвори без пари згортаються в діапазони (`49–51`) |
| Підсумок | Три плитки: пари, отвори без пари, різниця довжин |
| Попередження | Показується, коли в діапазонах A і B різна кількість отворів. Кнопки `FitPitch` для A і B за `pitch_for_count`; кнопка ховається, якщо функція повертає `None` |
| Footer | «Скинути ручні пари» (очищає `manual`), «Скасувати», «Зв'язати N пар» — primary. Під час редагування існуючого зв'язку кнопка зветься «Зберегти» |

`SeamLinkResult` обробляється в `editor_ui` поряд із `settings_dialog`:
- `Commit` надсилає `AddSeamLink` або `SetSeamLink` через `ProjectMessage` і закриває діалог;
- `FitPitch` надсилає `SetChildShape` і залишає діалог відкритим (пари перераховуються в наступному кадрі);
- `Cancel` закриває діалог.

Якщо `Project::apply` відхиляє подію (наприклад, через `SeamRangesOverlap`), повідомлення йде в статус, як для інших подій. Крім того, діалог сам заздалегідь перевіряє перекриття діапазонів, коли обидва боки — той самий шов, показує текст помилки під картками і вимикає кнопку «Зв'язати».

### Кольори (`src/ui_components/palette/`)
Новий файл `seam_link.rs` зі структурою `SeamLinkPaint`: `link_auto`, `link_manual`, `link_free`, `hole_fill`, `warning_bg`, `warning_border`, `warning_text`, `row_manual`, `row_free`. Значення для темної теми взяти з дизайну (`#78C3D7`, `#F0A04B`, `#E5604F`, `#2B2216`, `#5C4320`, `#F3C58A`, `#2E2414`, `#2E1A1A`), для світлої — підібрати відповідники. Літерали кольорів у коді діалогу заборонені (`docs/design.md`).

### Іконки
У `ui_components::icons::glyph` додати `LINK` і `REVERSE` (за процесом із `docs/icons.md`).

### Properties шва
У `src/panels/properties/stitch.rs`, коли вибрано `StitchLine`, додати розділ «Зв'язки (N)». Для кожного зв'язку з цим швом показати рядок «↔ Деталь 2 · Шов 1 · 48 пар» з кнопками «Змінити» (відкриває діалог з `editing: Some(id)`) і «Видалити» (`RemoveSeamLink`). Якщо `pairs().clamped`, рядок отримує попереджувальну позначку `SeamLinkStale`.

### Полотно (необов'язковий крок, після основного)
`draw_seam_links` у `src/editor/canvas.rs` малює пари тонкими лініями між центрами отворів, коли вибрано один зі зв'язаних швів. Її викликати після `draw_stitch_holes`, а кольори брати з `SeamLinkPaint`.

## 4. Тести (`cargo test --lib --no-default-features`)
| Модуль | Що перевірити |
|---|---|
| `shapes::stitch` | `hole_offsets` зростають і закінчуються на `length()`. `pitch_for_count` дає точну кількість для одного run і для ламаної лінії, а для недосяжної кількості повертає `None` |
| `seams` | Однакові кількості дають усі пари, Δ ≈ 0 для однакових ліній. Реверс розвертає B. Додатний і від'ємний `offset`. Ручна пара витісняє дві автопари. Отвори без пари при 48 і 52. `clamped` після скорочення шва. Той самий шов із діапазонами, що не перетинаються |
| `model` | Кожна помилка з таблиці валідації |
| `events` | `AddSeamLink` / `SetSeamLink` / `RemoveSeamLink`. Видалення шва чи деталі каскадно прибирає зв'язок, а Undo в `History` повертає і шов, і зв'язок. Відхилена подія лишає проєкт без змін |
| `persistence` | Round-trip зі зв'язком. Файл без зв'язків не містить ключа `seam_links`. Старий файл без поля відкривається |
| `history` | Підписи нових подій, злиття послідовних `SetSeamLink` |

Діалог, схему й малювання перевіряти вручну.

## 5. Документація
- `docs/seam-links.md` — опис функції в стилі `docs/stitch-holes.md`: таблиця файлів, розділ «Для користувачів», модель, алгоритм пар, валідація, тести.
- `docs/project-index.md` — нові модулі `seams`, `panels/seam_link`, `palette/seam_link.rs`, змінені файли.
- `CLAUDE.md` — пункт `seams` у списку модулів бібліотеки. До інваріантів додати: «зв'язок швів зберігає параметри, а не пари; пари обчислюються».

## Порядок виконання
1. `seams.rs`: модель, `pairs`, `hole_offsets`, `pitch_for_count` разом із тестами.
2. `Project.seam_links`, валідація, події, каскадне видалення, історія, persistence разом із тестами.
3. Записи i18n (компілятор вкаже на пропуски в обох каталогах).
4. Палітра, іконки, діалог, пункт меню.
5. Розділ у Properties.
6. (Необов'язково) лінії зв'язків на полотні.
7. Документація.

## Перевірка
- `cargo fmt`, `cargo clippy`, `cargo check --bin leathercraft`, `cargo test --lib --no-default-features`.
- `cargo run`:
  1. Намалювати дві деталі зі швами різної довжини. Відкрити **Викрійка → Зв'язати шви…**, вибрати шви. Мають з'явитися попередження з кількостями й червоні отвори без пари.
  2. Натиснути «Крок B → …»: кількості зрівнюються, попередження зникає, в Історії з'являється «Змінено фігуру».
  3. Перевірити реверс і зсув на схемі й у таблиці. Створити ручну пару кліками, прибрати її через Alt+клік.
  4. Вибрати той самий шов двічі з діапазонами, що перетинаються: має з'явитися помилка й вимкнена кнопка. Розвести діапазони — зв'язок створюється.
  5. Зберегти, відкрити заново: зв'язок на місці. Видалити шов, а потім Undo: зв'язок повертається.
  6. Перемкнути мову й тему: усі тексти перекладено, кольори беруться з палітри.
