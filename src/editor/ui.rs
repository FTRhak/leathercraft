use super::{
    EditorState, PROJECT_EXTENSION, ViewMode, confirm_discard::confirm_discard, export,
    format_length, panel_viewer::PanelViewer, send,
};
use crate::document::{ActiveProject, HistoryMessage, ProjectMessage};
use crate::panels::dock::{PanelDock, PanelTab};
use crate::panels::material::{self, DialogResult};
use crate::panels::settings::{SettingsDraft, SettingsResult, settings_dialog};
use crate::panels::tools;
use crate::preview::PreviewSettings;
use crate::settings::{self, Action, Hotkeys, Preferences};
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
    mut preferences: ResMut<Preferences>,
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
                file_menu(ui, project, &preferences, &mut events, state);
            });
            ui.menu_button(language.label(Label::View), |_| {});
            ui.menu_button(language.label(Label::Edit), |ui| {
                edit_menu(
                    ui,
                    project,
                    history,
                    &preferences,
                    &mut history_events,
                    state,
                );
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

    // Disabled rather than hidden in preview mode, so that the layout doesn't jump.
    egui::TopBottomPanel::top("tool_bar").show(ctx, |ui| {
        ui.add_enabled_ui(*view_mode == ViewMode::Editor2d, |ui| {
            let settings = tools::ToolSettings {
                tool: &mut state.tool,
                polygon_sides: &mut state.polygon_sides,
                op_mode: &mut state.op_mode,
                line_style: &mut state.line_style,
                stitch: &mut state.stitch,
                piece_selected: state.selected.is_some(),
            };
            tools::show_toolbar(ui, settings, project.units, state.language);
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

    handle_hotkeys(
        ctx,
        project,
        &preferences,
        &mut events,
        &mut history_events,
        state,
        &mut view_mode,
    );

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

    if let Some(draft) = &mut state.settings_dialog {
        match settings_dialog(ctx, draft, state.language) {
            Some(SettingsResult::Save) => {
                if *preferences != draft.preferences {
                    *preferences = draft.preferences.clone();
                    settings::save_preferences(&preferences);
                }
                state.settings_dialog = None;
            }
            Some(SettingsResult::Cancel) => state.settings_dialog = None,
            None => {}
        }
    }
    Ok(())
}

/// A menu button showing the action's first shortcut.
fn action_button(
    ui: &mut egui::Ui,
    hotkeys: &Hotkeys,
    action: Action,
    language: Language,
    enabled: bool,
) -> bool {
    let mut button = egui::Button::new(language.label(action.label()));
    if let Some(shortcut) = hotkeys.first(action) {
        button = button.shortcut_text(ui.ctx().format_shortcut(shortcut));
    }
    ui.add_enabled(enabled, button).clicked()
}

fn file_menu(
    ui: &mut egui::Ui,
    project: &Project,
    preferences: &Preferences,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    for action in [Action::New, Action::Open, Action::Save, Action::SaveAs] {
        if action_button(ui, &preferences.hotkeys, action, language, true) {
            file_command(action, project, preferences, events, state);
        }
    }
    ui.separator();
    if ui.button(language.label(Label::ExportSvg)).clicked() {
        export(project, state, "SVG", "svg", exports::to_svg);
    }
    if ui.button(language.label(Label::ExportDxf)).clicked() {
        export(project, state, "DXF", "dxf", exports::to_dxf);
    }
}

/// The File menu commands, shared by the menu and their hotkeys.
fn file_command(
    action: Action,
    project: &Project,
    preferences: &Preferences,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    match action {
        Action::New => {
            if confirm_discard(project, language) {
                // A draft would otherwise still react to Enter behind the dialog.
                state.draft.clear();
                state.shape_draft = None;
                state.new_project_dialog = Some(preferences.default_material);
            }
        }
        Action::Open => {
            if confirm_discard(project, language)
                && let Some(path) = project_dialog(language).pick_file()
            {
                send(events, ProjectEvent::Open { path });
            }
        }
        Action::Save => match project.path.clone() {
            Some(path) => send(events, ProjectEvent::Save { path }),
            None => save_as(events, language),
        },
        Action::SaveAs => save_as(events, language),
        _ => {}
    }
}

fn edit_menu(
    ui: &mut egui::Ui,
    project: &Project,
    history: &History,
    preferences: &Preferences,
    events: &mut MessageWriter<HistoryMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    let hotkeys = &preferences.hotkeys;
    let items = [
        (Action::Undo, history.can_undo(), HistoryCommand::Undo),
        (Action::Redo, history.can_redo(), HistoryCommand::Redo),
    ];
    for (action, enabled, command) in items {
        if action_button(ui, hotkeys, action, language, enabled) {
            events.write(HistoryMessage(command));
        }
    }
    ui.separator();
    let any_pieces = !project.pieces.is_empty();
    if action_button(ui, hotkeys, Action::SelectAll, language, any_pieces) {
        state.select_all(project);
    }
    ui.separator();
    if ui.button(language.label(Label::Settings)).clicked() {
        // A draft would otherwise still react to Enter behind the window.
        state.draft.clear();
        state.shape_draft = None;
        state.settings_dialog = Some(SettingsDraft::new(preferences));
    }
}

/// Text fields keep their own keys (undo, select all, letters), so hotkeys only act outside
/// them, and neither while drawing nor while a dialog is open.
fn handle_hotkeys(
    ctx: &egui::Context,
    project: &Project,
    preferences: &Preferences,
    events: &mut MessageWriter<ProjectMessage>,
    history_events: &mut MessageWriter<HistoryMessage>,
    state: &mut EditorState,
    view_mode: &mut ViewMode,
) {
    let drafting = !state.draft.is_empty() || state.shape_draft.is_some();
    let dialog_open = state.new_project_dialog.is_some() || state.settings_dialog.is_some();
    if drafting || dialog_open || ctx.wants_keyboard_input() {
        return;
    }
    let Some(action) = preferences.hotkeys.triggered(ctx) else {
        return;
    };
    if let Some(tool) = action.tool() {
        if !tool.needs_piece() || state.selected.is_some() {
            state.tool = tool;
        }
        return;
    }
    match action {
        Action::New | Action::Open | Action::Save | Action::SaveAs => {
            file_command(action, project, preferences, events, state);
        }
        Action::Undo => {
            history_events.write(HistoryMessage(HistoryCommand::Undo));
        }
        Action::Redo => {
            history_events.write(HistoryMessage(HistoryCommand::Redo));
        }
        Action::SelectAll => select_all(project, state),
        Action::Delete => delete_selection(project, events, state),
        Action::ToggleView => {
            *view_mode = match *view_mode {
                ViewMode::Editor2d => ViewMode::Preview3d,
                ViewMode::Preview3d => ViewMode::Editor2d,
            };
        }
        Action::ToggleSnap => {
            state.snap.enabled = !state.snap.enabled;
            state.save_snap_when_settled(false);
        }
        _ => {}
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
fn delete_selection(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &EditorState,
) {
    if let Some(id) = state.selected
        && state.selected_children().len() > 1
    {
        let children = state.selected_children().to_vec();
        send(events, ProjectEvent::RemoveChildren { id, children });
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

fn select_all(project: &Project, state: &mut EditorState) {
    // Within a piece whose shape is selected, Ctrl+A selects its other shapes.
    if state.selected_children().is_empty() {
        state.select_all(project);
    } else {
        state.select_all_children(project);
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
