//! The Layout panel: a tree of the project's pieces, from the "Layout Tree Pane" of the
//! Brilliant design. The root row counts the pieces; each piece is a card with its row (caret,
//! material swatch, name, material, lock and delete), and, while open, its base shape and its
//! child shapes in the order their operations apply.

mod rows;

use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{BooleanOp, ChildShape, PatternPiece, Project};
use leathercraft::shapes::Shape;
use rows::{ButtonLook, GAP, GLYPH_BOX, Icon, PIECE_BUTTON, ROW_HEIGHT, ROW_RADIUS, SHAPE_BUTTON};

use crate::ui_components::dropdown::{Dropdown, DropdownItem, ToneFn, dropdown_value};
use crate::ui_components::icons::{glyph, paint_glyph};
use crate::ui_components::palette::Palette;

/// Around the whole tree.
const CONTENT_PADDING: f32 = 10.0;
/// Between the root row and the cards, and between two cards.
const CARD_GAP: f32 = 4.0;
/// Inside an open card.
const CARD_PADDING: f32 = 4.0;
const CARD_RADIUS: u8 = 6;
/// Between the rows of an open card.
const CARD_ROW_GAP: f32 = 3.0;
const ROW_PADDING_X: f32 = 6.0;
/// Where the icon of a shape row starts, so that it sits under the swatch of its piece.
const SHAPE_INDENT: f32 = 20.0;
/// Between the buttons of a piece row.
const PIECE_BUTTON_GAP: f32 = 4.0;
/// Between the buttons of a shape row.
const SHAPE_BUTTON_GAP: f32 = 3.0;
const OP_WIDTH: f32 = 82.0;
const CARET_WIDTH: f32 = 11.0;
const FOLDER_WIDTH: f32 = 14.0;
const SWATCH: f32 = 7.0;
const NAME_SIZE: f32 = 12.0;
const MATERIAL_SIZE: f32 = 11.0;
const SHAPE_SIZE: f32 = 11.0;
const COUNT_SIZE: f32 = 10.0;
/// The material after a piece's name is left out when the name would get less room than this.
const MIN_NAME_WIDTH: f32 = 48.0;

fn shape_label(shape: &Shape) -> Label {
    match shape {
        Shape::Polygon(_) => Label::Polygon,
        Shape::Rectangle(_) => Label::Rectangle,
        Shape::Ellipse(_) => Label::Ellipse,
        Shape::RegularPolygon(_) => Label::RegularPolygon,
        Shape::Line(_) => Label::Centerline,
        Shape::StitchLine(_) => Label::StitchHoles,
    }
}

/// The glyph of a shape row: the tool that draws that kind of shape.
fn shape_glyph(shape: &Shape) -> char {
    match shape {
        Shape::Polygon(_) => glyph::DRAW,
        Shape::Rectangle(_) => glyph::RECTANGLE,
        Shape::Ellipse(_) => glyph::ELLIPSE,
        Shape::RegularPolygon(_) => glyph::REGULAR_POLYGON,
        Shape::Line(_) => glyph::CENTERLINE,
        Shape::StitchLine(_) => glyph::STITCH_HOLES,
    }
}

fn shape_name(shape: &Shape, language: Language) -> String {
    match shape {
        Shape::StitchLine(line) => language.message(&Message::StitchHolesRow {
            count: line.hole_count(),
        }),
        shape => language.label(shape_label(shape)).to_owned(),
    }
}

/// The tint of an operation: its picker, and the icon of the shapes it applies.
fn op_tone(op: BooleanOp) -> ToneFn {
    match op {
        BooleanOp::Union => |palette| &palette.layout.union,
        BooleanOp::Subtract => |palette| &palette.layout.subtract,
        BooleanOp::Intersect => |palette| &palette.layout.intersect,
    }
}

/// Whether a branch of the tree is open; every branch starts open.
fn is_open(ctx: &egui::Context, id: egui::Id) -> bool {
    ctx.data_mut(|data| *data.get_persisted_mut_or(id, true))
}

fn set_open(ctx: &egui::Context, id: egui::Id, open: bool) {
    ctx.data_mut(|data| data.insert_persisted(id, open));
}

/// The selected pieces, and the children selected in the primary piece.
pub struct Selection<'a> {
    pub primary: Option<u64>,
    pub pieces: &'a [u64],
    pub children: &'a [u64],
}

/// A clicked row. With Shift, a piece or child is added to or removed from the selection.
pub enum Pick {
    Piece { id: u64, toggle: bool },
    Child { id: u64, child: u64, toggle: bool },
}

/// What a row did: an edit to send, and a row to select.
type RowResult = (Option<ProjectEvent>, Option<Pick>);

fn shift_down(ui: &egui::Ui) -> bool {
    ui.input(|input| input.modifiers.shift)
}

/// Paints a glyph of the icon font with its box starting at `x`, centered on `y`.
fn paint_icon(painter: &egui::Painter, x: f32, y: f32, glyph: char, color: egui::Color32) {
    let rect = egui::Rect::from_min_size(
        egui::pos2(x, y - GLYPH_BOX / 2.0),
        egui::Vec2::splat(GLYPH_BOX),
    );
    paint_glyph(painter, rect, egui::Stroke::new(1.0, color), glyph);
}

/// Paints a galley with its left edge at `x`, centered on `y`.
fn paint_text(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    galley: std::sync::Arc<egui::Galley>,
    color: egui::Color32,
) {
    let pos = egui::pos2(x, y - galley.size().y / 2.0);
    painter.galley(pos, galley, color);
}

/// "Pattern pieces" with the number of pieces; a click opens or closes the whole tree.
/// Returns whether the tree is open.
fn root_row(ui: &mut egui::Ui, count: usize, language: Language) -> bool {
    let id = ui.make_persistent_id("layout_root");
    let mut open = is_open(ui.ctx(), id);
    let (rect, response) = rows::row(ui, ROW_HEIGHT);
    let title = language.label(Label::Pieces);
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, title));
    let painter = ui.painter().clone();
    let colors = &Palette::of(&painter).layout;
    let inner = rect.shrink2(egui::vec2(ROW_PADDING_X, 0.0));
    let y = rect.center().y;

    let count = rows::text(
        ui,
        &language.message(&Message::PieceCount { count }),
        egui::FontId::monospace(COUNT_SIZE),
        inner.width() / 2.0,
    );
    let count_left = inner.right() - count.size().x;
    let mut x = inner.left() + CARET_WIDTH + GAP;
    rows::paint_folder(
        &painter,
        egui::pos2(x + FOLDER_WIDTH / 2.0, y),
        colors.root_icon,
    );
    x += FOLDER_WIDTH + GAP;
    let title = rows::text(
        ui,
        title,
        egui::FontId::proportional(NAME_SIZE),
        count_left - GAP - x,
    );
    paint_text(&painter, x, y, title, colors.root_title);
    paint_text(&painter, count_left, y, count, colors.root_count);

    let caret = egui::pos2(inner.left() + CARET_WIDTH / 2.0, y);
    let toggled = rows::toggle(ui, id.with("toggle"), caret, open, colors.root_caret);
    if toggled || response.clicked() {
        open = !open;
        set_open(ui.ctx(), id, open);
    }
    open
}

/// A piece with its shapes, in a card that is highlighted while the piece is selected.
fn piece_card(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    selection: &Selection,
    language: Language,
) -> RowResult {
    let selected = selection.pieces.contains(&piece.id);
    let open_id = ui.make_persistent_id(("layout_piece", piece.id));
    let open = is_open(ui.ctx(), open_id);
    let colors = &Palette::of(ui.painter()).layout;
    let card = if selected {
        &colors.card_selected
    } else {
        &colors.card
    };
    let margin = if open { CARD_PADDING } else { 1.0 };
    egui::Frame::new()
        .fill(card.fill)
        .stroke(egui::Stroke::new(1.0, card.stroke))
        .corner_radius(CARD_RADIUS)
        .inner_margin(margin)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = CARD_ROW_GAP;
            let (mut event, mut pick, toggled) =
                piece_row(ui, piece, selected, open, open_id, language);
            if toggled {
                set_open(ui.ctx(), open_id, !open);
            }
            if !open {
                return (event, pick);
            }
            pick = base_row(ui, piece, language).or(pick);
            // Children are listed in the order their operations apply.
            for (index, child) in piece.children.iter().enumerate() {
                let (row_event, row_pick) = ui
                    .push_id(child.id, |ui| {
                        child_row(ui, piece, index, child, selection, language)
                    })
                    .inner;
                event = row_event.or(event);
                pick = row_pick.or(pick);
            }
            (event, pick)
        })
        .inner
}

/// The caret, the material swatch, the name and material, then lock and delete. Returns the
/// edit and pick of the row, and whether its caret was clicked.
fn piece_row(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    selected: bool,
    open: bool,
    open_id: egui::Id,
    language: Language,
) -> (Option<ProjectEvent>, Option<Pick>, bool) {
    let (rect, response) = rows::row(ui, ROW_HEIGHT);
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            &piece.name,
        )
    });
    let painter = ui.painter().clone();
    let palette = Palette::of(&painter);
    let colors = &palette.layout;
    let paint = if selected {
        &colors.piece_selected
    } else {
        &colors.piece
    };
    let fill = paint
        .fill
        .or(response.hovered().then_some(colors.piece_hover));
    if let Some(fill) = fill {
        painter.rect_filled(rect, ROW_RADIUS, fill);
    }

    let inner = rect.shrink2(egui::vec2(ROW_PADDING_X, 0.0));
    let look = if selected {
        ButtonLook::Framed
    } else {
        ButtonLook::Quiet
    };
    let mut event = None;
    let ((), buttons_left) = rows::trailing(ui, inner, PIECE_BUTTON_GAP, |ui| {
        ui.add_enabled_ui(!piece.locked, |ui| {
            let name = language.label(Label::DeletePiece);
            let icon = Icon::Glyph(glyph::TRASH);
            if rows::button(ui, PIECE_BUTTON, look, false, name, icon).clicked() {
                event = Some(ProjectEvent::RemovePiece { id: piece.id });
            }
        });
        let (name, icon) = if piece.locked {
            (Label::UnlockPiece, glyph::LOCK)
        } else {
            (Label::LockPiece, glyph::UNLOCK)
        };
        let name = language.label(name);
        let icon = Icon::Glyph(icon);
        if rows::button(ui, PIECE_BUTTON, look, piece.locked, name, icon).clicked() {
            event = Some(ProjectEvent::SetPieceLocked {
                id: piece.id,
                locked: !piece.locked,
            });
        }
    });

    let y = rect.center().y;
    let mut x = inner.left() + CARET_WIDTH + GAP;
    // The swatch shows the leather's own color, which is document data, not a theme color.
    let [red, green, blue] = piece.material.color.rgb();
    painter.circle(
        egui::pos2(x + SWATCH / 2.0, y),
        SWATCH / 2.0,
        egui::Color32::from_rgb(red, green, blue),
        egui::Stroke::new(1.0, palette.border.chip),
    );
    x += SWATCH + GAP;

    let room = buttons_left - GAP - x;
    let material = format!("({})", language.label(Label::Color(piece.material.color)));
    let material = rows::text(
        ui,
        &material,
        egui::FontId::proportional(MATERIAL_SIZE),
        f32::INFINITY,
    );
    let show_material = room - material.size().x - GAP >= MIN_NAME_WIDTH;
    let name_width = if show_material {
        room - material.size().x - GAP
    } else {
        room
    };
    let name = rows::text(
        ui,
        &piece.name,
        egui::FontId::proportional(NAME_SIZE),
        name_width,
    );
    let name_width = name.size().x;
    paint_text(&painter, x, y, name, paint.name);
    if show_material {
        paint_text(&painter, x + name_width + GAP, y, material, paint.material);
    }

    let caret = egui::pos2(inner.left() + CARET_WIDTH / 2.0, y);
    let toggled = rows::toggle(ui, open_id.with("toggle"), caret, open, paint.caret);
    let pick = response.clicked().then(|| Pick::Piece {
        id: piece.id,
        toggle: shift_down(ui),
    });
    (event, pick, toggled)
}

/// The icon and the numbered name of a shape row, cut to end before `right`.
#[allow(clippy::too_many_arguments)]
fn paint_shape_label(
    ui: &egui::Ui,
    rect: egui::Rect,
    right: f32,
    number: usize,
    shape: &Shape,
    icon_color: egui::Color32,
    text_color: egui::Color32,
    language: Language,
) {
    let painter = ui.painter();
    let y = rect.center().y;
    let x = rect.left() + SHAPE_INDENT - (GLYPH_BOX - SWATCH) / 2.0;
    paint_icon(painter, x, y, shape_glyph(shape), icon_color);
    let x = x + GLYPH_BOX + GAP / 2.0;
    let text = format!("{number}. {}", shape_name(shape, language));
    let galley = rows::text(
        ui,
        &text,
        egui::FontId::proportional(SHAPE_SIZE),
        right - GAP - x,
    );
    paint_text(painter, x, y, galley, text_color);
}

/// The base shape, numbered 1 and marked "Base". It has no operation and can't be removed on
/// its own, so a click selects the whole piece.
fn base_row(ui: &mut egui::Ui, piece: &PatternPiece, language: Language) -> Option<Pick> {
    let (rect, response) = rows::row(ui, ROW_HEIGHT);
    let colors = &Palette::of(ui.painter()).layout;
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, ROW_RADIUS, colors.shape_hover);
    }
    let inner = rect.shrink2(egui::vec2(ROW_PADDING_X, 0.0));
    let (_, badge_left) = rows::trailing(ui, inner, SHAPE_BUTTON_GAP, |ui| {
        rows::badge(ui, language.label(Label::Base))
    });
    paint_shape_label(
        ui,
        rect,
        badge_left,
        1,
        &piece.base,
        colors.shape_icon,
        colors.shape_text,
        language,
    );
    response.clicked().then(|| Pick::Piece {
        id: piece.id,
        toggle: shift_down(ui),
    })
}

/// A child shape: its icon in the color of its operation, its number and name, then the
/// operation picker and delete. A selected row also shows the buttons that reorder it.
fn child_row(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    index: usize,
    child: &ChildShape,
    selection: &Selection,
    language: Language,
) -> RowResult {
    let (id, child_id) = (piece.id, child.id);
    let selected = selection.primary == Some(id) && selection.children.contains(&child_id);
    let (rect, response) = rows::row(ui, ROW_HEIGHT);
    let palette = Palette::of(ui.painter());
    let colors = &palette.layout;
    if selected {
        ui.painter().rect(
            rect,
            ROW_RADIUS,
            colors.shape_selected.fill,
            egui::Stroke::new(1.0, colors.shape_selected.stroke),
            egui::StrokeKind::Inside,
        );
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, ROW_RADIUS, colors.shape_hover);
    }

    let inner = rect.shrink2(egui::vec2(ROW_PADDING_X, 0.0));
    let look = if selected {
        ButtonLook::Framed
    } else {
        ButtonLook::Quiet
    };
    let mut event = None;
    let ((), buttons_left) = rows::trailing(ui, inner, SHAPE_BUTTON_GAP, |ui| {
        ui.add_enabled_ui(!piece.locked, |ui| {
            let name = language.label(Label::DeleteChild);
            let icon = Icon::Glyph(glyph::TRASH);
            if rows::button(ui, SHAPE_BUTTON, look, false, name, icon).clicked() {
                event = Some(ProjectEvent::RemoveChild {
                    id,
                    child: child_id,
                });
            }
            if selected {
                let last = index + 1 == piece.children.len();
                ui.add_enabled_ui(!last, |ui| {
                    let name = language.label(Label::MoveDown);
                    let icon = Icon::Chevron { up: false };
                    if rows::button(ui, SHAPE_BUTTON, look, false, name, icon).clicked() {
                        event = Some(ProjectEvent::ReorderChild {
                            id,
                            child: child_id,
                            index: index + 1,
                        });
                    }
                });
                ui.add_enabled_ui(index > 0, |ui| {
                    let name = language.label(Label::MoveUp);
                    let icon = Icon::Chevron { up: true };
                    if rows::button(ui, SHAPE_BUTTON, look, false, name, icon).clicked() {
                        event = Some(ProjectEvent::ReorderChild {
                            id,
                            child: child_id,
                            index: index - 1,
                        });
                    }
                });
            }
            // A line's operation is ignored, so it has no picker.
            if child.shape.is_open() {
                return;
            }
            let mut op = child.op;
            Dropdown::new(
                ("child_op", id, child_id),
                language.label(Label::Operation(op)),
            )
            .width(OP_WIDTH)
            .compact(true)
            .tone(op_tone(op))
            .show_ui(ui, |ui| {
                for option in BooleanOp::ALL {
                    let item = DropdownItem::new(language.label(Label::Operation(option)));
                    dropdown_value(ui, &mut op, option, item);
                }
            });
            if op != child.op {
                event = Some(ProjectEvent::SetChildOp {
                    id,
                    child: child_id,
                    op,
                });
            }
        });
    });

    let icon_color = if child.shape.is_open() {
        colors.shape_icon
    } else {
        op_tone(child.op)(palette).text
    };
    let text_color = if selected {
        colors.shape_text_selected
    } else {
        colors.shape_text
    };
    // The base is number 1.
    let number = index + 2;
    paint_shape_label(
        ui,
        rect,
        buttons_left,
        number,
        &child.shape,
        icon_color,
        text_color,
        language,
    );
    let pick = response.clicked().then(|| Pick::Child {
        id,
        child: child_id,
        toggle: shift_down(ui),
    });
    (event, pick)
}

pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    selection: Selection,
    language: Language,
) -> RowResult {
    let mut event = None;
    let mut pick = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(CONTENT_PADDING)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = CARD_GAP;
                    if !root_row(ui, project.pieces.len(), language) {
                        return;
                    }
                    for piece in &project.pieces {
                        let (row_event, row_pick) = ui
                            .push_id(piece.id, |ui| piece_card(ui, piece, &selection, language))
                            .inner;
                        event = row_event.or(event.take());
                        pick = row_pick.or(pick.take());
                    }
                });
        });
    (event, pick)
}
