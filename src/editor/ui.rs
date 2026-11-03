use super::{
    EditorState, PROJECT_EXTENSION, ViewMode, confirm_discard::confirm_discard, export,
    format_length, panel_viewer::PanelViewer, send,
};
use crate::document::{ActiveProject, HistoryMessage, ProjectMessage};
use crate::panels::dock::{PanelDock, PanelTab};
use crate::panels::material::{self, DialogResult};
use crate::preview::PreviewSettings;
use crate::settings;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_dock::DockArea;
use leathercraft::events::ProjectEvent;
use leathercraft::exports;
use leathercraft::history::{History, HistoryCommand};
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Project;

// A Bevy system takes its resources as parameters.
#[allow(clippy::too_many_arguments)]
pub fn editor_ui(
    mut contexts: EguiContexts,
    project: Res<ActiveProject>,
    mut events: MessageWriter<ProjectMessage>,
    mut history_events: MessageWriter<HistoryMessage>,
    mut state: ResMut<EditorState>,
    mut view_mode: ResMut<ViewMode>,
    mut settings: ResMut<PreviewSettings>,
    mut dock: ResMut<PanelDock>,
) -> Result {
    let ctx: &egui::Context = contexts.ctx_mut()?;
    let history = project.history();
    let project: &Project = &project;
    let state = &mut *state;
    let dock = &mut *dock;

    egui::TopBottomPanel::top("file_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let language = state.language;
            ui.menu_button(language.label(Label::File), |ui| {
                file_menu(ui, project, &mut events, state);
            });
            ui.menu_button(language.label(Label::View), |_| {});
            ui.menu_button(language.label(Label::Edit), |ui| {
                edit_menu(ui, project, history, &mut history_events, state);
            });
            // Stays open on a click, so that the toggled state can be seen.
            egui::containers::menu::MenuButton::new(language.label(Label::Window))
                .config(
                    egui::containers::menu::MenuConfig::new()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| window_menu(ui, state, dock));
            ui.separator();
            ui.selectable_value(
                &mut *view_mode,
                ViewMode::Editor2d,
                language.label(Label::Editor2d),
            );
            ui.selectable_value(
                &mut *view_mode,
                ViewMode::Preview3d,
                language.label(Label::Preview3d),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                language_control(ui, state, dock);
            });
        });
    });

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let language = state.language;
            let status = language.message(&state.status);
            if state.status_is_error {
                ui.colored_label(ui.visuals().error_fg_color, status);
            } else {
                ui.label(status);
            }
            if let Some(cursor) = state.cursor {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{}, {}",
                        format_length(language, project.units, cursor.x),
                        format_length(language, project.units, cursor.y)
                    ));
                });
            }
        });
    });

    handle_delete_key(ctx, project, &mut events, state);
    handle_select_all_key(ctx, project, state);
    handle_history_keys(ctx, &mut history_events, state);

    // The canvas sets this again if its tab is showing.
    state.cursor = None;
    let bounds = ctx.available_rect();
    // The central panel `show` adds is transparent, so the 3D scene shows through in preview mode.
    DockArea::new(&mut dock.state)
        .window_bounds(bounds)
        .show_leaf_close_all_buttons(false)
        .show_leaf_collapse_buttons(false)
        .show(
            ctx,
            &mut PanelViewer {
                project,
                history,
                events: &mut events,
                history_events: &mut history_events,
                state,
                view_mode: *view_mode,
                settings: &mut settings,
                style: ctx.style(),
            },
        );
    dock.save_on_release(ctx);

    if let Some(material) = &mut state.new_project_dialog {
        match material::new_project_dialog(ctx, material, state.language) {
            Some(DialogResult::Create) => {
                let material = *material;
                state.new_project_dialog = None;
                send(&mut events, ProjectEvent::New { material });
            }
            Some(DialogResult::Cancel) => state.new_project_dialog = None,
            None => {}
        }
    }
    Ok(())
}

fn file_menu(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    if ui.button(language.label(Label::New)).clicked() && confirm_discard(project, language) {
        // A draft would otherwise still react to Enter behind the dialog.
        state.draft.clear();
        state.shape_draft = None;
        state.new_project_dialog = Some(project.default_material);
    }
    if ui.button(language.label(Label::Open)).clicked()
        && confirm_discard(project, language)
        && let Some(path) = project_dialog(language).pick_file()
    {
        send(events, ProjectEvent::Open { path });
    }
    if ui.button(language.label(Label::Save)).clicked() {
        match project.path.clone() {
            Some(path) => send(events, ProjectEvent::Save { path }),
            None => save_as(events, language),
        }
    }
    if ui.button(language.label(Label::SaveAs)).clicked() {
        save_as(events, language);
    }
    ui.separator();
    if ui.button(language.label(Label::ExportSvg)).clicked() {
        export(project, state, "SVG", "svg", exports::to_svg);
    }
    if ui.button(language.label(Label::ExportDxf)).clicked() {
        export(project, state, "DXF", "dxf", exports::to_dxf);
    }
}

const UNDO: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
const REDO: egui::KeyboardShortcut = egui::KeyboardShortcut::new(
    egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
    egui::Key::Z,
);
const REDO_ALT: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Y);
const SELECT_ALL: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::A);

fn edit_menu(
    ui: &mut egui::Ui,
    project: &Project,
    history: &History,
    events: &mut MessageWriter<HistoryMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    let items = [
        (Label::Undo, UNDO, history.can_undo(), HistoryCommand::Undo),
        (Label::Redo, REDO, history.can_redo(), HistoryCommand::Redo),
    ];
    for (label, shortcut, enabled, command) in items {
        let button = egui::Button::new(language.label(label))
            .shortcut_text(ui.ctx().format_shortcut(&shortcut));
        if ui.add_enabled(enabled, button).clicked() {
            events.write(HistoryMessage(command));
        }
    }
    ui.separator();
    let button = egui::Button::new(language.label(Label::SelectAll))
        .shortcut_text(ui.ctx().format_shortcut(&SELECT_ALL));
    if ui.add_enabled(!project.pieces.is_empty(), button).clicked() {
        state.select_all(project);
    }
}

/// Text fields keep their own undo, so the shortcuts only act outside them.
fn handle_history_keys(
    ctx: &egui::Context,
    events: &mut MessageWriter<HistoryMessage>,
    state: &EditorState,
) {
    let drafting = !state.draft.is_empty() || state.shape_draft.is_some();
    if drafting || state.new_project_dialog.is_some() || ctx.wants_keyboard_input() {
        return;
    }
    // Redo first: matching ignores an extra Shift, so Ctrl+Z would also take Ctrl+Shift+Z.
    let command = ctx.input_mut(|input| {
        if input.consume_shortcut(&REDO) || input.consume_shortcut(&REDO_ALT) {
            Some(HistoryCommand::Redo)
        } else if input.consume_shortcut(&UNDO) {
            Some(HistoryCommand::Undo)
        } else {
            None
        }
    });
    if let Some(command) = command {
        events.write(HistoryMessage(command));
    }
}

fn window_menu(ui: &mut egui::Ui, state: &EditorState, dock: &mut PanelDock) {
    let language = state.language;
    for tab in PanelTab::CLOSEABLE {
        let mut open = dock.is_open(tab);
        if ui
            .toggle_value(&mut open, language.label(tab.label()))
            .changed()
        {
            dock.toggle(tab, ui.ctx().content_rect().center());
        }
    }
    ui.separator();
    if ui.button(language.label(Label::ResetPanels)).clicked() {
        dock.reset(language);
    }
}

/// With several pieces selected, the locked ones stay.
fn handle_delete_key(
    ctx: &egui::Context,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &EditorState,
) {
    let drafting = !state.draft.is_empty() || state.shape_draft.is_some();
    if drafting || state.new_project_dialog.is_some() || ctx.wants_keyboard_input() {
        return;
    }
    let delete_pressed = ctx.input(|input| {
        input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
    });
    if !delete_pressed {
        return;
    }
    if state.pieces.len() > 1 {
        let mut ids: Vec<u64> = project
            .pieces
            .iter()
            .filter(|piece| !piece.locked && state.is_selected(piece.id))
            .map(|piece| piece.id)
            .collect();
        let event = match ids.len() {
            0 => return,
            1 => ProjectEvent::RemovePiece { id: ids.remove(0) },
            _ => ProjectEvent::RemovePieces { ids },
        };
        send(events, event);
        return;
    }
    if let Some(id) = state.selected {
        let event = match state.selected_child_id() {
            Some(child) => ProjectEvent::RemoveChild { id, child },
            None => ProjectEvent::RemovePiece { id },
        };
        send(events, event);
    }
}

fn handle_select_all_key(ctx: &egui::Context, project: &Project, state: &mut EditorState) {
    let drafting = !state.draft.is_empty() || state.shape_draft.is_some();
    if drafting || state.new_project_dialog.is_some() || ctx.wants_keyboard_input() {
        return;
    }
    if ctx.input_mut(|input| input.consume_shortcut(&SELECT_ALL)) {
        state.select_all(project);
    }
}

fn language_control(ui: &mut egui::Ui, state: &mut EditorState, dock: &mut PanelDock) {
    let mut language = state.language;
    egui::ComboBox::from_label(language.label(Label::Language))
        .selected_text(language.native_name())
        .show_ui(ui, |ui| {
            for option in Language::ALL {
                ui.selectable_value(&mut language, option, option.native_name());
            }
        });
    if language != state.language {
        state.language = language;
        settings::save_language(language);
        dock.set_language(language);
    }
}

fn project_dialog(language: Language) -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter(language.label(Label::ProjectFileType), &[PROJECT_EXTENSION])
}

fn save_as(events: &mut MessageWriter<ProjectMessage>, language: Language) {
    if let Some(mut path) = project_dialog(language).save_file() {
        if path.extension().is_none() {
            path.set_extension(PROJECT_EXTENSION);
        }
        send(events, ProjectEvent::Save { path });
    }
}
