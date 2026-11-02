# Деталь = основна форма + дочірні примітиви з булевими операціями

## Контекст

Зараз `PatternPiece` — це готовий полігон `points` плюс `holes`; отвори й надрізи «запікаються» у геометрію (`PatternPiece::cut`, `AddHole`), і після цього їх не можна змінити. Потрібна параметрична модель: деталь складається з **основної форми** (полігон, прямокутник або овал) і впорядкованого списку **дочірніх примітивів** (прямокутник, коло/овал; полігон теж дозволено, бо це той самий `Shape`). Кожен дочірній примітив має свою булеву операцію (вирізати / об'єднати / перетнути), операції застосовуються в порядку списку. Дочірні примітиви можна будь-коли виділити, перемістити, змінити розмір, операцію та порядок.

Рішення користувача:
- примітиви зберігаються **параметрично** (центр, розміри, поворот);
- основа — **полігон або примітив**;
- якщо результат порожній або розпадається на кілька частин — **зміна приймається**, деталь підсвічується як помилкова, експорт блокується;
- старі файли (v1) **не підтримуються**: `PROJECT_VERSION = 2`, без міграції.

## Модель (lib)

### `src/shapes.rs` — параметричні форми
```rust
pub enum Shape {
    Polygon { points: Vec<Point> },
    Rectangle { center: Point, width: f64, height: f64, rotation: f64 }, // rotation у радіанах
    Ellipse { center: Point, radius_x: f64, radius_y: f64, rotation: f64 },
}
```
Serde: `#[serde(tag = "kind", rename_all = "snake_case")]`. Методи: `ring() -> Vec<Point>` (еліпс — `CIRCLE_SEGMENTS` = 72 точки), `translate(dx, dy)`, `center()`, `bounds()`. Існуючі `rectangle(a, b)` / `circle(center, edge)` перетворити на конструктори `Shape` (`Shape::rectangle_from_corners`, `Shape::ellipse_from_center(center, edge, circle: bool)`), повертають `Option<Shape>` (None — без площі). Тести shapes оновити.

### `src/model.rs`
```rust
pub enum BooleanOp { Subtract, Union, Intersect }          // serde snake_case
pub struct ChildShape { pub id: u64, pub op: BooleanOp, pub shape: Shape }
pub struct PatternPiece { id, name, base: Shape, children: Vec<ChildShape>, locked, material }
```
- Прибрати `points`, `holes`, `Cut`, `cut`, `outline_without`, `hole_at`, `hole_indices`, `validate_holes`, `MAX_HOLES_PER_PIECE` → `MAX_CHILDREN_PER_PIECE = 512`.
- `PatternPiece::next_child_id()`, `child(id)`, `child_index(id)`.
- `PatternPiece::bounds()` — межі основи та всіх дочірніх (для fit view і хіт-тестів редагування).
- `Project::validate` перевіряє **лише параметри**: `validate_ring` для `base.ring()` і кожного дочірнього `ring()` (скінченні координати, кількість точок, площа), додатні розміри/радіуси та скінченний поворот (новий `Message::InvalidShapeSize { name }`), унікальні ненульові id дочірніх, ліміт кількості. Результат булевих операцій **не** валідується тут (рішення «дозволити, показати помилку»).
- `PROJECT_VERSION = 2`.

### Новий `src/boolean.rs` — обчислення результату
```rust
pub struct Region { pub outline: Vec<Point>, pub holes: Vec<Vec<Point>> }
impl PatternPiece { pub fn region(&self) -> Result<Region, Message> }
```
- Старт: shapes = [[base.ring()]]; для кожного child по порядку: `shapes.overlay(&child.ring(), rule, FillRule::NonZero)` з `i_overlay` (`OverlayRule::Difference | Union | Intersect`) — той самий API, що зараз у `outline_without` (model.rs:249).
- Порожньо на будь-якому кроці → `Message::ResultEmpty { name }`; кінцевий результат з >1 shape → `Message::ResultSplit { name }`.
- Одна shape: `outline = shape[0]`, `holes = shape[1..]`; округлення координат як `CUT_PRECISION_MM` (перенести константу сюди).
- `Region`: `rings()`, `vertices()`, `hole_indices()`, `bounds()`, `contains(point)` — перенести з `PatternPiece` (вони потрібні canvas/preview/exports).
- Субтракт, що не перетинається з деталлю, — просто без ефекту (не помилка).

### `src/events.rs`
Замінити `AddPiece{points}`, `SetPieceGeometry`, `AddHole`, `RemoveHole` на:
- `AddPiece { name, base: Shape }`
- `SetBaseShape { id, shape }`
- `MovePiece { id, dx, dy }` (основа + всі дочірні)
- `AddChild { id, op, shape }` → `ProjectChange::ChildAdded { id, child }`
- `SetChildShape { id, child, shape }`
- `SetChildOp { id, child, op }`
- `ReorderChild { id, child, index }` (нова позиція у списку)
- `RemoveChild { id, child }` → `ProjectChange::ChildRemoved { id }`
Всі через `edit()` + `unlocked_piece_mut`; новий `Message::ChildNotFound { name }`. Решта (`Rename`, `Locked`, `Material`, `RemovePiece`) без змін. `ProjectChange::HoleAdded/HoleRemoved/EdgeCut` прибрати.

### `src/history.rs`
`describe` — мітки для нових подій (`ActionMovePiece`, `ActionEditBase`, `ActionAddChild`, `ActionEditChild`, `ActionSetChildOp`, `ActionReorderChild`, `ActionRemoveChild`). `MergeKey`: послідовні `SetChildShape` одного child і `SetBaseShape` однієї деталі об'єднуються (як зараз перейменування) — щоб числові поля панелі не засмічували історію.

### `src/exports/*`
`validate_export`: крім `project.validate()`, для кожної деталі `piece.region()`; помилка → новий `ExportError::InvalidPiece(Message)` (конвертується у `Message` як є). `to_svg`/`to_dxf` ітерують `region.rings()`; межі SVG — об'єднання `region.bounds()`.

### `src/persistence.rs`
Без змін логіки; оновити тести (замість `holes_round_trip...` — round-trip деталі з дочірніми примітивами всіх трьох операцій; файл версії 1 відхиляється з `UnsupportedVersion`).

### `src/i18n/{mod,en,uk}.rs`
Прибрати `Hole*`, `SelectPieceForHole`, `Label::HoleMode`, `ActionAddHole/RemoveHole`. Додати: `Label::{OpNone/NewPiece, OpSubtract, OpUnion, OpIntersect, ShapeRectangle, ShapeEllipse, ShapePolygon, Base, CenterX, CenterY, Width, Height, RadiusX, RadiusY, Rotation, MoveUp, MoveDown, DeleteChild}`, `Message::{ResultEmpty, ResultSplit, ChildNotFound, InvalidShapeSize, TooManyChildren, SelectPieceForChild, ChildAdded, ChildDeleted, Action…}`. Компілятор підсвітить пропущені гілки.

## Редактор (bin)

### Стан виділення — `editor/editor_state_struct.rs`
`selected: Option<u64>` + `selected_hole` → `selected: Option<u64>` + `selected_child: Option<u64>` (id дочірнього, валідний лише разом з `selected`; аналог `selected_hole_index` → `selected_child_id`). `hole_mode: bool` → `op_mode: Option<BooleanOp>` (None — інструменти створюють нову деталь). `on_project_change` обробляє `ChildAdded` (виділити новий child) / `ChildRemoved`. `displayed(piece)` повертає `Cow`/клон із `drag.current`.

### Перетягування — `editor/drag_state_struct.rs`, `editor/drag.rs`
`DragTarget`: `Piece`, `BaseVertex(usize)` (лише для полігональної основи), `BaseHandle(Handle)`, `Child(u64)`, `ChildVertex(u64, usize)`, `ChildHandle(u64, Handle)`, де `Handle` — кутові ручки прямокутника / 4 ручки осей еліпса (у локальних координатах з урахуванням повороту; протилежна сторона фіксована). `DragState { piece_id, target, start, original: PatternPiece, current: PatternPiece, released }`.
`finish_drag` відправляє рівно одну подію: `MovePiece`, `SetBaseShape` або `SetChildShape`.
Пріоритет хіт-тесту на виділеній деталі: ручки виділеного child → ручки основи → тіло дочірнього (останній у списку зверху; хіт по `ring_contains` його `ring()` або близькості до контуру) → деталь. Невиділена деталь — по `region().contains` або, якщо регіон невалідний, по `base`.

### Canvas — `editor/canvas.rs`
- `select_at`: дочірні виділеної деталі мають пріоритет, інакше деталь (як зараз з отворами, `canvas.rs:139`).
- `commit_shape`: `op_mode = None` → `AddPiece { base }`; інакше `AddChild { id, op, shape }` або `SelectPieceForChild`.
- `Tool::Circle` → `Tool::Ellipse`: тягнути від центру, `radius_x=|dx|`, `radius_y=|dy|`, Shift — коло (радіус = відстань). `Tool::shape_points` → `Tool::shape(start, end, shift) -> Option<Shape>`.
- `draw_pieces`: заливка й контур з `region()`; для виділеної деталі — пунктирні контури основи та всіх дочірніх (колір за операцією: вирізати/об'єднати/перетнути), виділений child яскравіше + його ручки. Невалідний регіон: червоний пунктир основи+дочірніх і текст помилки біля назви. `region()` рахується щокадру — для 72-точкових примітивів це дешево; кеш не додаємо, поки не з'явиться потреба.

### Панелі
- `panels/tools/mod.rs`: кнопка «Отвір» → група з 4 icon-кнопок режиму операції (нова деталь / вирізати / об'єднати / перетнути), нові намальовані іконки; іконка еліпса для `Tool::Ellipse`.
- `panels/layout/mod.rs`: дерево — рядок деталі розгортається (`CollapsingHeader`), всередині рядок «Основа» і рядки дочірніх у порядку застосування: комбо операції (`SetChildOp`), ▲/▼ (`ReorderChild`), кошик (`RemoveChild`), клік — виділити child (`selected_child`). `show_panel` отримує `&mut EditorState`-подібні поля `selected`/`selected_child`.
- `editor/mod.rs::selected_piece_details`: секція параметрів виділеної форми (основи або child): центр X/Y, ширина/висота або радіуси, поворот (градуси в UI, радіани в моделі), у одиницях `Units`. Зміни → `SetBaseShape`/`SetChildShape` (злиття в історії, див. вище).
- `editor/ui.rs` Delete: виділений child → `RemoveChild`, інакше `RemovePiece` (замінює гілку `RemoveHole`, `ui.rs:251`).

### 3D — `preview/mod.rs`
`build_piece_mesh` бере `piece.region()` (`vertices()`, `hole_indices()`); деталі з невалідним регіоном пропускаються.

## Документація
Оновити `CLAUDE.md` (Invariants: «A piece is a closed polygon…» → опис base+children, результат обчислюється, невалідний результат дозволено в проєкті, але блокує експорт/3D; модулі `boolean`) і `docs/project-index.md` (нові файли/символи, таблиця тестів).

## Порядок роботи
1. lib: `shapes::Shape` → `model` → `boolean` → `events` → `history` → `exports` → `persistence` → `i18n`; тести lib зелені.
2. bin: стан і drag → canvas → tools/layout/details панелі → preview → `cargo check`.
3. Документація.

## Перевірка
- `cargo test --lib --no-default-features -q` — нові тести:
  - `boolean`: subtract усередині → отвір; subtract через край → виїмка; union виступу; intersect; порядок має значення (subtract потім union ≠ union потім subtract); порожній результат → `ResultEmpty`; розріз навпіл → `ResultSplit`; subtract поза деталлю — без змін.
  - `events`: `AddChild`/`SetChildShape`/`SetChildOp`/`ReorderChild`/`RemoveChild`, заблокована деталь відхиляє зміни, `MovePiece` рухає дочірні, перетягування child у стан «розпалось» приймається.
  - `exports`: невалідний регіон → `ExportError::InvalidPiece`; SVG містить отвір як підшлях.
  - `persistence`: round-trip v2, v1 відхиляється.
  - `history`: злиття послідовних `SetChildShape`.
- `cargo check --bin leathercraft --message-format=short`, `cargo clippy`, `cargo fmt`.
- Ручний запуск `cargo run`: створити прямокутну деталь, у режимі «вирізати» додати овал усередині (отвір) і через край (виїмка), в режимі «об'єднати» — прямокутний виступ; змінити порядок у Layout і побачити різницю; перетягнути child (рух і ручки розміру), змінити поворот у полях; розрізати деталь навпіл — червона підсвітка, експорт SVG показує помилку; undo/redo; збереження і повторне відкриття; 3D-превʼю.
