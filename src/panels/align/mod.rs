//! The Align panel, from the "Alignment Body" of the Brilliant design: three groups of
//! `AlignButton`s, the Auto box and the fixed gap of Distribute Spacing, and the Align To choice.

use crate::ui_components::align_button::{self, AlignButton};
use crate::ui_components::checkbox::Checkbox;
use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value};
use crate::ui_components::icons::glyph;
use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::align::{AlignTo, Arrange, Axis, Edge, Spacing};
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;

/// Between the panel's edges and its content, as in the design.
const PANEL_MARGIN: i8 = 12;
/// Between the groups, the divider and the Align To row.
const SECTION_GAP: f32 = 10.0;
/// Between a group's title and its buttons.
const TITLE_GAP: f32 = 6.0;
const TITLE_SIZE: f32 = 11.0;
/// The smallest gap between the buttons of a row, which spread over the panel's width.
const ROW_GAP: f32 = 4.0;
const SPACING_ROW_GAP: f32 = 8.0;
/// Around the Auto checkbox, which sits a little apart from the buttons and the field.
const CHECKBOX_PADDING: f32 = 4.0;
const MIN_FIELD_WIDTH: f32 = 80.0;
const REFERENCE_WIDTH: f32 = 112.0;

/// The Align To choice; the key object itself is the primary selected piece or child.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reference {
    #[default]
    Selection,
    KeyObject,
    Project,
    /// The base shape of the selected children's piece; offered only for children.
    Parent,
}

impl Reference {
    const ALL: [Self; 4] = [
        Self::Selection,
        Self::KeyObject,
        Self::Project,
        Self::Parent,
    ];

    fn label(self) -> Label {
        match self {
            Self::Selection => Label::AlignToSelection,
            Self::KeyObject => Label::AlignToKeyObject,
            Self::Project => Label::AlignToProject,
            Self::Parent => Label::AlignToParent,
        }
    }
}

/// What the panel arranges: the selected pieces, or the selected children of one piece.
/// `key` is the primary one.
#[derive(Clone, Copy, Debug)]
pub enum Targets<'a> {
    Pieces {
        ids: &'a [u64],
        key: Option<u64>,
    },
    Children {
        piece: u64,
        ids: &'a [u64],
        key: Option<u64>,
    },
}

impl Targets<'_> {
    fn ids(&self) -> &[u64] {
        match self {
            Self::Pieces { ids, .. } | Self::Children { ids, .. } => ids,
        }
    }

    fn key(&self) -> Option<u64> {
        match self {
            Self::Pieces { key, .. } | Self::Children { key, .. } => *key,
        }
    }

    fn children(&self) -> bool {
        matches!(self, Self::Children { .. })
    }
}

/// The panel's settings, which are interface state rather than part of the project.
#[derive(Clone, Copy, Debug)]
pub struct AlignOptions {
    pub reference: Reference,
    pub spacing_auto: bool,
    pub spacing_mm: f64,
}

impl Default for AlignOptions {
    fn default() -> Self {
        Self {
            reference: Reference::default(),
            spacing_auto: true,
            spacing_mm: 10.0,
        }
    }
}

/// The font glyph of an arrange button; `Horizontal` works on left, center or right edges.
fn arrange_glyph(arrange: Arrange) -> char {
    match arrange {
        Arrange::Align(Axis::Horizontal, Edge::Min) => glyph::ALIGN_LEFT,
        Arrange::Align(Axis::Horizontal, Edge::Center) => glyph::ALIGN_CENTER_X,
        Arrange::Align(Axis::Horizontal, Edge::Max) => glyph::ALIGN_RIGHT,
        Arrange::Align(Axis::Vertical, Edge::Min) => glyph::ALIGN_TOP,
        Arrange::Align(Axis::Vertical, Edge::Center) => glyph::ALIGN_CENTER_Y,
        Arrange::Align(Axis::Vertical, Edge::Max) => glyph::ALIGN_BOTTOM,
        Arrange::Distribute(Axis::Vertical, Edge::Min) => glyph::DISTRIBUTE_TOP,
        Arrange::Distribute(Axis::Vertical, Edge::Center) => glyph::DISTRIBUTE_CENTER_Y,
        Arrange::Distribute(Axis::Vertical, Edge::Max) => glyph::DISTRIBUTE_BOTTOM,
        Arrange::Distribute(Axis::Horizontal, Edge::Min) => glyph::DISTRIBUTE_LEFT,
        Arrange::Distribute(Axis::Horizontal, Edge::Center) => glyph::DISTRIBUTE_CENTER_X,
        Arrange::Distribute(Axis::Horizontal, Edge::Max) => glyph::DISTRIBUTE_RIGHT,
        Arrange::Space(Axis::Vertical) => glyph::SPACE_VERTICAL,
        Arrange::Space(Axis::Horizontal) => glyph::SPACE_HORIZONTAL,
    }
}

/// Small text in the color of the group titles, as in the design.
fn title_text(ui: &egui::Ui, text: &str) -> egui::RichText {
    let color = Palette::of(ui.painter()).text.group_title;
    egui::RichText::new(text).size(TITLE_SIZE).color(color)
}

/// One button of `arrange`; returns whether it was clicked.
fn arrange_button(ui: &mut egui::Ui, arrange: Arrange, enabled: bool, language: Language) -> bool {
    let button = AlignButton::new(
        arrange_glyph(arrange),
        language.label(Label::Arrange(arrange)),
    );
    ui.add_enabled(enabled, button).clicked()
}

/// A titled group: the title above the content.
fn group<R>(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = TITLE_GAP;
        let text = title_text(ui, title);
        ui.label(text);
        add_contents(ui)
    })
    .inner
}

/// A row of buttons spread over the panel's width, as in the design; returns the one clicked.
fn arrange_row(
    ui: &mut egui::Ui,
    options: &[Arrange],
    enabled: impl Fn(Arrange) -> bool,
    language: Language,
) -> Option<Arrange> {
    let count = options.len() as f32;
    let gap = ((ui.available_width() - count * align_button::SIZE.x) / (count - 1.0))
        .floor()
        .max(ROW_GAP);
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(gap, ROW_GAP);
        for &arrange in options {
            if arrange_button(ui, arrange, enabled(arrange), language) {
                clicked = Some(arrange);
            }
        }
    });
    clicked
}

/// The spacing buttons, then the Auto box and the fixed gap, which apply only against the key
/// object.
fn spacing_row(
    ui: &mut egui::Ui,
    units: Units,
    options: &mut AlignOptions,
    enabled: impl Fn(Arrange) -> bool,
    language: Language,
) -> Option<Arrange> {
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACING_ROW_GAP;
        for &arrange in &Arrange::SPACE {
            if arrange_button(ui, arrange, enabled(arrange), language) {
                clicked = Some(arrange);
            }
        }
        ui.add_enabled_ui(options.reference == Reference::KeyObject, |ui| {
            ui.spacing_mut().item_spacing.x = CHECKBOX_PADDING;
            ui.add_space(CHECKBOX_PADDING);
            ui.add(Checkbox::new(
                &mut options.spacing_auto,
                language.label(Label::SpacingAuto),
            ))
            .on_hover_text(language.label(Label::Spacing));
            ui.add_space(CHECKBOX_PADDING);
            let scale = units.from_millimeters(1.0);
            let mut value = options.spacing_mm * scale;
            let (step, decimals) = match units {
                Units::Millimeters => (1.0, 1),
                Units::Inches => (0.05, 3),
            };
            let field = NumberInput::new(&mut value)
                .id_salt("align_spacing")
                .range(0.0..=f64::MAX)
                .step(step)
                .decimals(decimals)
                .unit(language.unit_symbol(units))
                .width(ui.available_width().max(MIN_FIELD_WIDTH));
            let response = ui
                .add_enabled(!options.spacing_auto, field)
                .on_hover_text(language.label(Label::Spacing));
            if response.changed() {
                options.spacing_mm = value / scale;
            }
        });
    });
    clicked
}

/// A 1 px line across the panel, between the buttons and the Align To choice.
fn divider(ui: &mut egui::Ui) {
    let color = Palette::of(ui.painter()).border.divider;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}

/// The Align To label and its dropdown, at the right edge of the panel.
fn reference_row(
    ui: &mut egui::Ui,
    options: &mut AlignOptions,
    children: bool,
    language: Language,
) {
    ui.horizontal(|ui| {
        let text = title_text(ui, language.label(Label::AlignTo));
        ui.label(text);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Dropdown::new("align_to", language.label(options.reference.label()))
                .width(REFERENCE_WIDTH)
                .show_ui(ui, |ui| {
                    for reference in Reference::ALL {
                        if reference == Reference::Parent && !children {
                            continue;
                        }
                        let item = DropdownItem::new(language.label(reference.label()));
                        dropdown_value(ui, &mut options.reference, reference, item);
                    }
                });
        });
    });
}

/// The Align panel, acting on `targets`.
pub fn show_panel(
    ui: &mut egui::Ui,
    units: Units,
    targets: Targets,
    options: &mut AlignOptions,
    language: Language,
) -> Option<ProjectEvent> {
    let children = targets.children();
    let to = match options.reference {
        Reference::Selection => Some(AlignTo::Selection),
        Reference::KeyObject => targets.key().map(AlignTo::KeyObject),
        Reference::Project => Some(AlignTo::Project),
        Reference::Parent if children => Some(AlignTo::Parent),
        Reference::Parent => None,
    };
    // As in Illustrator, a fixed gap is measured from the key object.
    let fixed = options.reference == Reference::KeyObject && !options.spacing_auto;
    let spacing = if fixed {
        Spacing::Fixed(options.spacing_mm)
    } else {
        Spacing::Auto
    };
    // Against the project's or the parent's bounds, fewer items are enough, since they are not
    // the reference.
    let to_project = matches!(to, Some(AlignTo::Project | AlignTo::Parent));
    let enabled = |arrange: Arrange| {
        let minimum = match arrange {
            Arrange::Align(..) if to_project => 1,
            Arrange::Align(..) => 2,
            Arrange::Space(_) if fixed => 2,
            Arrange::Distribute(..) | Arrange::Space(_) if to_project => 2,
            Arrange::Distribute(..) | Arrange::Space(_) => 3,
        };
        to.is_some() && targets.ids().len() >= minimum
    };

    let clicked = egui::ScrollArea::vertical()
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(PANEL_MARGIN)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = SECTION_GAP;
                    let mut clicked = group(ui, language.label(Label::AlignObjects), |ui| {
                        arrange_row(ui, &Arrange::ALIGN, enabled, language)
                    });
                    clicked =
                        clicked.or(group(ui, language.label(Label::DistributeObjects), |ui| {
                            arrange_row(ui, &Arrange::DISTRIBUTE, enabled, language)
                        }));
                    clicked =
                        clicked.or(group(ui, language.label(Label::DistributeSpacing), |ui| {
                            spacing_row(ui, units, options, enabled, language)
                        }));
                    divider(ui);
                    reference_row(ui, options, children, language);
                    let hint = match options.reference {
                        Reference::KeyObject if children => Some(Label::KeyShapeHint),
                        Reference::KeyObject => Some(Label::KeyObjectHint),
                        Reference::Parent if children => Some(Label::ParentHint),
                        _ => None,
                    };
                    if let Some(hint) = hint {
                        let color = Palette::of(ui.painter()).text.muted;
                        ui.label(
                            egui::RichText::new(language.label(hint))
                                .size(TITLE_SIZE)
                                .color(color),
                        );
                    }
                    clicked
                })
                .inner
        })
        .inner;

    let arrange = clicked?;
    let to = to?;
    Some(match targets {
        Targets::Pieces { ids, .. } => ProjectEvent::ArrangePieces {
            ids: ids.to_vec(),
            arrange,
            to,
            spacing,
        },
        Targets::Children { piece, ids, .. } => ProjectEvent::ArrangeChildren {
            id: piece,
            children: ids.to_vec(),
            arrange,
            to,
            spacing,
        },
    })
}
