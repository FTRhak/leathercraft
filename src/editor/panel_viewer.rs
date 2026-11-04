use super::{EditorState, ViewMode, canvas, send};
use crate::document::{HistoryMessage, ProjectMessage};
use crate::icons::{glyph, icon_text};
use crate::panels::dock::PanelTab;
use crate::panels::{align, history, layout, material, properties, tools};
use crate::preview::PreviewSettings;
use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::TabViewer;
use leathercraft::history::History;
use leathercraft::i18n::Label;
use leathercraft::model::Project;

/// Draws each dock tab from the editor's state.
pub(super) struct PanelViewer<'a, 'e, 'h, 's> {
    pub(super) project: &'a Project,
    pub(super) history: &'a History,
    pub(super) events: &'a mut MessageWriter<'e, ProjectMessage>,
    pub(super) history_events: &'a mut MessageWriter<'h, HistoryMessage>,
    pub(super) state: &'a mut EditorState,
    pub(super) view_mode: ViewMode,
    /// Stays behind `ResMut`, so that the camera only updates when the controls are shown.
    pub(super) settings: &'a mut ResMut<'s, PreviewSettings>,
    /// Sizes the icons in the tab titles.
    pub(super) style: std::sync::Arc<egui::Style>,
}

impl TabViewer for PanelViewer<'_, '_, '_, '_> {
    type Tab = PanelTab;

    fn title(&mut self, tab: &mut PanelTab) -> egui::WidgetText {
        let label = match (*tab, self.view_mode) {
            (PanelTab::Canvas, ViewMode::Preview3d) => Label::Preview3d,
            _ => tab.label(),
        };
        let text = self.state.language.label(label);
        match tab {
            // Tab titles are drawn in the button text style.
            PanelTab::History => {
                icon_text(&self.style, egui::TextStyle::Button, glyph::HISTORY, text)
            }
            _ => text.into(),
        }
    }

    /// Not derived from the title, which changes with the language and the view mode.
    fn id(&mut self, tab: &mut PanelTab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut PanelTab) {
        let language = self.state.language;
        let editing = self.view_mode == ViewMode::Editor2d;
        match *tab {
            PanelTab::Canvas => {
                if editing {
                    canvas(ui, self.project, self.events, self.state);
                }
            }
            // Disabled rather than removed in preview mode, so that the layout doesn't jump.
            PanelTab::Tools => {
                ui.add_enabled_ui(editing, |ui| {
                    let state = &mut *self.state;
                    tools::show_panel(
                        ui,
                        &mut state.tool,
                        &mut state.polygon_sides,
                        &mut state.op_mode,
                        language,
                    );
                });
            }
            PanelTab::Align => {
                ui.add_enabled_ui(editing, |ui| {
                    let state = &mut *self.state;
                    // With children selected, the panel arranges them within their piece.
                    let children = state.selected_children().to_vec();
                    let targets = match state.selected {
                        Some(piece) if !children.is_empty() => align::Targets::Children {
                            piece,
                            ids: &children,
                            key: state.selected_child_id(),
                        },
                        _ => align::Targets::Pieces {
                            ids: &state.pieces,
                            key: state.selected,
                        },
                    };
                    let event = align::show_panel(
                        ui,
                        self.project.units,
                        targets,
                        &mut state.align,
                        language,
                    );
                    if let Some(event) = event {
                        send(self.events, event);
                    }
                });
            }
            PanelTab::Properties => {
                let state = &mut *self.state;
                let piece = state
                    .selected
                    .and_then(|id| self.project.pieces.iter().find(|piece| piece.id == id))
                    .map(|piece| state.displayed(piece));
                let child = state.selected_child_id();
                let part = state.selected_part().map(|(_, part)| part);
                let view = if editing {
                    properties::ViewControls::Editor(properties::EditorView {
                        hint: state.tool.hint(),
                        snap_enabled: &mut state.snap_enabled,
                        snap_mm: &mut state.snap_mm,
                        fit_requested: &mut state.fit_requested,
                    })
                } else {
                    properties::ViewControls::Preview(self.settings)
                };
                let selection = properties::Selection {
                    piece: piece.as_deref(),
                    child,
                    part,
                    rename_buffer: &mut state.rename_buffer,
                };
                let event = properties::show_panel(ui, self.project, view, selection, language);
                if let Some(event) = event {
                    send(self.events, event);
                }
            }
            PanelTab::Layout => {
                let state = &mut *self.state;
                let selection = layout::Selection {
                    primary: state.selected,
                    pieces: &state.pieces,
                    children: state.selected_children(),
                };
                let (event, pick) = layout::show_panel(ui, self.project, selection, language);
                match pick {
                    Some(layout::Pick::Piece { id, toggle: true }) => state.toggle_piece(id),
                    Some(layout::Pick::Piece { id, toggle: false }) => state.select(Some(id), None),
                    Some(layout::Pick::Child {
                        id,
                        child,
                        toggle: true,
                    }) => state.toggle_child(id, child),
                    Some(layout::Pick::Child {
                        id,
                        child,
                        toggle: false,
                    }) => state.select(Some(id), Some(child)),
                    None => {}
                }
                if let Some(event) = event {
                    send(self.events, event);
                }
            }
            PanelTab::Material => {
                let event = material::show_panel(ui, self.project, self.state.selected, language);
                if let Some(event) = event {
                    send(self.events, event);
                }
            }
            PanelTab::History => {
                if let Some(command) = history::show_panel(ui, self.history, language) {
                    self.history_events.write(HistoryMessage(command));
                }
            }
        }
    }

    fn is_closeable(&self, tab: &PanelTab) -> bool {
        *tab != PanelTab::Canvas
    }

    fn allowed_in_windows(&self, tab: &mut PanelTab) -> bool {
        *tab != PanelTab::Canvas
    }

    /// In preview mode the 3D scene shows through the canvas tab.
    fn clear_background(&self, tab: &PanelTab) -> bool {
        *tab != PanelTab::Canvas || self.view_mode == ViewMode::Editor2d
    }

    fn scroll_bars(&self, tab: &PanelTab) -> [bool; 2] {
        match tab {
            PanelTab::Tools | PanelTab::Properties | PanelTab::Align => [false, true],
            // The canvas pans itself, and these two have their own scroll areas.
            PanelTab::Canvas | PanelTab::Layout | PanelTab::Material | PanelTab::History => {
                [false, false]
            }
        }
    }
}
