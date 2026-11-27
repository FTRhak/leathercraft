//! The stitch, assembly and thread cards of the Link seams dialog, from the "Method" and
//! "Thread Row" rows of the design: a tile per stitch and per way of laying the pieces together,
//! each with a small drawing, and under them the locked holes, the overlap and the side on top;
//! then the thread's type, thickness and color, with the estimate of how much the seam takes.

use super::{CARD_PADDING, CARD_RADIUS, EDGE, LABEL_TEXT, SMALL_TEXT, SeamLinkDraft, caption};
use crate::editor::format_length;
use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value};
use crate::ui_components::group_button::GroupButton;
use crate::ui_components::input::{NumberInput, TextInput};
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use egui::epaint::CubicBezierShape;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{Project, Units};
use leathercraft::seams::{LinkPairs, stitch_line};
use leathercraft::sewing::{
    Assembly, MAX_LOCK_HOLES, MAX_OVERLAP_MM, MAX_THREAD_MM, MIN_OVERLAP_MM, MIN_THREAD_MM, OnTop,
    Sewing, StitchMethod, Thread, ThreadColor, ThreadKind, ThreadShade,
};
use std::ops::RangeInclusive;

const CARD_GAP: f32 = 12.0;
const TILE_GAP: f32 = 6.0;
const TILE_HEIGHT: f32 = 75.0;
const TILE_RADIUS: u8 = 6;
const TILE_PADDING: f32 = 8.0;
/// The drawings are made on a 64 × 24 grid, as in the design.
const GLYPH_SIZE: egui::Vec2 = egui::vec2(64.0, 24.0);
const TITLE_HEIGHT: f32 = 16.0;
const OPTIONS_HEIGHT: f32 = 26.0;
const FIELD_WIDTH: f32 = 84.0;
/// The thread card: the gap between its groups of fields, the type dropdown's narrowest and
/// widest, the swatches and the hex field.
const GROUP_GAP: f32 = 24.0;
const KIND_WIDTH: (f32, f32) = (170.0, 250.0);
/// What the thickness and color groups take beside the type dropdown.
const THREAD_FIELDS_REST: f32 = 560.0;
const SWATCH_SIZE: f32 = 22.0;
const SWATCH_DOT: f32 = 15.0;
const SWATCH_GAP: f32 = 3.0;
const HEX_WIDTH: f32 = 76.0;

/// The stitch and assembly cards side by side, and the thread card under them, with the
/// dialog's margins.
pub(super) fn sewing_cards(
    ui: &mut egui::Ui,
    draft: &mut SeamLinkDraft,
    project: &Project,
    pairs: Option<&LinkPairs>,
    language: Language,
) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(EDGE as i8, 14))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = CARD_GAP;
            let estimate = pairs.and_then(|pairs| thread_text(draft, project, pairs, language));
            let sewing = &mut draft.link.sewing;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                let width = (ui.available_width() - CARD_GAP) / 2.0;
                card(ui, width, |ui, inner| {
                    stitch_card(ui, inner, sewing, language)
                });
                card(ui, width, |ui, inner| {
                    assembly_card(ui, inner, sewing, project.units, language)
                });
            });
            let width = ui.available_width();
            card(ui, width, |ui, _| {
                thread_card(ui, &mut sewing.thread, estimate, project.units, language)
            });
        });
}

/// The thread the seam takes, measured along A's range at A's pitch.
fn thread_text(
    draft: &SeamLinkDraft,
    project: &Project,
    pairs: &LinkPairs,
    language: Language,
) -> Option<String> {
    let side = &draft.link.a;
    let line = stitch_line(project, side.piece, side.child)?;
    let sewing = &draft.link.sewing;
    let thread = sewing.thread_mm(pairs.length_a, line.pitch);
    Some(language.message(&Message::ThreadEstimate {
        length: format_length(language, project.units, thread),
        factor_tenths: sewing.stitch.thread_factor_tenths(),
    }))
}

fn card(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui, f32)) {
    let palette = Palette::of(ui.painter());
    let inner = width - (CARD_PADDING.left + CARD_PADDING.right) as f32 - 2.0;
    egui::Frame::new()
        .fill(palette.surface.ruler)
        .stroke(egui::Stroke::new(1.0, palette.border.control))
        .corner_radius(CARD_RADIUS)
        .inner_margin(CARD_PADDING)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner);
                ui.spacing_mut().item_spacing.y = 10.0;
                add(ui, inner);
            });
        });
}

fn stitch_card(ui: &mut egui::Ui, width: f32, sewing: &mut Sewing, language: Language) {
    title_row(
        ui,
        language.label(Label::StitchMethodTitle),
        language.label(Label::StitchMethodHint(sewing.stitch)),
    );
    tiles(ui, width, &mut sewing.stitch, StitchMethod::ALL, |method| {
        (
            language.label(Label::StitchMethod(method)),
            language.label(Label::StitchMethodNote(method)),
            Glyph::Stitch(method),
        )
    });
    options_row(ui, |ui| {
        caption(ui, language.label(Label::LockHoles));
        ui.add(
            NumberInput::new(&mut sewing.lock_holes)
                .range(0..=MAX_LOCK_HOLES)
                .unit(language.label(Label::HolesShort))
                .width(FIELD_WIDTH)
                .id_salt("seam_link_lock"),
        );
    });
}

fn assembly_card(
    ui: &mut egui::Ui,
    width: f32,
    sewing: &mut Sewing,
    units: Units,
    language: Language,
) {
    let hint = language.message(&Message::AssemblyHint {
        assembly: sewing.assembly,
        top: sewing.on_top.letter(),
        under: sewing.on_top.other_letter(),
    });
    title_row(ui, language.label(Label::AssemblyTitle), &hint);
    tiles(ui, width, &mut sewing.assembly, Assembly::ALL, |assembly| {
        (
            language.label(Label::Assembly(assembly)),
            language.label(Label::AssemblyNote(assembly)),
            Glyph::Assembly(assembly),
        )
    });
    let (overlap, stacks) = (sewing.assembly.uses_overlap(), sewing.assembly.stacks());
    options_row(ui, |ui| {
        caption(ui, language.label(Label::Overlap));
        ui.add_enabled_ui(overlap, |ui| {
            length_input(
                ui,
                &mut sewing.overlap_mm,
                MIN_OVERLAP_MM..=MAX_OVERLAP_MM,
                Precision::Overlap,
                units,
                language,
                "seam_link_overlap",
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(
                stacks,
                GroupButton::new(&mut sewing.on_top)
                    .option(OnTop::B, "B")
                    .option(OnTop::A, "A"),
            );
            caption(ui, language.label(Label::OnTop));
        });
    });
}

/// The thread's type, thickness and color, with the estimate of how much the seam takes beside
/// the title.
fn thread_card(
    ui: &mut egui::Ui,
    thread: &mut Thread,
    estimate: Option<String>,
    units: Units,
    language: Language,
) {
    title_row(
        ui,
        language.label(Label::ThreadTitle),
        estimate.as_deref().unwrap_or_default(),
    );
    options_row(ui, |ui| {
        // The dropdown gives up width first when the dialog is narrow.
        let kind_width =
            (ui.available_width() - THREAD_FIELDS_REST).clamp(KIND_WIDTH.0, KIND_WIDTH.1);
        caption(ui, language.label(Label::ThreadKindCaption));
        Dropdown::new(
            "seam_link_thread_kind",
            language.label(Label::ThreadKind(thread.kind)),
        )
        .width(kind_width)
        .show_ui(ui, |ui| {
            for kind in ThreadKind::ALL {
                let item = DropdownItem::new(language.label(Label::ThreadKind(kind)));
                dropdown_value(ui, &mut thread.kind, kind, item);
            }
        });
        ui.add_space(GROUP_GAP - ui.spacing().item_spacing.x);
        caption(ui, language.label(Label::ThreadDiameter));
        length_input(
            ui,
            &mut thread.diameter_mm,
            MIN_THREAD_MM..=MAX_THREAD_MM,
            Precision::Thread,
            units,
            language,
            "seam_link_thread_diameter",
        );
        ui.add_space(GROUP_GAP - ui.spacing().item_spacing.x);
        caption(ui, language.label(Label::ThreadColorCaption));
        thread_color(ui, &mut thread.color, language);
    });
}

/// A swatch per named shade, a last one that opens a color picker (showing the color while it
/// is none of the shades), the hex field and the name of the color.
fn thread_color(ui: &mut egui::Ui, color: &mut ThreadColor, language: Language) {
    let shade = color.shade();
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = SWATCH_GAP;
        for option in ThreadShade::ALL {
            let name = language.label(Label::ThreadShade(option));
            if swatch(ui, Some(option.color()), shade == Some(option), name).clicked() {
                *color = option.color();
            }
        }
        let custom = shade.is_none().then_some(*color);
        let name = language.label(Label::ThreadCustomColor);
        let response = swatch(ui, custom, custom.is_some(), name);
        let palette = Palette::of(ui.painter());
        let frame = egui::Frame::popup(ui.style())
            .fill(palette.surface.ruler)
            .stroke(egui::Stroke::new(1.0, palette.border.control));
        egui::Popup::from_toggle_button_response(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .frame(frame)
            .show(|ui| {
                let [red, green, blue] = color.rgb();
                let mut picked = egui::Color32::from_rgb(red, green, blue);
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut picked,
                    egui::color_picker::Alpha::Opaque,
                );
                *color = ThreadColor([picked.r(), picked.g(), picked.b()]);
            });
    });
    hex_field(ui, color);
    let name = match shade {
        Some(shade) => language.label(Label::ThreadShade(shade)),
        None => language.label(Label::ThreadCustomColor),
    };
    muted(ui, name, LABEL_TEXT);
}

/// The color as `#RRGGBB`. What is typed stays while the field has the focus, and is taken as
/// soon as it reads as a color.
fn hex_field(ui: &mut egui::Ui, color: &mut ThreadColor) {
    let id = egui::Id::new("seam_link_thread_hex");
    let mut text = ui
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_else(|| color.to_hex());
    let response = ui.add(
        TextInput::new(&mut text)
            .width(HEX_WIDTH)
            .id_salt("seam_link_thread_hex"),
    );
    if response.has_focus() {
        if let Some(typed) = ThreadColor::from_hex(&text) {
            *color = typed;
        }
        ui.data_mut(|data| data.insert_temp(id, text));
    } else {
        ui.data_mut(|data| data.remove::<String>(id));
    }
}

/// A round swatch of `color`, ringed while it is chosen; without a color, the swatch that opens
/// the picker, a plus in a ring.
fn swatch(
    ui: &mut egui::Ui,
    color: Option<ThreadColor>,
    selected: bool,
    name: &str,
) -> egui::Response {
    let palette = Palette::of(ui.painter());
    let (rect, response) =
        ui.allocate_exact_size(egui::Vec2::splat(SWATCH_SIZE), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, selected, name)
    });
    let response = response.on_hover_text(name);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    let center = rect.center();
    let ring = if selected {
        Some(palette.seam_link.link_auto)
    } else if response.hovered() || response.has_focus() {
        Some(palette.border.chip)
    } else {
        None
    };
    if let Some(ring) = ring {
        painter.circle_stroke(
            center,
            SWATCH_SIZE / 2.0 - 0.75,
            egui::Stroke::new(1.5, ring),
        );
    }
    let radius = SWATCH_DOT / 2.0;
    match color {
        // The color is document data, not a theme color, so it is painted as it is; the edge
        // keeps a black thread visible on a dark card and a white one on a light card.
        Some(color) => {
            let [red, green, blue] = color.rgb();
            painter.circle(
                center,
                radius,
                egui::Color32::from_rgb(red, green, blue),
                egui::Stroke::new(1.0, palette.border.control),
            );
        }
        None => {
            painter.circle(
                center,
                radius,
                palette.surface.input,
                egui::Stroke::new(1.0, palette.seam_link.arrow),
            );
            let arm = radius * 0.5;
            let stroke = egui::Stroke::new(1.2, palette.text.secondary);
            painter.line_segment(
                [center - egui::vec2(arm, 0.0), center + egui::vec2(arm, 0.0)],
                stroke,
            );
            painter.line_segment(
                [center - egui::vec2(0.0, arm), center + egui::vec2(0.0, arm)],
                stroke,
            );
        }
    }
    response
}

/// How finely a length field steps, in each unit.
#[derive(Clone, Copy)]
enum Precision {
    Overlap,
    Thread,
}

impl Precision {
    fn step_and_decimals(self, units: Units) -> (f64, usize) {
        match (self, units) {
            (Self::Overlap, Units::Millimeters) => (0.5, 1),
            (Self::Overlap, Units::Inches) => (0.02, 2),
            (Self::Thread, Units::Millimeters) => (0.05, 2),
            (Self::Thread, Units::Inches) => (0.002, 3),
        }
    }
}

/// A length in display units; written back only when changed, so that an untouched value isn't
/// rounded through the units.
fn length_input(
    ui: &mut egui::Ui,
    millimeters: &mut f64,
    range: RangeInclusive<f64>,
    precision: Precision,
    units: Units,
    language: Language,
    id_salt: &str,
) {
    let scale = units.from_millimeters(1.0);
    let current = *millimeters * scale;
    let mut value = current;
    let (step, decimals) = precision.step_and_decimals(units);
    ui.add(
        NumberInput::new(&mut value)
            .range(range.start() * scale..=range.end() * scale)
            .step(step)
            .decimals(decimals)
            .unit(language.unit_symbol(units))
            .width(FIELD_WIDTH)
            .id_salt(id_salt),
    );
    if value != current {
        *millimeters = (value / scale).clamp(*range.start(), *range.end());
    }
}

/// The card's title, and a hint on the chosen tile at the right, cut short when it doesn't fit.
fn title_row(ui: &mut egui::Ui, title: &str, hint: &str) {
    let palette = Palette::of(ui.painter());
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), TITLE_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_height(TITLE_HEIGHT);
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .size(LABEL_TEXT)
                    .strong()
                    .extra_letter_spacing(0.66)
                    .color(palette.text.secondary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(hint)
                            .size(LABEL_TEXT)
                            .color(palette.text.muted),
                    )
                    .truncate(),
                );
            });
        },
    );
}

fn options_row(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), OPTIONS_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_height(OPTIONS_HEIGHT);
            ui.spacing_mut().item_spacing.x = 6.0;
            add(ui);
        },
    );
}

fn muted(ui: &mut egui::Ui, text: &str, size: f32) {
    let palette = Palette::of(ui.painter());
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .size(size)
                .color(palette.text.muted),
        )
        .truncate(),
    );
}

/// A row of tiles choosing one value: a click on a tile picks it.
fn tiles<V: Copy + PartialEq + std::hash::Hash, const N: usize>(
    ui: &mut egui::Ui,
    width: f32,
    current: &mut V,
    values: [V; N],
    describe: impl Fn(V) -> (&'static str, &'static str, Glyph),
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = TILE_GAP;
        let tile_width = (width - TILE_GAP * (N - 1) as f32) / N as f32;
        for value in values {
            let (name, note, glyph) = describe(value);
            if tile(ui, tile_width, *current == value, name, note, glyph).clicked() {
                *current = value;
            }
        }
    });
}

fn tile(
    ui: &mut egui::Ui,
    width: f32,
    selected: bool,
    name: &str,
    note: &str,
    glyph: Glyph,
) -> egui::Response {
    let palette = Palette::of(ui.painter());
    let paint = &palette.seam_link;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, TILE_HEIGHT), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, selected, name)
    });
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let (fill, border) = if selected {
        (paint.tile_selected, paint.link_auto)
    } else if response.hovered() {
        (palette.surface.input, palette.border.chip)
    } else {
        (palette.surface.input, palette.border.control)
    };
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        TILE_RADIUS,
        fill,
        egui::Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(1.0),
            TILE_RADIUS,
            egui::Stroke::new(1.0, palette.accent.border),
            egui::StrokeKind::Outside,
        );
    }

    // The drawing shrinks on narrow tiles rather than spilling over their edges.
    let scale = ((width - TILE_PADDING) / GLYPH_SIZE.x).min(1.0);
    let glyph_rect = egui::Rect::from_center_size(
        egui::pos2(
            rect.center().x,
            rect.top() + TILE_PADDING + GLYPH_SIZE.y / 2.0,
        ),
        GLYPH_SIZE * scale,
    );
    let thread = if selected {
        paint.link_auto
    } else {
        paint.thread_idle
    };
    glyph.paint(&Pen {
        painter: &painter,
        origin: glyph_rect.min,
        scale,
        palette,
        thread,
    });

    let mut y = rect.top() + TILE_PADDING + GLYPH_SIZE.y + 5.0;
    let note_color = if selected {
        paint.link_auto
    } else {
        palette.text.muted
    };
    for (text, size, color) in [
        (name, LABEL_TEXT, palette.text.primary),
        (note, SMALL_TEXT, note_color),
    ] {
        let mut job = egui::text::LayoutJob::simple_singleline(
            text.to_owned(),
            egui::FontId::proportional(size),
            color,
        );
        job.wrap = egui::text::TextWrapping::truncate_at_width(width - 2.0 * 4.0);
        let galley = painter.layout_job(job);
        painter.galley(
            egui::pos2(rect.center().x - galley.size().x / 2.0, y),
            galley.clone(),
            color,
        );
        y += galley.size().y + 4.0;
    }
    response
}

/// Draws on a tile in the design's 64 × 24 grid.
struct Pen<'a> {
    painter: &'a egui::Painter,
    origin: egui::Pos2,
    scale: f32,
    palette: &'a Palette,
    thread: egui::Color32,
}

impl Pen<'_> {
    fn at(&self, x: f32, y: f32) -> egui::Pos2 {
        self.origin + egui::vec2(x, y) * self.scale
    }

    fn line(&self, (x1, y1): (f32, f32), (x2, y2): (f32, f32), stroke: egui::Stroke) {
        let stroke = egui::Stroke::new(stroke.width * self.scale, stroke.color);
        self.painter
            .line_segment([self.at(x1, y1), self.at(x2, y2)], stroke);
    }

    fn thread(&self, from: (f32, f32), to: (f32, f32)) {
        self.line(from, to, egui::Stroke::new(1.6, self.thread));
    }

    /// A thin line for an edge of a piece seen from above.
    fn edge(&self, y: f32) {
        self.line(
            (0.0, y),
            (64.0, y),
            egui::Stroke::new(1.0, self.palette.seam_link.arrow),
        );
    }

    fn hole(&self, x: f32, y: f32) {
        self.painter.circle(
            self.at(x, y),
            1.6 * self.scale,
            self.palette.surface.input,
            egui::Stroke::new(1.0, self.palette.text.secondary),
        );
    }

    /// A piece in cross-section.
    fn layer(&self, x: f32, y: f32, width: f32, color: egui::Color32) {
        let rect = egui::Rect::from_min_max(self.at(x, y), self.at(x + width, y + 5.0));
        self.painter.rect_filled(rect, 1, color);
    }

    fn curve(&self, points: [(f32, f32); 4], stroke: egui::Stroke) {
        let points = points.map(|(x, y)| self.at(x, y));
        self.painter.add(CubicBezierShape::from_points_stroke(
            points,
            false,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(stroke.width * self.scale, stroke.color),
        ));
    }
}

#[derive(Clone, Copy)]
enum Glyph {
    Stitch(StitchMethod),
    Assembly(Assembly),
}

impl Glyph {
    fn paint(self, pen: &Pen) {
        match self {
            Self::Stitch(method) => paint_stitch(pen, method),
            Self::Assembly(assembly) => paint_assembly(pen, assembly),
        }
    }
}

/// The stitch seen from the grain side.
fn paint_stitch(pen: &Pen, method: StitchMethod) {
    const ONE_ROW: [f32; 5] = [6.0, 19.0, 32.0, 45.0, 58.0];
    const TWO_ROWS: [f32; 5] = [8.0, 20.0, 32.0, 44.0, 56.0];
    match method {
        StitchMethod::Saddle => {
            for x in &ONE_ROW[..4] {
                pen.thread((x + 2.0, 13.5), (x + 11.0, 10.5));
            }
            ONE_ROW.iter().for_each(|&x| pen.hole(x, 12.0));
        }
        StitchMethod::Running => {
            let back = egui::Stroke::new(1.0, pen.palette.seam_link.arrow);
            for (index, x) in ONE_ROW[..4].iter().enumerate() {
                if index % 2 == 0 {
                    pen.thread((x + 2.0, 12.0), (x + 11.0, 12.0));
                } else {
                    // The stretch on the flesh side, dashed.
                    for dash in 0..3 {
                        let start = x + 2.0 + 3.5 * dash as f32;
                        pen.line((start, 12.0), (start + 2.0, 12.0), back);
                    }
                }
            }
            ONE_ROW.iter().for_each(|&x| pen.hole(x, 12.0));
        }
        StitchMethod::Baseball => {
            pen.edge(10.0);
            pen.edge(14.0);
            for x in TWO_ROWS {
                pen.thread((x, 5.0), (x + 6.0, 11.2));
                pen.thread((x, 19.0), (x + 6.0, 12.8));
            }
            for x in TWO_ROWS {
                pen.hole(x, 5.0);
                pen.hole(x, 19.0);
            }
        }
        StitchMethod::Whip => {
            pen.line(
                (2.0, 6.0),
                (62.0, 6.0),
                egui::Stroke::new(2.0, pen.palette.text.secondary),
            );
            for x in TWO_ROWS {
                pen.thread((x, 16.0), (x + 7.0, 3.0));
            }
            TWO_ROWS.iter().for_each(|&x| pen.hole(x, 16.0));
        }
        StitchMethod::Cross => {
            const XS: [f32; 4] = [8.0, 24.0, 40.0, 56.0];
            pen.edge(10.0);
            pen.edge(14.0);
            for x in &XS[..3] {
                pen.thread((*x, 5.0), (x + 16.0, 19.0));
                pen.thread((*x, 19.0), (x + 16.0, 5.0));
            }
            for x in XS {
                pen.hole(x, 5.0);
                pen.hole(x, 19.0);
            }
        }
    }
}

/// The pieces in cross-section, A lighter than B, with the seam through them.
fn paint_assembly(pen: &Pen, assembly: Assembly) {
    let (a, b) = (
        pen.palette.seam_link.leather_a,
        pen.palette.seam_link.leather_b,
    );
    match assembly {
        Assembly::FleshToFlesh => {
            pen.layer(4.0, 6.0, 56.0, a);
            pen.layer(4.0, 12.0, 56.0, b);
            pen.thread((52.0, 2.0), (52.0, 21.0));
        }
        Assembly::Lapped => {
            pen.layer(2.0, 7.0, 36.0, a);
            pen.layer(26.0, 12.0, 36.0, b);
            pen.thread((32.0, 3.0), (32.0, 21.0));
        }
        Assembly::Butted => {
            pen.layer(2.0, 10.0, 28.0, a);
            pen.layer(34.0, 10.0, 28.0, b);
            pen.thread((25.0, 6.0), (39.0, 19.0));
            pen.thread((25.0, 19.0), (39.0, 6.0));
        }
        Assembly::Turned => {
            let arrow = pen.palette.text.secondary;
            pen.layer(4.0, 6.0, 42.0, a);
            pen.layer(4.0, 12.0, 42.0, b);
            pen.thread((40.0, 2.0), (40.0, 21.0));
            pen.curve(
                [(50.0, 3.0), (60.0, 5.0), (60.0, 17.0), (50.0, 19.0)],
                egui::Stroke::new(1.2, arrow),
            );
            pen.painter.add(egui::Shape::convex_polygon(
                vec![pen.at(52.0, 16.5), pen.at(47.0, 19.0), pen.at(52.0, 21.5)],
                arrow,
                egui::Stroke::NONE,
            ));
        }
        Assembly::Folded => {
            pen.layer(2.0, 5.0, 46.0, a);
            pen.layer(2.0, 14.0, 46.0, a);
            pen.curve(
                [(47.0, 7.5), (56.0, 7.5), (56.0, 16.5), (47.0, 16.5)],
                egui::Stroke::new(5.0, a),
            );
            pen.thread((30.0, 2.0), (30.0, 22.0));
        }
    }
}
