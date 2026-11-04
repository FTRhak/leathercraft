use super::{Label, Message};
use crate::align::{Arrange, Axis, Edge};
use crate::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Origin, Strength, Texture, ThicknessCategory, Treatment, WaterResistance,
};
use crate::model::{BooleanOp, Units};
use crate::shapes::{HoleKind, MAX_HOLE_MM, MAX_PITCH_MM, MIN_HOLE_MM, MIN_PITCH_MM};

pub(super) fn label(label: Label) -> &'static str {
    match label {
        Label::File => "Файл",
        Label::View => "Вигляд",
        Label::Edit => "Редагування",
        Label::Window => "Вікно",
        Label::New => "Створити",
        Label::Open => "Відкрити…",
        Label::Save => "Зберегти",
        Label::SaveAs => "Зберегти як…",
        Label::ExportSvg => "Експорт SVG…",
        Label::ExportDxf => "Експорт DXF…",
        Label::Editor2d => "2D-редактор",
        Label::Preview3d => "3D-перегляд",
        Label::Language => "Мова",
        Label::UnsavedChanges => "Незбережені зміни",
        Label::DiscardChanges => "Відкинути незбережені зміни в цьому проєкті?",
        Label::ProjectFileType => "Проєкт Leathercraft",
        Label::Tools => "Інструменти",
        Label::Select => "Вибір",
        Label::Draw => "Малювання",
        Label::Rectangle => "Прямокутник",
        Label::Ellipse => "Овал",
        Label::Polygon => "Полігон",
        Label::RegularPolygon => "Багатокутник",
        Label::Edge => "Грань",
        Label::Centerline => "Осьова лінія",
        Label::CenterlineHint => {
            "Перетягуйте від одного кінця до іншого, щоб додати осьову лінію до вибраної деталі. Shift — поворот кроком 15°. Esc скасовує."
        }
        Label::LineType => "Тип лінії",
        Label::LineSolid => "Суцільна",
        Label::LineDashed => "Штрихова",
        Label::LineDashDot => "Штрихпунктирна",
        Label::DashLength => "Штрих",
        Label::GapLength => "Проміжок",
        Label::StitchHoles => "Отвори для зшивання",
        Label::StitchHolesHint => {
            "Клацайте кожну вершину лінії шва на вибраній деталі. Подвійне клацання або Enter завершує, Backspace прибирає останню вершину, Shift — поворот кроком 15°. Esc скасовує."
        }
        Label::HoleType => "Тип отвору",
        Label::HoleKind(kind) => hole_kind(kind),
        Label::Pitch => "Крок",
        Label::PitchPresets => "Пробійник",
        Label::HoleSize => "Розмір отвору",
        Label::StitchLength => "Довжина",
        Label::HoleCount => "Отворів",
        Label::ActualPitch => "Фактичний крок",
        Label::StartX => "Початок X",
        Label::StartY => "Початок Y",
        Label::EndX => "Кінець X",
        Label::EndY => "Кінець Y",
        Label::Angle => "Кут, °",
        Label::RectangleHint => "Перетягуйте від кута до кута. Esc скасовує.",
        Label::EllipseHint => "Перетягуйте від центру назовні. Shift — коло. Esc скасовує.",
        Label::RegularPolygonHint => {
            "Перетягуйте від центру до вершини. Shift — поворот кроком 15°. Esc скасовує."
        }
        Label::NewPieceMode => "Нова деталь: інструменти малювання створюють деталь",
        Label::Operation(op) => operation(op),
        Label::OperationHint(op) => operation_hint(op),
        Label::Base => "Основа",
        Label::CenterX => "Центр X",
        Label::CenterY => "Центр Y",
        Label::Width => "Ширина",
        Label::Height => "Висота",
        Label::RadiusX => "Радіус X",
        Label::RadiusY => "Радіус Y",
        Label::Radius => "Радіус",
        Label::Sides => "Кількість граней",
        Label::Rotation => "Поворот, °",
        Label::EdgeLength => "Довжина",
        Label::CornerRadius => "Радіус заокруглення",
        Label::ToPolygon => "У многокутник",
        Label::MoveUp => "Застосувати раніше",
        Label::MoveDown => "Застосувати пізніше",
        Label::DeleteChild => "Видалити фігуру",
        Label::DrawHint => {
            "Клацайте, щоб додавати точки. Клацніть першу точку або натисніть Enter, щоб замкнути контур; Esc скасовує."
        }
        Label::EdgeHint => {
            "Клацніть грань полігона, щоб задати її довжину, або вершину вибраної деталі, щоб заокруглити кут, на панелі властивостей. Перетягуйте вершину, щоб перемістити її. Esc знімає вибір."
        }
        Label::SelectHint => {
            "Перетягуйте деталь або вершину вибраної деталі. Shift+клік додає або прибирає деталь, Ctrl+A вибирає всі. Del видаляє вибрані деталі."
        }
        Label::NavigationHint => {
            "Перетягування правою або середньою кнопкою зсуває вигляд; коліщатко змінює масштаб."
        }
        Label::SnapMillimeters => "Прив’язка (мм)",
        Label::ZoomToFit => "Показати все",
        Label::DisplayUnits => "Одиниці виміру",
        Label::Pieces => "Деталі",
        Label::DeletePiece => "Видалити деталь",
        Label::Layout => "Макет",
        Label::LockPiece => "Заблокувати деталь",
        Label::UnlockPiece => "Розблокувати деталь",
        Label::Material => "Матеріал",
        Label::Canvas => "Полотно",
        Label::Properties => "Властивості",
        Label::ClosePanel => "Закрити",
        Label::FloatPanel => "Відкріпити",
        Label::ResetPanels => "Скинути панелі",
        Label::History => "Історія",
        Label::Align => "Вирівнювання",
        Label::AlignObjects => "Вирівнювання об’єктів",
        Label::DistributeObjects => "Розподіл об’єктів",
        Label::DistributeSpacing => "Розподіл проміжків",
        Label::Arrange(arrange) => self::arrange(arrange),
        Label::AlignTo => "Вирівняти відносно",
        Label::AlignToSelection => "Виділення",
        Label::AlignToKeyObject => "Ключовий об’єкт",
        Label::AlignToProject => "Межі проєкту",
        Label::AlignToParent => "Батьківська фігура",
        Label::KeyObjectHint => "Ключовий об’єкт — основна вибрана деталь; вона лишається на місці",
        Label::KeyShapeHint => "Ключовий об’єкт — основна вибрана фігура; вона лишається на місці",
        Label::ParentHint => "Фігури вирівнюються відносно базової фігури своєї деталі",
        Label::Spacing => "Проміжок",
        Label::SpacingAuto => "Авто",
        Label::Undo => "Скасувати",
        Label::Redo => "Повторити",
        Label::SelectAll => "Вибрати все",
        Label::Settings => "Налаштування…",
        Label::SettingsTitle => "Налаштування",
        Label::DefaultMaterial => "Типовий матеріал",
        Label::Hotkeys => "Гарячі клавіші",
        Label::PressKeys => "Натисніть клавіші… (Esc — скасувати)",
        Label::AddShortcut => "Додати комбінацію",
        Label::RemoveShortcut => "Видалити комбінацію",
        Label::ResetToDefaults => "Скинути до типових",
        Label::ShortcutConflict => "Комбінації, виділені червоним, призначено кілька разів",
        Label::Ok => "OK",
        Label::DeleteSelection => "Видалити вибране",
        Label::ToggleView => "Перемкнути 2D / 3D",
        Label::NoPieceSelected => "Деталь не вибрано",
        Label::Camera => "Камера",
        Label::Orbit => "Оберт",
        Label::Tilt => "Нахил",
        Label::Zoom => "Масштаб",
        Label::NewProjectTitle => "Новий проєкт: типовий матеріал",
        Label::Create => "Створити",
        Label::Cancel => "Скасувати",
        Label::BasicProperties => "Базові характеристики",
        Label::PhysicalProperties => "Фізичні характеристики",
        Label::OriginField => "Походження",
        Label::TreatmentField => "Тип обробки",
        Label::ThicknessField => "Товщина",
        Label::ColorField => "Колір",
        Label::TextureField => "Фактура",
        Label::StrengthField => "Міцність",
        Label::FlexibilityField => "Гнучкість",
        Label::WaterResistanceField => "Водостійкість",
        Label::HeatResistanceField => "Термостійкість",
        Label::Origin(value) => origin(value),
        Label::OriginHint(value) => origin_hint(value),
        Label::Treatment(value) => treatment(value),
        Label::TreatmentHint(value) => treatment_hint(value),
        Label::Thickness(value) => thickness(value),
        Label::ColorGroup(value) => color_group(value),
        Label::Color(value) => color(value),
        Label::Texture(value) => texture(value),
        Label::Strength(value) => strength(value),
        Label::Flexibility(value) => flexibility(value),
        Label::FlexibilityHint(value) => flexibility_hint(value),
        Label::WaterResistance(value) => water_resistance(value),
        Label::WaterResistanceHint(value) => water_resistance_hint(value),
        Label::HeatResistance(value) => heat_resistance(value),
    }
}

fn origin(value: Origin) -> &'static str {
    match value {
        Origin::Calf => "Теляча",
        Origin::Cow => "Коров’яча",
        Origin::Bull => "Бича",
        Origin::Goat => "Козяча",
        Origin::Sheep => "Овеча",
        Origin::Ram => "Бараняча",
        Origin::Pig => "Свиняча",
        Origin::Horse => "Кінська",
        Origin::Deer => "Оленяча",
        Origin::Elk => "Лосина",
        Origin::Crocodile => "Крокодиляча",
        Origin::Snake => "Зміїна",
        Origin::Ostrich => "Страусина",
    }
}

fn origin_hint(value: Origin) -> &'static str {
    match value {
        Origin::Calf => "М’яка, тонка, еластична",
        Origin::Cow => "Міцна, універсальна",
        Origin::Bull => "Щільна, груба, витривала",
        Origin::Goat => "Легка, гнучка",
        Origin::Sheep => "М’яка, ніжна",
        Origin::Ram => "Тонка, еластична",
        Origin::Pig => "Пориста, доступна",
        Origin::Horse => "Щільна, зносостійка",
        Origin::Deer => "М’яка, міцна",
        Origin::Elk => "Еластична, витривала",
        Origin::Crocodile => "Дорога, декоративна",
        Origin::Snake => "Тонка, візерунчаста",
        Origin::Ostrich => "Фактурна, з характерними фолікулами",
    }
}

fn treatment(value: Treatment) -> &'static str {
    match value {
        Treatment::Raw => "Сира",
        Treatment::Dried => "Сушена",
        Treatment::Soaked => "Вимочена",
        Treatment::VegetableTanned => "Рослинного дублення",
        Treatment::ChromeTanned => "Хромового дублення",
        Treatment::AldehydeTanned => "Альдегідного дублення",
        Treatment::OilTanned => "Жирового дублення",
        Treatment::CombinationTanned => "Комбінованого дублення",
        Treatment::Suede => "Замша",
        Treatment::Nubuck => "Нубук",
        Treatment::Patent => "Лакована",
        Treatment::Impregnated => "Просочена",
    }
}

fn treatment_hint(value: Treatment) -> &'static str {
    match value {
        Treatment::Raw => "Необроблена шкура",
        Treatment::Dried => "Зневоднена",
        Treatment::Soaked => "Оброблена водою",
        Treatment::VegetableTanned => "Дублена рослинними танінами",
        Treatment::ChromeTanned => "Дублена солями хрому",
        Treatment::AldehydeTanned => "Спеціальна хімічна обробка",
        Treatment::OilTanned => "Насичена жирами",
        Treatment::CombinationTanned => "Поєднання різних методів",
        Treatment::Suede => "Оброблена з ворсистою поверхнею",
        Treatment::Nubuck => "Дрібноворсиста шліфована поверхня",
        Treatment::Patent => "Покрита глянцевим лаком",
        Treatment::Impregnated => "Насичена захисними речовинами",
    }
}

fn thickness(value: ThicknessCategory) -> &'static str {
    match value {
        ThicknessCategory::ExtraThin => "Надтонка",
        ThicknessCategory::Thin => "Тонка",
        ThicknessCategory::Medium => "Середня",
        ThicknessCategory::Dense => "Щільна",
        ThicknessCategory::Thick => "Товста",
        ThicknessCategory::ExtraThick => "Надтовста",
        ThicknessCategory::Heavy => "Надміцна",
    }
}

fn color_group(value: ColorGroup) -> &'static str {
    match value {
        ColorGroup::White => "Білі",
        ColorGroup::Black => "Чорні",
        ColorGroup::Brown => "Коричневі",
        ColorGroup::Ginger => "Руді",
        ColorGroup::Red => "Червоні",
        ColorGroup::Yellow => "Жовті",
        ColorGroup::Green => "Зелені",
        ColorGroup::Blue => "Сині",
        ColorGroup::Purple => "Фіолетові",
        ColorGroup::Unusual => "Незвичайні",
    }
}

fn color(value: LeatherColor) -> &'static str {
    match value {
        LeatherColor::White => "Білий",
        LeatherColor::Milky => "Молочний",
        LeatherColor::Cream => "Кремовий",
        LeatherColor::Black => "Чорний",
        LeatherColor::Charcoal => "Вугільний",
        LeatherColor::Graphite => "Графітовий",
        LeatherColor::LightBrown => "Світло-коричневий",
        LeatherColor::Chocolate => "Шоколадний",
        LeatherColor::Chestnut => "Каштановий",
        LeatherColor::Ginger => "Рудий",
        LeatherColor::Copper => "Мідний",
        LeatherColor::Terracotta => "Теракотовий",
        LeatherColor::Red => "Червоний",
        LeatherColor::Burgundy => "Бордовий",
        LeatherColor::Wine => "Винний",
        LeatherColor::Yellow => "Жовтий",
        LeatherColor::Golden => "Золотистий",
        LeatherColor::Amber => "Бурштиновий",
        LeatherColor::Olive => "Оливковий",
        LeatherColor::Emerald => "Смарагдовий",
        LeatherColor::Malachite => "Малахітовий",
        LeatherColor::LightBlue => "Блакитний",
        LeatherColor::Blue => "Синій",
        LeatherColor::DarkBlue => "Темно-синій",
        LeatherColor::Lavender => "Лавандовий",
        LeatherColor::Magenta => "Пурпурний",
        LeatherColor::Violet => "Фіолетовий",
        LeatherColor::Pearl => "Перламутровий",
        LeatherColor::Iridescent => "Райдужний",
        LeatherColor::Metallic => "Металевий",
    }
}

fn texture(value: Texture) -> &'static str {
    match value {
        Texture::Matte => "Матова",
        Texture::SemiMatte => "Напівматова",
        Texture::Rough => "Шорстка",
        Texture::Porous => "Пориста",
        Texture::Sanded => "Шліфована",
        Texture::Embossed => "Тиснена",
        Texture::Perforated => "Перфорована",
        Texture::Scaly => "Луската",
        Texture::Fibrous => "Волокниста",
        Texture::Marbled => "Мармурова",
        Texture::Cracked => "З тріщинами",
        Texture::NaturalScars => "З природними шрамами",
    }
}

fn strength(value: Strength) -> &'static str {
    match value {
        Strength::VeryLow => "Дуже низька",
        Strength::Low => "Низька",
        Strength::Medium => "Середня",
        Strength::High => "Висока",
        Strength::VeryHigh => "Дуже висока",
    }
}

fn flexibility(value: Flexibility) -> &'static str {
    match value {
        Flexibility::VerySoft => "Дуже м’яка",
        Flexibility::Soft => "М’яка",
        Flexibility::SemiSoft => "Напівм’яка",
        Flexibility::Medium => "Середня",
        Flexibility::Stiff => "Жорстка",
        Flexibility::VeryStiff => "Дуже жорстка",
        Flexibility::Rigid => "Дубова",
    }
}

fn flexibility_hint(value: Flexibility) -> &'static str {
    match value {
        Flexibility::VerySoft => "Легко згинається",
        Flexibility::Soft => "Гнучка, приємна на дотик",
        Flexibility::SemiSoft => "Зберігає частину форми",
        Flexibility::Medium => "Помірно гнучка",
        Flexibility::Stiff => "Погано згинається",
        Flexibility::VeryStiff => "Тримає задану форму",
        Flexibility::Rigid => "Надзвичайно тверда",
    }
}

fn water_resistance(value: WaterResistance) -> &'static str {
    match value {
        WaterResistance::None => "Відсутня",
        WaterResistance::Low => "Низька",
        WaterResistance::Moderate => "Помірна",
        WaterResistance::High => "Висока",
        WaterResistance::Waterproof => "Водонепроникна",
        WaterResistance::Absolute => "Абсолютна",
    }
}

fn water_resistance_hint(value: WaterResistance) -> &'static str {
    match value {
        WaterResistance::None => "Швидко вбирає воду",
        WaterResistance::Low => "Слабкий захист",
        WaterResistance::Moderate => "Витримує невелике зволоження",
        WaterResistance::High => "Відштовхує воду",
        WaterResistance::Waterproof => "Не пропускає воду за визначених умов",
        WaterResistance::Absolute => "Фантазійна властивість",
    }
}

fn heat_resistance(value: HeatResistance) -> &'static str {
    match value {
        HeatResistance::None => "Відсутня",
        HeatResistance::Low => "Низька",
        HeatResistance::Moderate => "Помірна",
        HeatResistance::High => "Висока",
        HeatResistance::VeryHigh => "Дуже висока",
    }
}

pub(super) fn message(message: &Message) -> String {
    match message {
        Message::Ready => "Готово.".to_owned(),
        Message::NewProject => "Новий проєкт.".to_owned(),
        Message::Opened { path } => format!("Відкрито {path}."),
        Message::Saved { path } => format!("Збережено {path}."),
        Message::Exported { path } => format!("Експортовано {path}."),
        Message::PieceAdded { name } => format!("Додано: {name}."),
        Message::PieceDeleted => "Деталь видалено.".to_owned(),
        Message::PiecesDeleted { count } => {
            let noun = plural(*count, "деталь", "деталі", "деталей");
            format!("Видалено {count} {noun}.")
        }
        Message::PointCount { count } => {
            let noun = plural(*count, "точка", "точки", "точок");
            format!("{count} {noun}")
        }
        Message::SelectedEdge { number } => format!("Грань {number}"),
        Message::SelectedVertex { number } => format!("Вершина {number}"),
        Message::DefaultPieceName { id } => format!("Деталь {id}"),
        Message::PieceNotFound { id } => format!("Деталь {id} не існує."),
        Message::PieceLocked { name } => format!("Деталь «{name}» заблоковано."),
        Message::AlreadyArranged => "Деталі вже так розташовані.".to_owned(),
        Message::ShapesAlreadyArranged => "Фігури вже так розташовані.".to_owned(),
        Message::ChildAdded { name } => format!("Додано фігуру в деталь «{name}»."),
        Message::ChildDeleted => "Фігуру видалено.".to_owned(),
        Message::ChildrenDeleted { count } => {
            let noun = plural(*count, "фігуру", "фігури", "фігур");
            format!("Видалено {count} {noun}.")
        }
        Message::SelectPieceForChild => "Виберіть деталь, до якої треба додати фігуру.".to_owned(),
        Message::ChildNotFound { name } => format!("У деталі «{name}» немає такої фігури."),
        Message::TooManyChildren { name, max } => {
            format!("Деталь «{name}» може мати не більше {max} фігур.")
        }
        Message::InvalidChildId { name } => {
            format!("Фігура деталі «{name}» має неправильний або повторений ідентифікатор.")
        }
        Message::InvalidShapeSize { name } => {
            format!("Фігури деталі «{name}» мають бути додатного розміру.")
        }
        Message::BaseMustBeClosed { name } => {
            format!("Основа деталі «{name}» має бути замкненою фігурою, а не лінією.")
        }
        Message::TooManyDashes { name, max } => {
            format!("Лінія деталі «{name}» мала б понад {max} штрихів; збільште їх довжину.")
        }
        Message::InvalidHoleParameters { name } => format!(
            "Отвори деталі «{name}» повинні мати розмір від {} до {} мм і крок від {} до {} мм.",
            decimal(MIN_HOLE_MM),
            decimal(MAX_HOLE_MM),
            decimal(MIN_PITCH_MM),
            decimal(MAX_PITCH_MM)
        ),
        Message::HolesOverlap { name } => {
            format!("Отвори деталі «{name}» накладалися б; зменште їх або збільште крок.")
        }
        Message::TooManyHoles { name, max } => {
            format!("Лінія шва деталі «{name}» мала б понад {max} отворів; збільште крок.")
        }
        Message::StitchHolesRow { count } => format!("Отвори для зшивання ({count})"),
        Message::StitchVertices { count } => {
            let noun = plural(*count, "вершина", "вершини", "вершин");
            format!("{count} {noun}")
        }
        Message::ThreadHint { band } => format!(
            "Нитка {} мм, шкіра {} мм.",
            range(band.thread_mm()),
            range(band.leather_mm())
        ),
        Message::LeatherOutsideBand { band } => format!(
            "Шкіра деталі не в межах {} мм, що підходять для цього кроку.",
            range(band.leather_mm())
        ),
        Message::ResultEmpty { name } => format!("Від деталі «{name}» нічого не лишилося."),
        Message::ResultSplit { name } => {
            format!("Деталь «{name}» розпадається на кілька частин.")
        }
        Message::UnsupportedVersion { found, expected } => {
            format!("Версія проєкту {found} не підтримується (очікується {expected}).")
        }
        Message::TooManyPieces { max } => {
            format!("Проєкт може містити не більше ніж {max} деталей.")
        }
        Message::InvalidPieceId { name } => {
            format!("Деталь «{name}» має недійсний або повторюваний ідентифікатор.")
        }
        Message::PieceNameRequired { id } => format!("Деталь {id} повинна мати назву."),
        Message::PointCountOutOfRange { name, max } => {
            format!("Деталь «{name}» повинна мати від 3 до {max} точок.")
        }
        Message::CoordinateOutOfRange { name } => {
            format!("Деталь «{name}» має точку поза підтримуваним діапазоном координат.")
        }
        Message::CoincidentPoints { name } => {
            format!("Деталь «{name}» має дві послідовні точки в одному місці.")
        }
        Message::ZeroArea { name } => {
            format!("Деталь «{name}» повинна охоплювати ненульову площу.")
        }
        Message::ThicknessOutOfRange { name } => format!(
            "Товщина деталі «{name}» повинна бути від {} до {} мм.",
            decimal(MIN_THICKNESS_MM),
            decimal(MAX_THICKNESS_MM)
        ),
        Message::DefaultThicknessOutOfRange => format!(
            "Товщина типового матеріалу повинна бути від {} до {} мм.",
            decimal(MIN_THICKNESS_MM),
            decimal(MAX_THICKNESS_MM)
        ),
        Message::CouldNotEncode { detail } => format!("Не вдалося закодувати проєкт: {detail}"),
        Message::CouldNotRead { path, detail } => {
            format!("Не вдалося прочитати «{path}»: {detail}")
        }
        Message::CouldNotParse { path, detail } => {
            format!("Не вдалося розібрати «{path}»: {detail}")
        }
        Message::CouldNotWrite { path, detail } => {
            format!("Не вдалося записати «{path}»: {detail}")
        }
        Message::ExportEmpty => "Додайте принаймні одну деталь перед експортом.".to_owned(),
        Message::Undone { action } => format!("Скасовано: {}.", self::message(action)),
        Message::Redone { action } => format!("Повторено: {}.", self::message(action)),
        Message::HistoryRestored { action } => format!("Повернуто до: {}.", self::message(action)),
        Message::ActionNewProject => "Новий проєкт".to_owned(),
        Message::ActionOpened { name } => format!("Відкрито {name}"),
        Message::ActionUnits => "Змінено одиниці відображення".to_owned(),
        Message::ActionAddPiece { name } => format!("Додано «{name}»"),
        Message::ActionRenamePiece { name } => format!("Перейменовано на «{name}»"),
        Message::ActionEditBase { name } => format!("Змінено основу «{name}»"),
        Message::ActionMovePiece { name } => format!("Переміщено «{name}»"),
        Message::ActionMovePieces { count } => {
            let noun = plural(*count, "деталь", "деталі", "деталей");
            format!("Переміщено {count} {noun}")
        }
        Message::ActionAlignPieces { count } => {
            let noun = plural(*count, "деталь", "деталі", "деталей");
            format!("Вирівняно {count} {noun}")
        }
        Message::ActionDistributePieces { count } => {
            let noun = plural(*count, "деталь", "деталі", "деталей");
            format!("Розподілено {count} {noun}")
        }
        Message::ActionLockPiece { name } => format!("Заблоковано «{name}»"),
        Message::ActionUnlockPiece { name } => format!("Розблоковано «{name}»"),
        Message::ActionMaterial { name } => format!("Змінено матеріал «{name}»"),
        Message::ActionRemovePiece { name } => format!("Видалено «{name}»"),
        Message::ActionRemovePieces { count } => {
            let noun = plural(*count, "деталь", "деталі", "деталей");
            format!("Видалено {count} {noun}")
        }
        Message::ActionAddChild { name } => format!("Додано фігуру в «{name}»"),
        Message::ActionEditChild { name } => format!("Змінено фігуру «{name}»"),
        Message::ActionSetChildOp { name } => format!("Змінено операцію «{name}»"),
        Message::ActionReorderChild { name } => format!("Змінено порядок фігур «{name}»"),
        Message::ActionRemoveChild { name } => format!("Видалено фігуру з «{name}»"),
        Message::ActionMoveChildren { name, count } => {
            let noun = plural(*count, "фігуру", "фігури", "фігур");
            format!("Переміщено {count} {noun} у «{name}»")
        }
        Message::ActionAlignChildren { name, count } => {
            let noun = plural(*count, "фігуру", "фігури", "фігур");
            format!("Вирівняно {count} {noun} у «{name}»")
        }
        Message::ActionDistributeChildren { name, count } => {
            let noun = plural(*count, "фігуру", "фігури", "фігур");
            format!("Розподілено {count} {noun} у «{name}»")
        }
        Message::ActionRemoveChildren { name, count } => {
            let noun = plural(*count, "фігуру", "фігури", "фігур");
            format!("Видалено {count} {noun} з «{name}»")
        }
    }
}

fn arrange(arrange: Arrange) -> &'static str {
    match arrange {
        Arrange::Align(Axis::Horizontal, Edge::Min) => "Вирівняти ліворуч",
        Arrange::Align(Axis::Horizontal, Edge::Center) => "Вирівняти по центру горизонтально",
        Arrange::Align(Axis::Horizontal, Edge::Max) => "Вирівняти праворуч",
        Arrange::Align(Axis::Vertical, Edge::Min) => "Вирівняти вгору",
        Arrange::Align(Axis::Vertical, Edge::Center) => "Вирівняти по центру вертикально",
        Arrange::Align(Axis::Vertical, Edge::Max) => "Вирівняти вниз",
        Arrange::Distribute(Axis::Vertical, Edge::Min) => "Розподілити за верхніми краями",
        Arrange::Distribute(Axis::Vertical, Edge::Center) => "Розподілити за центрами вертикально",
        Arrange::Distribute(Axis::Vertical, Edge::Max) => "Розподілити за нижніми краями",
        Arrange::Distribute(Axis::Horizontal, Edge::Min) => "Розподілити за лівими краями",
        Arrange::Distribute(Axis::Horizontal, Edge::Center) => {
            "Розподілити за центрами горизонтально"
        }
        Arrange::Distribute(Axis::Horizontal, Edge::Max) => "Розподілити за правими краями",
        Arrange::Space(Axis::Vertical) => "Рівні проміжки по вертикалі",
        Arrange::Space(Axis::Horizontal) => "Рівні проміжки по горизонталі",
    }
}

fn operation(op: BooleanOp) -> &'static str {
    match op {
        BooleanOp::Subtract => "Вирізати",
        BooleanOp::Union => "Об’єднати",
        BooleanOp::Intersect => "Перетнути",
    }
}

fn operation_hint(op: BooleanOp) -> &'static str {
    match op {
        BooleanOp::Subtract => "Вирізати: нові фігури вирізаються з вибраної деталі",
        BooleanOp::Union => "Об’єднати: нові фігури додаються до вибраної деталі",
        BooleanOp::Intersect => {
            "Перетнути: від вибраної деталі лишається тільки частина всередині нових фігур"
        }
    }
}

pub(super) fn unit_symbol(units: Units) -> &'static str {
    match units {
        Units::Millimeters => "мм",
        Units::Inches => "дюйм",
    }
}

fn decimal(value: f64) -> String {
    value.to_string().replace('.', ",")
}

fn plural(count: usize, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    match (count % 10, count % 100) {
        (_, 11..=14) => many,
        (1, _) => one,
        (2..=4, _) => few,
        _ => many,
    }
}

fn hole_kind(kind: HoleKind) -> &'static str {
    match kind {
        HoleKind::French => "Французький",
        HoleKind::Diamond => "Ромбоподібний",
        HoleKind::Round => "Круглий",
    }
}

fn range((min, max): (f64, Option<f64>)) -> String {
    match max {
        Some(max) => format!("{}–{}", decimal(min), decimal(max)),
        None => format!("{}+", decimal(min)),
    }
}
