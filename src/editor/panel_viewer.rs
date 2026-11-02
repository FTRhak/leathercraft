use super::{
    EditorState, ViewMode, canvas, selected_piece_details, send, units_control, view_controls,
};
use crate::document::{HistoryMessage, ProjectMessage};
use crate::panels::dock::PanelTab;
use crate::panels::{history, layout, material, tools};
use crate::preview::{self, PreviewSettings};
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
}

impl TabViewer for PanelViewer<'_, '_, '_, '_> {
    type Tab = PanelTab;

    fn title(&mut self, tab: &mut PanelTab) -> egui::WidgetText {
        let label = match (*tab, self.view_mode) {
            (PanelTab::Canvas, ViewMode::Preview3d) => Label::Preview3d,
            _ => tab.label(),
        };
        self.state.language.label(label).into()
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
                    tools::show_panel(
                        ui,
                        &mut self.state.tool,
                        &mut self.state.hole_mode,
                        language,
                    );
                });
            }
            PanelTab::Properties => {
                if editing {
                    view_controls(ui, self.state);
                } else {
                    preview::show_controls(ui, self.settings, language);
                }
                ui.separator();
                units_control(ui, self.project, self.events, language);
                ui.separator();
                selected_piece_details(ui, self.project, self.events, self.state);
            }
            PanelTab::Layout => {
                let event =
                    layout::show_panel(ui, self.project, &mut self.state.selected, language);
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
            PanelTab::Tools | PanelTab::Properties => [false, true],
            // The canvas pans itself, and these two have their own scroll areas.
            PanelTab::Canvas | PanelTab::Layout | PanelTab::Material | PanelTab::History => {
                [false, false]
            }
        }
    }
}
