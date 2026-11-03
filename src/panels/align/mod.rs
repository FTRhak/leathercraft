use crate::icons::icon_button;
use bevy_egui::egui;
use leathercraft::align::{AlignTo, Arrange, Axis, Edge, Spacing};
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_MARGIN: f32 = 6.0;

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

/// The edge or center of a bar spanning `min..max`.
fn edge_at(edge: Edge, min: f32, max: f32) -> f32 {
    match edge {
        Edge::Min => min,
        Edge::Center => (min + max) * 0.5,
        Edge::Max => max,
    }
}

/// Bars drawn along the axis the pieces move on, as `(min, max)` along it and across it.
/// The vertical icons are the horizontal ones with the coordinates swapped.
fn paint_arrange_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    stroke: egui::Stroke,
    arrange: Arrange,
) {
    let rect = rect.shrink(ICON_MARGIN);
    let at = |along: f32, across: f32| match arrange.axis() {
        Axis::Horizontal => rect.lerp_inside(egui::vec2(along, across)),
        Axis::Vertical => rect.lerp_inside(egui::vec2(across, along)),
    };
    let bar = |along: (f32, f32), across: (f32, f32)| {
        let bar = egui::Rect::from_two_pos(at(along.0, across.0), at(along.1, across.1));
        painter.rect_stroke(bar, 0.0, stroke, egui::StrokeKind::Middle);
    };
    let faded = egui::Stroke::new(1.0, stroke.color.gamma_multiply(0.5));
    let line = |along: f32| painter.line_segment([at(along, 0.0), at(along, 1.0)], faded);
    // Three bars of different sizes, spread along the axis.
    let spread = [
        ((0.0, 0.2), (0.25, 0.75)),
        ((0.35, 0.65), (0.1, 0.9)),
        ((0.8, 1.0), (0.3, 0.7)),
    ];
    match arrange {
        Arrange::Align(_, edge) => {
            line(edge_at(edge, 0.0, 1.0));
            for (length, across) in [(0.85, (0.12, 0.4)), (0.5, (0.6, 0.88))] {
                let start = match edge {
                    Edge::Min => 0.0,
                    Edge::Center => (1.0 - length) * 0.5,
                    Edge::Max => 1.0 - length,
                };
                bar((start, start + length), across);
            }
        }
        Arrange::Distribute(_, edge) => {
            for (along, across) in spread {
                line(edge_at(edge, along.0, along.1));
                bar(along, across);
            }
        }
        Arrange::Space(_) => {
            for (along, across) in spread {
                bar(along, across);
            }
            for gap in [(0.2, 0.35), (0.65, 0.8)] {
                painter.line_segment([at(gap.0, 0.5), at(gap.1, 0.5)], faded);
                for end in [gap.0, gap.1] {
                    painter.line_segment([at(end, 0.4), at(end, 0.6)], faded);
                }
            }
        }
    }
}

/// A row of buttons; returns the one clicked.
fn arrange_row(
    ui: &mut egui::Ui,
    options: &[Arrange],
    enabled: impl Fn(Arrange) -> bool,
    language: Language,
) -> Option<Arrange> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for &arrange in options {
            ui.add_enabled_ui(enabled(arrange), |ui| {
                let response = icon_button(
                    ui,
                    ICON_BUTTON_SIZE,
                    false,
                    language.label(Label::Arrange(arrange)),
                    |painter, rect, stroke| paint_arrange_icon(painter, rect, stroke, arrange),
                );
                if response.clicked() {
                    clicked = Some(arrange);
                }
            });
        }
    });
    clicked
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

    ui.strong(language.label(Label::AlignObjects));
    let mut clicked = arrange_row(ui, &Arrange::ALIGN, enabled, language);
    ui.strong(language.label(Label::DistributeObjects));
    clicked = clicked.or(arrange_row(ui, &Arrange::DISTRIBUTE, enabled, language));
    ui.strong(language.label(Label::DistributeSpacing));
    clicked = clicked.or(arrange_row(ui, &Arrange::SPACE, enabled, language));

    ui.add_enabled_ui(options.reference == Reference::KeyObject, |ui| {
        ui.horizontal(|ui| {
            ui.label(language.label(Label::Spacing));
            ui.checkbox(
                &mut options.spacing_auto,
                language.label(Label::SpacingAuto),
            );
            let scale = units.from_millimeters(1.0);
            let mut value = options.spacing_mm * scale;
            let field = egui::DragValue::new(&mut value)
                .speed(0.1 * scale)
                .max_decimals(3)
                .range(0.0..=f64::MAX)
                .suffix(format!(" {}", language.unit_symbol(units)));
            if ui.add_enabled(!options.spacing_auto, field).changed() {
                options.spacing_mm = value / scale;
            }
        });
    });

    ui.separator();
    ui.horizontal(|ui| {
        ui.label(language.label(Label::AlignTo));
        egui::ComboBox::from_id_salt("align_to")
            .selected_text(language.label(options.reference.label()))
            .show_ui(ui, |ui| {
                for reference in Reference::ALL {
                    if reference == Reference::Parent && !children {
                        continue;
                    }
                    ui.selectable_value(
                        &mut options.reference,
                        reference,
                        language.label(reference.label()),
                    );
                }
            });
    });
    let hint = match options.reference {
        Reference::KeyObject if children => Some(Label::KeyShapeHint),
        Reference::KeyObject => Some(Label::KeyObjectHint),
        Reference::Parent if children => Some(Label::ParentHint),
        _ => None,
    };
    if let Some(hint) = hint {
        ui.label(egui::RichText::new(language.label(hint)).weak());
    }

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
