//! The Link seams dialog, from the "Stitch Link Dialog" of the Brilliant design: two seam cards
//! with the start offset between them, the schematic of the links beside the table of pairs and
//! a summary, a warning when the hole counts differ, and the footer.
//!
//! The dialog edits a copy of the link (`SeamLinkDraft`) and computes its pairs from the project
//! every frame, so that a pitch fitted from the warning shows at once.

mod schematic;
mod table;

use crate::editor::format_length;
use crate::ui_components::button::Button;
use crate::ui_components::dropdown::{Dropdown, DropdownItem};
use crate::ui_components::icons::{
    paint_close, paint_link, paint_reset, paint_reverse, paint_warning,
};
use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{Project, Units};
use leathercraft::seams::{
    LinkPairs, ManualPair, SeamLink, SeamSide, stitch_line, stitch_lines, stitch_number,
};
use leathercraft::shapes::{Shape, StitchLine};
use schematic::{Side, schematic};

/// The design's size, and the smallest size the cards and the body still fit in.
const DEFAULT_SIZE: egui::Vec2 = egui::vec2(1000.0, 700.0);
const MIN_SIZE: egui::Vec2 = egui::vec2(860.0, 600.0);
/// Kept free around the dialog when the window is too small for it.
const SCREEN_MARGIN: f32 = 20.0;
/// The width of the strips along the edges that resize the dialog.
const RESIZE_GRIP: f32 = 6.0;
/// The warning and the footer, before they are first measured.
const DEFAULT_BOTTOM_HEIGHT: f32 = 116.0;
const MIN_BODY_HEIGHT: f32 = 240.0;
const DIALOG_RADIUS: u8 = 10;
const EDGE: f32 = 18.0;
const CARD_GAP: f32 = 12.0;
const OFFSET_WIDTH: f32 = 110.0;
const CARD_PADDING: egui::Margin = egui::Margin::symmetric(14, 12);
const CARD_RADIUS: u8 = 8;
const CARD_ROW_GAP: f32 = 10.0;
/// The schematic's part of the body's width: 560 of 948 in the design.
const SCHEMATIC_SHARE: f32 = 0.59;
const BODY_GAP: f32 = 16.0;
const BODY_PADDING: f32 = 14.0;
const COLUMN_GAP: f32 = 10.0;
const TITLE_ROW_HEIGHT: f32 = 16.0;
/// The summary tiles under the table.
const SUMMARY_HEIGHT: f32 = 50.0;
const RANGE_FIELD_WIDTH: f32 = 60.0;
const OFFSET_FIELD_WIDTH: f32 = 84.0;
const CHIP_PADDING: egui::Margin = egui::Margin::symmetric(7, 3);
const CHIP_RADIUS: u8 = 4;
const SMALL_TEXT: f32 = 10.0;
const LABEL_TEXT: f32 = 11.0;
const TEXT: f32 = 12.0;
const TITLE_TEXT: f32 = 15.0;
const SMALL_BUTTON_HEIGHT: f32 = 26.0;
const ICON_SIZE: f32 = 13.0;

/// The link being made or changed while the dialog is open.
#[derive(Clone, Debug)]
pub struct SeamLinkDraft {
    /// The link's id when it was opened to change an existing link.
    pub editing: Option<u64>,
    pub link: SeamLink,
    /// The hole of A clicked first, waiting for a hole of B to make a manual pair.
    pub pending_a: Option<usize>,
}

/// What opens the dialog: a new link, starting from a seam if one is given, or an existing link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenLink {
    New(Option<(u64, u64)>),
    Edit(u64),
}

impl SeamLinkDraft {
    /// A draft for `request`, or `None` when the project has no stitch line or no such link.
    pub fn open(project: &Project, request: OpenLink) -> Option<Self> {
        match request {
            OpenLink::Edit(id) => project.seam_link(id).map(|link| Self {
                editing: Some(id),
                link: link.clone(),
                pending_a: None,
            }),
            OpenLink::New(seam) => {
                // The given seam is both A and B at first; the user picks B.
                let (piece, child) = seam
                    .filter(|&(piece, child)| stitch_line(project, piece, child).is_some())
                    .or_else(|| stitch_lines(project).first().copied())?;
                let side = SeamSide::whole(piece, child, stitch_line(project, piece, child)?);
                Some(Self {
                    editing: None,
                    link: SeamLink {
                        id: 0,
                        a: side,
                        b: side,
                        offset: 0,
                        manual: Vec::new(),
                    },
                    pending_a: None,
                })
            }
        }
    }
}

pub enum SeamLinkResult {
    /// Adds or changes the link; the dialog closes.
    Commit(ProjectEvent),
    /// Changes a seam's pitch; the dialog stays open.
    FitPitch(ProjectEvent),
    Cancel,
}

/// A seam's name, as in its list: the piece and the seam's number among its stitch lines.
pub fn seam_name(project: &Project, piece: u64, child: u64, language: Language) -> String {
    let name = project
        .pieces
        .iter()
        .find(|candidate| candidate.id == piece)
        .map_or_else(|| piece.to_string(), |piece| piece.name.clone());
    let index = stitch_number(project, piece, child).unwrap_or_default();
    language.message(&Message::SeamName { piece: name, index })
}

/// Clicking outside the dialog does nothing, so a stray click cannot lose the pairs set by hand.
/// Its edges and corners resize it; the size is kept for the session.
pub fn seam_link_dialog(
    ctx: &egui::Context,
    draft: &mut SeamLinkDraft,
    project: &Project,
    language: Language,
) -> Option<SeamLinkResult> {
    let palette = Palette::of_visuals(&ctx.style().visuals);
    let frame = egui::Frame::new()
        .fill(palette.surface.panel)
        .stroke(egui::Stroke::new(1.0, palette.border.field))
        .corner_radius(DIALOG_RADIUS);
    let id = egui::Id::new("seam_link_dialog");
    let size = dialog_size(ctx, id);
    let modal = egui::Modal::new(id).frame(frame).show(ctx, |ui| {
        ui.set_width(size.x);
        ui.set_height(size.y);
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
        let rect = egui::Rect::from_min_size(ui.max_rect().min, size);
        let result = ui
            .scope(|ui| dialog_contents(ui, draft, project, language))
            .inner;
        resize_edges(ui, id, rect);
        result
    });
    // Esc first drops a hole waiting for its pair, then closes an open dropdown, then the dialog.
    if draft.pending_a.is_some()
        && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        draft.pending_a = None;
    }
    let escaped = !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    modal.inner.or(escaped.then_some(SeamLinkResult::Cancel))
}

/// The dialog's size: the last one the user dragged to, within the screen.
fn dialog_size(ctx: &egui::Context, id: egui::Id) -> egui::Vec2 {
    let screen = ctx.content_rect().size() - egui::Vec2::splat(2.0 * SCREEN_MARGIN);
    let max = screen.max(MIN_SIZE);
    ctx.data(|data| data.get_temp::<egui::Vec2>(id.with("size")))
        .unwrap_or(DEFAULT_SIZE)
        .clamp(MIN_SIZE, max)
}

/// Drag strips along the dialog's edges and corners. The dialog stays centered, so an edge
/// grows it on both sides and follows the pointer.
fn resize_edges(ui: &mut egui::Ui, id: egui::Id, rect: egui::Rect) {
    use egui::CursorIcon;
    let grip = RESIZE_GRIP;
    let (left, right) = (rect.left(), rect.right());
    let (top, bottom) = (rect.top(), rect.bottom());
    // (strip, resizes width, resizes height, cursor)
    let strips = [
        (
            egui::Rect::from_min_max(
                egui::pos2(right - grip, top + grip),
                egui::pos2(right, bottom - grip),
            ),
            true,
            false,
            CursorIcon::ResizeHorizontal,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(left, top + grip),
                egui::pos2(left + grip, bottom - grip),
            ),
            true,
            false,
            CursorIcon::ResizeHorizontal,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(left + grip, bottom - grip),
                egui::pos2(right - grip, bottom),
            ),
            false,
            true,
            CursorIcon::ResizeVertical,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(left + grip, top),
                egui::pos2(right - grip, top + grip),
            ),
            false,
            true,
            CursorIcon::ResizeVertical,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(right - 2.0 * grip, bottom - 2.0 * grip),
                egui::pos2(right, bottom),
            ),
            true,
            true,
            CursorIcon::ResizeNwSe,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(left, top),
                egui::pos2(left + 2.0 * grip, top + 2.0 * grip),
            ),
            true,
            true,
            CursorIcon::ResizeNwSe,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(right - 2.0 * grip, top),
                egui::pos2(right, top + 2.0 * grip),
            ),
            true,
            true,
            CursorIcon::ResizeNeSw,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(left, bottom - 2.0 * grip),
                egui::pos2(left + 2.0 * grip, bottom),
            ),
            true,
            true,
            CursorIcon::ResizeNeSw,
        ),
    ];
    // The modal is centered on the screen; its own rect lags a frame behind while it grows.
    let center = ui.ctx().content_rect().center();
    let mut size = rect.size();
    for (index, (strip, width, height, cursor)) in strips.into_iter().enumerate() {
        let response = ui.interact(strip, id.with(("resize", index)), egui::Sense::drag());
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if response.dragged()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let reach = (pointer - center).abs() * 2.0;
            if width {
                size.x = reach.x;
            }
            if height {
                size.y = reach.y;
            }
        }
    }
    // The grip in the bottom right corner, as on a window.
    let color = Palette::of(ui.painter()).text.muted;
    let corner = rect.right_bottom() - egui::vec2(5.0, 5.0);
    for step in 1..=3 {
        let offset = 3.5 * step as f32;
        ui.painter().line_segment(
            [
                corner - egui::vec2(offset, 0.0),
                corner - egui::vec2(0.0, offset),
            ],
            egui::Stroke::new(1.0, color),
        );
    }
    if size != rect.size() {
        let size = size.max(MIN_SIZE).round();
        ui.ctx()
            .data_mut(|data| data.insert_temp(id.with("size"), size));
        ui.ctx().request_repaint();
    }
}

fn dialog_contents(
    ui: &mut egui::Ui,
    draft: &mut SeamLinkDraft,
    project: &Project,
    language: Language,
) -> Option<SeamLinkResult> {
    let mut result = None;
    if header(ui, language) {
        result = Some(SeamLinkResult::Cancel);
    }
    seam_cards(ui, draft, project, language);
    horizontal_rule(ui);

    // The body takes the height the warning and the footer leave, measured on the last frame.
    let bottom_id = egui::Id::new("seam_link_dialog_bottom");
    let bottom = ui
        .ctx()
        .data(|data| data.get_temp::<f32>(bottom_id))
        .unwrap_or(DEFAULT_BOTTOM_HEIGHT);
    let body_height = (ui.available_height() - bottom).max(MIN_BODY_HEIGHT);
    let width = ui.available_width();

    let pairs = draft.link.pairs(project);
    let range_error = range_error(&draft.link);
    ui.allocate_ui_with_layout(
        egui::vec2(width, body_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_height(body_height);
            match &pairs {
                Ok(pairs) => body(ui, draft, pairs, project.units, language),
                Err(error) => error_line(ui, &language.message(error)),
            }
        },
    );

    let bottom_top = ui.cursor().top();
    if let Some(error) = &range_error {
        error_line(ui, &language.message(error));
    }
    if let Ok(pairs) = &pairs
        && pairs.count_mismatch()
        && let Some(fit) = warning(ui, &draft.link, pairs, project, language)
    {
        match fit.side {
            Side::A => draft.link.a.to = fit.to,
            Side::B => draft.link.b.to = fit.to,
        }
        result = Some(SeamLinkResult::FitPitch(fit.event));
    }
    ui.add_space(14.0);

    let can_commit =
        range_error.is_none() && pairs.as_ref().is_ok_and(|pairs| !pairs.pairs.is_empty());
    let count = pairs.as_ref().map_or(0, |pairs| pairs.pairs.len());
    match footer(ui, draft, can_commit, count, language) {
        Some(FooterAction::Reset) => {
            draft.link.manual.clear();
            draft.pending_a = None;
        }
        Some(FooterAction::Cancel) => result = Some(SeamLinkResult::Cancel),
        Some(FooterAction::Commit) => {
            let link = draft.link.clone();
            let event = match draft.editing {
                Some(id) => ProjectEvent::SetSeamLink {
                    link: SeamLink { id, ..link },
                },
                None => ProjectEvent::AddSeamLink { link },
            };
            result = Some(SeamLinkResult::Commit(event));
        }
        None => {}
    }
    let measured = ui.cursor().top() - bottom_top;
    if (measured - bottom).abs() > 0.5 {
        ui.ctx()
            .data_mut(|data| data.insert_temp(bottom_id, measured));
        ui.ctx().request_repaint();
    }
    result
}

/// What keeps the link from being saved, checked before `Project::apply` would refuse it.
fn range_error(link: &SeamLink) -> Option<Message> {
    if link.a.from > link.a.to || link.b.from > link.b.to {
        Some(Message::InvalidSeamRange)
    } else if link.ranges_overlap() {
        Some(Message::SeamRangesOverlap)
    } else {
        None
    }
}

/// The icon, title and subtitle, and the close button; returns whether it was clicked.
fn header(ui: &mut egui::Ui, language: Language) -> bool {
    let palette = Palette::of(ui.painter());
    let mut closed = false;
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: EDGE as i8,
            right: 16,
            top: 14,
            bottom: 12,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                paint_link(
                    ui.painter(),
                    rect,
                    egui::Stroke::new(1.5, palette.accent.base),
                );
                ui.label(
                    egui::RichText::new(language.label(Label::LinkSeamsTitle))
                        .size(TITLE_TEXT)
                        .strong()
                        .color(palette.text.primary),
                );
                ui.label(
                    egui::RichText::new(language.label(Label::LinkSeamsSubtitle))
                        .size(TEXT)
                        .color(palette.text.muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(egui::Vec2::splat(26.0), egui::Sense::click());
                    let response = response.on_hover_text(language.label(Label::CloseWindow));
                    if response.hovered() {
                        ui.painter().rect_filled(rect, 5, palette.surface.chip);
                    }
                    paint_close(
                        ui.painter(),
                        rect,
                        egui::Stroke::new(1.5, palette.text.secondary),
                    );
                    closed = response.clicked();
                });
            });
        });
    closed
}

fn horizontal_rule(ui: &mut egui::Ui) {
    let palette = Palette::of(ui.painter());
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0, palette.border.divider);
}

fn error_line(ui: &mut egui::Ui, text: &str) {
    let palette = Palette::of(ui.painter());
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(EDGE as i8, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(TEXT)
                    .color(palette.seam_link.link_free),
            );
        });
}

/// The two seam cards with the start offset between them.
fn seam_cards(ui: &mut egui::Ui, draft: &mut SeamLinkDraft, project: &Project, language: Language) {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: EDGE as i8,
            right: EDGE as i8,
            top: 4,
            bottom: 14,
        })
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                let card_width = (ui.available_width() - OFFSET_WIDTH - 2.0 * CARD_GAP) / 2.0;
                let changed_a = seam_card(ui, card_width, Side::A, draft, project, language);
                offset_column(ui, draft, project, language);
                let changed_b = seam_card(ui, card_width, Side::B, draft, project, language);
                // Hole numbers of the manual pairs belong to the seams they were set on.
                if changed_a || changed_b {
                    draft.link.manual.clear();
                    draft.pending_a = None;
                }
            });
        });
}

/// A seam's card: its title and hole count, the seam list, its hole type, pitch and length, its
/// range and the Reverse toggle. Returns whether another seam was picked.
fn seam_card(
    ui: &mut egui::Ui,
    width: f32,
    which: Side,
    draft: &mut SeamLinkDraft,
    project: &Project,
    language: Language,
) -> bool {
    let palette = Palette::of(ui.painter());
    let units = project.units;
    let mut picked = None;
    let side = match which {
        Side::A => draft.link.a,
        Side::B => draft.link.b,
    };
    let line = stitch_line(project, side.piece, side.child);
    let count = line.map_or(0, StitchLine::hole_count);
    let mut edited = side;
    let inner_width = width - (CARD_PADDING.left + CARD_PADDING.right) as f32;
    egui::Frame::new()
        .fill(palette.surface.ruler)
        .stroke(egui::Stroke::new(1.0, palette.border.control))
        .corner_radius(CARD_RADIUS)
        .inner_margin(CARD_PADDING)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner_width);
                ui.spacing_mut().item_spacing = egui::vec2(6.0, CARD_ROW_GAP);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let (letter, title) = match which {
                        Side::A => ("A", Label::SeamA),
                        Side::B => ("B", Label::SeamB),
                    };
                    side_badge(ui, letter);
                    ui.label(
                        egui::RichText::new(language.label(title))
                            .size(TEXT)
                            .strong()
                            .color(palette.text.primary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        chip(ui, &language.message(&Message::HoleCountBadge { count }));
                    });
                });

                let selected = seam_name(project, side.piece, side.child, language);
                Dropdown::new(("seam_link_side", which), selected)
                    .width(inner_width)
                    .show_ui(ui, |ui| {
                        for (piece, child) in stitch_lines(project) {
                            let Some(line) = stitch_line(project, piece, child) else {
                                continue;
                            };
                            let name = project
                                .pieces
                                .iter()
                                .find(|candidate| candidate.id == piece)
                                .map(|piece| piece.name.clone())
                                .unwrap_or_default();
                            let text = language.message(&Message::SeamOption {
                                piece: name,
                                index: stitch_number(project, piece, child).unwrap_or_default(),
                                holes: line.hole_count(),
                            });
                            let chosen = side.piece == piece && side.child == child;
                            if ui.add(DropdownItem::new(text).selected(chosen)).clicked() && !chosen
                            {
                                picked = Some(SeamSide::whole(piece, child, line));
                            }
                        }
                    });

                if let Some(line) = line {
                    ui.horizontal(|ui| {
                        let size = format_length(language, units, line.size);
                        chip(
                            ui,
                            &format!("{} · {size}", language.label(Label::HoleKind(line.hole))),
                        );
                        chip(
                            ui,
                            &format!(
                                "{} {}",
                                language.label(Label::Pitch),
                                format_length(language, units, line.pitch)
                            ),
                        );
                        chip(
                            ui,
                            &format!(
                                "{} {}",
                                language.label(Label::StitchLength),
                                format_length(language, units, line.length())
                            ),
                        );
                    });
                }

                // As tall as the fields, so that the captions are centered on them.
                let row = egui::vec2(ui.available_width(), SMALL_BUTTON_HEIGHT);
                let centered = egui::Layout::left_to_right(egui::Align::Center);
                ui.allocate_ui_with_layout(row, centered, |ui| {
                    let last = count.max(1);
                    caption(ui, language.label(Label::HolesFrom));
                    let mut from = edited.from + 1;
                    ui.add(
                        NumberInput::new(&mut from)
                            .range(1..=last)
                            .width(RANGE_FIELD_WIDTH)
                            .id_salt(("seam_link_from", which)),
                    );
                    edited.from = from - 1;
                    ui.add_space(4.0);
                    caption(ui, language.label(Label::HolesTo));
                    let mut to = edited.to + 1;
                    // A range that runs past a shortened line keeps its end until it is edited.
                    let to_max = last.max(to);
                    ui.add(
                        NumberInput::new(&mut to)
                            .range(1..=to_max)
                            .width(RANGE_FIELD_WIDTH)
                            .id_salt(("seam_link_to", which)),
                    );
                    edited.to = to - 1;
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if toggle_button(ui, edited.reversed, language.label(Label::Reverse))
                            .clicked()
                        {
                            edited.reversed = !edited.reversed;
                        }
                    });
                });
            })
        });

    let side = match which {
        Side::A => &mut draft.link.a,
        Side::B => &mut draft.link.b,
    };
    match picked {
        Some(picked) => {
            *side = picked;
            true
        }
        None => {
            *side = edited;
            false
        }
    }
}

/// The start offset between the cards: B's sequence shifted against A's, in holes.
fn offset_column(
    ui: &mut egui::Ui,
    draft: &mut SeamLinkDraft,
    project: &Project,
    language: Language,
) {
    let palette = Palette::of(ui.painter());
    let longest = [&draft.link.a, &draft.link.b]
        .iter()
        .filter_map(|side| stitch_line(project, side.piece, side.child))
        .map(StitchLine::hole_count)
        .max()
        .unwrap_or(0) as i64;
    ui.allocate_ui_with_layout(
        egui::vec2(OFFSET_WIDTH, 146.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(OFFSET_WIDTH);
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.add_space(36.0);
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(18.0), egui::Sense::hover());
            paint_link(
                ui.painter(),
                rect,
                egui::Stroke::new(1.5, palette.seam_link.arrow),
            );
            caption(ui, language.label(Label::StartOffset));
            let before = draft.link.offset;
            ui.add(
                NumberInput::new(&mut draft.link.offset)
                    .range(-longest..=longest)
                    .unit(language.label(Label::HolesShort))
                    .width(OFFSET_FIELD_WIDTH)
                    .id_salt("seam_link_offset"),
            );
            if draft.link.offset != before {
                draft.pending_a = None;
            }
        },
    );
}

/// The schematic with its legend and hint, beside the table of pairs and the summary.
fn body(
    ui: &mut egui::Ui,
    draft: &mut SeamLinkDraft,
    pairs: &LinkPairs,
    units: Units,
    language: Language,
) {
    let palette = Palette::of(ui.painter());
    let inner = ui.available_size() - egui::vec2(2.0 * EDGE, 2.0 * BODY_PADDING);
    // The schematic takes the share of the width it has in the design; the pairs the rest.
    let schematic_width = ((inner.x - BODY_GAP) * SCHEMATIC_SHARE).round();
    let pairs_width = inner.x - BODY_GAP - schematic_width;
    let column = |ui: &mut egui::Ui, width: f32, add: &mut dyn FnMut(&mut egui::Ui)| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, inner.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(width);
                ui.set_height(inner.y);
                ui.spacing_mut().item_spacing.y = COLUMN_GAP;
                add(ui);
            },
        );
    };
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(EDGE as i8, BODY_PADDING as i8))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = BODY_GAP;
                column(ui, schematic_width, &mut |ui| {
                    title_row(ui, |ui| {
                        section_title(ui, language.label(Label::LinkSchematic));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            legend(ui, language)
                        });
                    });
                    let height = (inner.y - 2.0 * TITLE_ROW_HEIGHT - 2.0 * COLUMN_GAP).max(120.0);
                    let size = egui::vec2(schematic_width, height);
                    if let Some(click) =
                        schematic(ui, size, pairs, draft.link.offset, draft.pending_a)
                    {
                        apply_click(draft, click);
                    }
                    title_row(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(language.label(Label::LinkHint))
                                    .size(LABEL_TEXT)
                                    .color(palette.text.muted),
                            )
                            .truncate(),
                        );
                    });
                });
                column(ui, pairs_width, &mut |ui| {
                    title_row(ui, |ui| {
                        section_title(ui, language.label(Label::HolePairs));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(language.message(&Message::PairsCount {
                                    count: pairs.pairs.len(),
                                }))
                                .size(LABEL_TEXT)
                                .color(palette.text.muted),
                            );
                        });
                    });
                    let height =
                        (inner.y - TITLE_ROW_HEIGHT - SUMMARY_HEIGHT - 2.0 * COLUMN_GAP).max(80.0);
                    table::pair_table(ui, pairs, height, units, language);
                    summary(ui, pairs, units, language);
                });
            });
        });
}

/// A row of a fixed height, so that the columns' heights add up.
fn title_row(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), TITLE_ROW_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_height(TITLE_ROW_HEIGHT);
            add(ui);
        },
    );
}

/// A click on a hole of the schematic: A waits for B, B makes the manual pair, Alt removes one.
fn apply_click(draft: &mut SeamLinkDraft, click: schematic::Click) {
    let manual = &mut draft.link.manual;
    match (click.side, click.alt) {
        (Side::A, true) => manual.retain(|pair| pair.a != click.hole),
        (Side::B, true) => manual.retain(|pair| pair.b != click.hole),
        (Side::A, false) => {
            draft.pending_a = (draft.pending_a != Some(click.hole)).then_some(click.hole);
        }
        (Side::B, false) => {
            if let Some(a) = draft.pending_a.take() {
                manual.retain(|pair| pair.a != a && pair.b != click.hole);
                manual.push(ManualPair { a, b: click.hole });
            }
        }
    }
}

/// Three tiles: the pairs, the holes without a pair and the difference of the lengths.
fn summary(ui: &mut egui::Ui, pairs: &LinkPairs, units: Units, language: Language) {
    let palette = Palette::of(ui.painter());
    let (free_a, free_b) = (pairs.free_a.len(), pairs.free_b.len());
    let free_side = match (free_a, free_b) {
        (0, 0) | (1.., 1..) => None,
        (_, 0) => Some('A'),
        (0, _) => Some('B'),
    };
    let difference = (pairs.length_b - pairs.length_a).abs();
    let creeping = difference >= 0.05;
    let tiles = [
        (
            language.label(Label::Pairs).to_owned(),
            pairs.pairs.len().to_string(),
            palette.text.primary,
        ),
        (
            language.message(&Message::FreeIn { side: free_side }),
            (free_a + free_b).to_string(),
            if free_a + free_b > 0 {
                palette.seam_link.link_free
            } else {
                palette.text.primary
            },
        ),
        (
            language.label(Label::LengthDifference).to_owned(),
            format_length(language, units, difference),
            if creeping {
                palette.seam_link.link_manual
            } else {
                palette.text.primary
            },
        ),
    ];
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let width = (ui.available_width() - 16.0) / 3.0;
        for (title, value, color) in tiles {
            egui::Frame::new()
                .fill(palette.surface.ruler)
                .stroke(egui::Stroke::new(1.0, palette.border.divider))
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // The margins and the 1 px border on each side.
                        ui.set_width(width - 22.0);
                        ui.set_height(SUMMARY_HEIGHT - 18.0);
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(title)
                                    .size(SMALL_TEXT)
                                    .color(palette.text.muted),
                            )
                            .truncate(),
                        );
                        ui.label(
                            egui::RichText::new(value)
                                .monospace()
                                .size(14.0)
                                .strong()
                                .color(color),
                        );
                    })
                });
        }
    });
}

/// The warning shown while the ranges hold different numbers of holes, with a button per side
/// that fits its pitch; returns the fit of a clicked button.
fn warning(
    ui: &mut egui::Ui,
    link: &SeamLink,
    pairs: &LinkPairs,
    project: &Project,
    language: Language,
) -> Option<Fit> {
    let palette = Palette::of(ui.painter());
    let paint = &palette.seam_link;
    let (count_a, count_b) = (pairs.sequence_a.len(), pairs.sequence_b.len());
    let free = pairs.free_a.len() + pairs.free_b.len();
    let side = match (pairs.free_a.is_empty(), pairs.free_b.is_empty()) {
        (true, false) => Some('B'),
        (false, true) => Some('A'),
        _ => None,
    };
    let fits = [
        (Side::A, &link.a, count_b, count_a),
        (Side::B, &link.b, count_a, count_b),
    ]
    .map(|(which, side, target, current)| fit_pitch(project, which, side, target, current));
    let mut clicked = None;
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(EDGE as i8, 0))
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(paint.warning_bg)
                .stroke(egui::Stroke::new(1.0, paint.warning_border))
                .corner_radius(6)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        let (rect, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                        paint_warning(
                            ui.painter(),
                            rect,
                            egui::Stroke::new(1.4, paint.link_manual),
                        );
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(
                                egui::RichText::new(language.message(
                                    &Message::HoleCountMismatch {
                                        a: count_a,
                                        b: count_b,
                                    },
                                ))
                                .size(TEXT)
                                .strong()
                                .color(paint.warning_text),
                            );
                            ui.label(
                                egui::RichText::new(
                                    language.message(&Message::UnpairedHoles { side, count: free }),
                                )
                                .size(LABEL_TEXT)
                                .color(paint.warning_detail),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            // Right to left, so B comes last in the list to end up on the right.
                            for fit in fits.into_iter().rev().flatten() {
                                let text = language.message(&Message::FitPitch {
                                    side: fit.side.letter(),
                                    pitch: format_length(language, project.units, fit.pitch),
                                });
                                if warning_button(ui, &text).clicked() {
                                    clicked = Some(fit);
                                }
                            }
                        });
                    });
                });
        });
    clicked
}

/// A pitch for one side, offered by the warning.
struct Fit {
    side: Side,
    pitch: f64,
    /// The new last hole of the side's range, which grows or shrinks with the line.
    to: usize,
    event: ProjectEvent,
}

/// The edit that gives `side`'s line the pitch at which its range holds `target` holes instead
/// of `current`, keeping the count of holes outside the range; `None` if no pitch does.
fn fit_pitch(
    project: &Project,
    which: Side,
    side: &SeamSide,
    target: usize,
    current: usize,
) -> Option<Fit> {
    let line = stitch_line(project, side.piece, side.child)?;
    let wanted = (line.hole_count() + target).checked_sub(current)?;
    let pitch = line.pitch_for_count(wanted)?;
    let shape = Shape::StitchLine(StitchLine {
        pitch,
        ..line.clone()
    });
    Some(Fit {
        side: which,
        pitch,
        to: (side.to.min(line.hole_count().saturating_sub(1)) + target).checked_sub(current)?,
        event: ProjectEvent::SetChildShape {
            id: side.piece,
            child: side.child,
            shape,
        },
    })
}

enum FooterAction {
    Reset,
    Cancel,
    Commit,
}

fn footer(
    ui: &mut egui::Ui,
    draft: &SeamLinkDraft,
    can_commit: bool,
    count: usize,
    language: Language,
) -> Option<FooterAction> {
    let palette = Palette::of(ui.painter());
    horizontal_rule(ui);
    egui::Frame::new()
        .fill(palette.surface.ruler)
        .inner_margin(egui::Margin::symmetric(EDGE as i8, 12))
        .corner_radius(egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: DIALOG_RADIUS,
            se: DIALOG_RADIUS,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let mut action = None;
                let reset = ui.add_enabled_ui(!draft.link.manual.is_empty(), |ui| {
                    icon_button(ui, language.label(Label::ResetManualPairs), paint_reset)
                });
                if reset.inner.clicked() {
                    action = Some(FooterAction::Reset);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let commit = match draft.editing {
                        Some(_) => language.label(Label::Save).to_owned(),
                        None => language.message(&Message::LinkPairsButton { count }),
                    };
                    if ui
                        .add_enabled(can_commit, Button::primary(commit))
                        .clicked()
                    {
                        action = Some(FooterAction::Commit);
                    }
                    if ui
                        .add(Button::secondary(language.label(Label::Cancel)))
                        .clicked()
                    {
                        action = Some(FooterAction::Cancel);
                    }
                });
                action
            })
            .inner
        })
        .inner
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    let palette = Palette::of(ui.painter());
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(LABEL_TEXT)
            .strong()
            .extra_letter_spacing(0.66)
            .color(palette.text.secondary),
    );
}

fn caption(ui: &mut egui::Ui, text: &str) {
    let palette = Palette::of(ui.painter());
    ui.label(
        egui::RichText::new(text)
            .size(LABEL_TEXT)
            .color(palette.text.secondary),
    );
}

/// A small label on a chip background, such as the hole count of a seam.
fn chip(ui: &mut egui::Ui, text: &str) {
    let palette = Palette::of(ui.painter());
    egui::Frame::new()
        .fill(palette.surface.chip)
        .corner_radius(CHIP_RADIUS)
        .inner_margin(CHIP_PADDING)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(SMALL_TEXT)
                    .strong()
                    .color(palette.text.label),
            );
        });
}

/// The A or B letter on its square.
fn side_badge(ui: &mut egui::Ui, letter: &str) {
    let palette = Palette::of(ui.painter());
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(20.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 4, palette.seam_link.side_badge);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        letter,
        egui::FontId::proportional(LABEL_TEXT),
        palette.seam_link.link_auto,
    );
}

/// The legend of the schematic: automatic, manual and unpaired holes.
fn legend(ui: &mut egui::Ui, language: Language) {
    let palette = Palette::of(ui.painter());
    let paint = &palette.seam_link;
    ui.spacing_mut().item_spacing.x = 6.0;
    // Right to left, so the entries are listed last first.
    for (label, color) in [
        (Label::PairFree, paint.link_free),
        (Label::PairManualLegend, paint.link_manual),
        (Label::PairAuto, paint.link_auto),
    ] {
        ui.label(
            egui::RichText::new(language.label(label))
                .size(LABEL_TEXT)
                .color(palette.text.label),
        );
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::hover());
        schematic::paint_diamond(ui.painter(), rect.center(), 4.5, None, color);
        ui.add_space(8.0);
    }
}

/// A button with a painted icon and a text, in the chip colors.
fn icon_button(
    ui: &mut egui::Ui,
    text: &str,
    paint: fn(&egui::Painter, egui::Rect, egui::Stroke),
) -> egui::Response {
    let palette = Palette::of(ui.painter());
    let colors = (
        palette.surface.chip,
        palette.border.chip,
        palette.text.label,
    );
    small_button(ui, text, Some(paint), colors)
}

/// The Reverse toggle: in the accent colors while on.
fn toggle_button(ui: &mut egui::Ui, on: bool, text: &str) -> egui::Response {
    let palette = Palette::of(ui.painter());
    let colors = if on {
        (
            palette.accent.container,
            palette.accent.border,
            palette.accent.base,
        )
    } else {
        (
            palette.surface.chip,
            palette.border.chip,
            palette.text.label,
        )
    };
    small_button(ui, text, Some(paint_reverse), colors)
}

/// A fit pitch button, in the warning colors.
fn warning_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let palette = Palette::of(ui.painter());
    let paint = &palette.seam_link;
    small_button(
        ui,
        text,
        None,
        (
            palette.surface.chip,
            paint.warning_border,
            paint.warning_text,
        ),
    )
}

/// A 26 px tall button of an optional icon and a text, in `(fill, stroke, content)` colors,
/// lighter on hover and faded while disabled.
fn small_button(
    ui: &mut egui::Ui,
    text: &str,
    icon: Option<fn(&egui::Painter, egui::Rect, egui::Stroke)>,
    (fill, stroke, content): (egui::Color32, egui::Color32, egui::Color32),
) -> egui::Response {
    const PADDING: f32 = 10.0;
    const GAP: f32 = 6.0;
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_owned(), egui::FontId::proportional(TEXT), content);
    let icon_width = if icon.is_some() { ICON_SIZE + GAP } else { 0.0 };
    let size = egui::vec2(
        galley.size().x + icon_width + 2.0 * PADDING,
        SMALL_BUTTON_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), text));
    if ui.is_rect_visible(rect) {
        let fade = if ui.is_enabled() { 1.0 } else { 0.45 };
        let hovered = ui.is_enabled() && response.hovered();
        let stroke_color = if hovered {
            content.gamma_multiply(0.6)
        } else {
            stroke
        };
        let painter = ui.painter();
        painter.rect(
            rect,
            5,
            fill.gamma_multiply(fade),
            egui::Stroke::new(1.0, stroke_color.gamma_multiply(fade)),
            egui::StrokeKind::Inside,
        );
        let mut x = rect.left() + PADDING;
        let content = content.gamma_multiply(fade);
        if let Some(icon) = icon {
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(x + ICON_SIZE / 2.0, rect.center().y),
                egui::Vec2::splat(ICON_SIZE),
            );
            icon(painter, icon_rect, egui::Stroke::new(1.3, content));
            x += icon_width;
        }
        painter.galley(
            egui::pos2(x, rect.center().y - galley.size().y / 2.0),
            galley,
            content,
        );
    }
    response
}
