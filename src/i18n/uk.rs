use super::{Label, Message};
use crate::model::Units;

pub(super) fn label(label: Label) -> &'static str {
    match label {
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
        Label::Tool => "Інструмент",
        Label::Select => "Вибір",
        Label::Draw => "Малювання",
        Label::DrawHint => {
            "Клацайте, щоб додавати точки. Клацніть першу точку або натисніть Enter, щоб замкнути контур; Esc скасовує."
        },
        Label::SelectHint => {
            "Перетягуйте деталь або вершину вибраної деталі. Del видаляє вибрану деталь."
        },
        Label::NavigationHint => {
            "Перетягування правою або середньою кнопкою зсуває вигляд; коліщатко змінює масштаб."
        },
        Label::SnapMillimeters => "Прив’язка (мм)",
        Label::ZoomToFit => "Показати все",
        Label::DisplayUnits => "Одиниці виміру",
        Label::Pieces => "Деталі",
        Label::DeletePiece => "Видалити деталь",
        Label::Camera => "Камера",
        Label::Orbit => "Оберт",
        Label::Tilt => "Нахил",
        Label::Zoom => "Масштаб",
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
        },
        Message::DefaultPieceName { id } => format!("Деталь {id}"),
        Message::PieceNotFound { id } => format!("Деталь {id} не існує."),
        Message::UnsupportedVersion { found, expected } => {
            format!("Версія проєкту {found} не підтримується (очікується {expected}).")
        },
        Message::TooManyPieces { max } => {
            format!("Проєкт може містити не більше ніж {max} деталей.")
        },
        Message::InvalidPieceId { name } => {
            format!("Деталь «{name}» має недійсний або повторюваний ідентифікатор.")
        },
        Message::PieceNameRequired { id } => format!("Деталь {id} повинна мати назву."),
        Message::PointCountOutOfRange { name, max } => {
            format!("Деталь «{name}» повинна мати від 3 до {max} точок.")
        },
        Message::CoordinateOutOfRange { name } => {
            format!("Деталь «{name}» має точку поза підтримуваним діапазоном координат.")
        },
        Message::CoincidentPoints { name } => {
            format!("Деталь «{name}» має дві послідовні точки в одному місці.")
        },
        Message::ZeroArea { name } => {
            format!("Деталь «{name}» повинна охоплювати ненульову площу.")
        },
        Message::CouldNotEncode { detail } => format!("Не вдалося закодувати проєкт: {detail}"),
        Message::CouldNotRead { path, detail } => {
            format!("Не вдалося прочитати «{path}»: {detail}")
        },
        Message::CouldNotParse { path, detail } => {
            format!("Не вдалося розібрати «{path}»: {detail}")
        }
        Message::CouldNotWrite { path, detail } => {
            format!("Не вдалося записати «{path}»: {detail}")
        },
        Message::ExportEmpty => "Додайте принаймні одну деталь перед експортом.".to_owned(),
    }
}

pub(super) fn unit_symbol(units: Units) -> &'static str {
    match units {
        Units::Millimeters => "мм",
        Units::Inches => "дюйм",
    }
}

fn plural(count: usize, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    match (count % 10, count % 100) {
        (_, 11..=14) => many,
        (1, _) => one,
        (2..=4, _) => few,
        _ => many,
    }
}
