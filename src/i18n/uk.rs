use super::{Label, Message};
use crate::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Origin, Strength, Texture, ThicknessCategory, Treatment, WaterResistance,
};
use crate::model::Units;

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
        Label::Circle => "Коло",
        Label::RectangleHint => "Перетягуйте від кута до кута. Esc скасовує.",
        Label::CircleHint => "Перетягуйте від центру назовні. Esc скасовує.",
        Label::HoleMode => "Отвір: Малювання, Прямокутник і Коло вирізають у вибраній деталі",
        Label::DrawHint => {
            "Клацайте, щоб додавати точки. Клацніть першу точку або натисніть Enter, щоб замкнути контур; Esc скасовує."
        }
        Label::SelectHint => {
            "Перетягуйте деталь або вершину вибраної деталі. Del видаляє вибрану деталь."
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
        Label::Undo => "Скасувати",
        Label::Redo => "Повторити",
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
        Message::PointCount { count } => {
            let noun = plural(*count, "точка", "точки", "точок");
            format!("{count} {noun}")
        }
        Message::DefaultPieceName { id } => format!("Деталь {id}"),
        Message::PieceNotFound { id } => format!("Деталь {id} не існує."),
        Message::PieceLocked { name } => format!("Деталь «{name}» заблоковано."),
        Message::HoleAdded { name } => format!("Додано отвір у деталь «{name}»."),
        Message::HoleDeleted => "Отвір видалено.".to_owned(),
        Message::PieceEdgeCut { name } => format!("Змінено край деталі «{name}»."),
        Message::HoleSplitsPiece { name } => {
            format!("Цей виріз розділив би деталь «{name}» на кілька частин.")
        }
        Message::HoleCoversPiece { name } => {
            format!("Цей виріз повністю прибрав би деталь «{name}».")
        }
        Message::SelectPieceForHole => "Виберіть деталь, у якій треба вирізати отвір.".to_owned(),
        Message::HoleNotFound { name } => format!("У деталі «{name}» немає такого отвору."),
        Message::TooManyHoles { name, max } => {
            format!("Деталь «{name}» може мати не більше {max} отворів.")
        }
        Message::HoleOutsidePiece { name } => {
            format!("Отвори деталі «{name}» мають лежати всередині контуру й не торкатися його.")
        }
        Message::HolesOverlap { name } => {
            format!("Отвори деталі «{name}» не повинні перетинатися чи торкатися один одного.")
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
        Message::ActionReshapePiece { name } => format!("Змінено форму «{name}»"),
        Message::ActionLockPiece { name } => format!("Заблоковано «{name}»"),
        Message::ActionUnlockPiece { name } => format!("Розблоковано «{name}»"),
        Message::ActionMaterial { name } => format!("Змінено матеріал «{name}»"),
        Message::ActionRemovePiece { name } => format!("Видалено «{name}»"),
        Message::ActionAddHole { name } => format!("Додано отвір у «{name}»"),
        Message::ActionCutEdge { name } => format!("Змінено край «{name}»"),
        Message::ActionRemoveHole { name } => format!("Видалено отвір у «{name}»"),
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
