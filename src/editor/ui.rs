use super::{
    EditorState, PROJECT_EXTENSION, ViewMode, canvas, confirm_discard::confirm_discard, export,
    format_length, selected_piece_details, send, units_control, view_controls,
};
use crate::document::{ActiveProject, ProjectMessage};
use crate::panels::layout;
use crate::panels::material::{self, DialogResult};
use crate::panels::tools;
use crate::preview::{self, PreviewSettings};
use crate::settings;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use leathercraft::events::ProjectEvent;
use leathercraft::exports;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Project;

pub fn editor_ui(
    mut contexts: EguiContexts,
    project: Res<ActiveProject>,
    mut events: MessageWriter<ProjectMessage>,
    mut state: ResMut<EditorState>,
    mut view_mode: ResMut<ViewMode>,
    mut settings: ResMut<PreviewSettings>,
) -> Result {
    let ctx: &egui::Context = contexts.ctx_mut()?;
    let project: &Project = &project;
    let state = &mut *state;

    egui::TopBottomPanel::top("file_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let language = state.language;
            ui.menu_button(language.label(Label::File), |ui| {
                file_menu(ui, project, &mut events, state);
            });
            ui.menu_button(language.label(Label::View), |_| {});
            ui.menu_button(language.label(Label::Edit), |_| {});
            // Stays open on a click, so that the toggled state can be seen.
            egui::containers::menu::MenuButton::new(language.label(Label::Window))
                .config(
                    egui::containers::menu::MenuConfig::new()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| window_menu(ui, state));
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
                language_control(ui, state);
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

    if *view_mode == ViewMode::Editor2d && state.panels.tools {
        egui::SidePanel::left("tools")
            .default_width(110.0)
            .show(ctx, |ui| {
                tools::show_panel(ui, &mut state.tool, state.language);
            });
    }

    egui::SidePanel::left("sidebar")
        .default_width(220.0)
        .show(ctx, |ui| {
            if *view_mode == ViewMode::Editor2d {
                view_controls(ui, state);
                ui.separator();
            } else {
                preview::show_controls(ui, &mut settings, state.language);
                ui.separator();
            }
            units_control(ui, project, &mut events, state.language);
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                selected_piece_details(ui, project, &mut events, state);
            });
        });

    if state.panels.layout {
        egui::SidePanel::right("layout")
            .default_width(200.0)
            .show(ctx, |ui| {
                let event = layout::show_panel(ui, project, &mut state.selected, state.language);
                if let Some(event) = event {
                    send(&mut events, event);
                }
            });
    }

    if state.panels.material {
        egui::SidePanel::right("material")
            .default_width(300.0)
            .show(ctx, |ui| {
                let event = material::show_panel(ui, project, state.selected, state.language);
                if let Some(event) = event {
                    send(&mut events, event);
                }
            });
    }

    handle_delete_key(ctx, &mut events, state);

    // In preview mode no central panel is drawn, so the 3D scene shows through.
    if *view_mode == ViewMode::Editor2d {
        egui::CentralPanel::default().show(ctx, |ui| canvas(ui, project, &mut events, state));
    } else {
        state.cursor = None;
    }

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

fn window_menu(ui: &mut egui::Ui, state: &mut EditorState) {
    let language = state.language;
    let mut panels = state.panels;
    ui.toggle_value(&mut panels.tools, language.label(Label::Tools));
    ui.toggle_value(&mut panels.layout, language.label(Label::Layout));
    ui.toggle_value(&mut panels.material, language.label(Label::Material));
    if panels != state.panels {
        state.panels = panels;
        settings::save_panels(panels);
    }
}

fn handle_delete_key(
    ctx: &egui::Context,
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
    if delete_pressed && let Some(id) = state.selected {
        send(events, ProjectEvent::RemovePiece { id });
    }
}

fn language_control(ui: &mut egui::Ui, state: &mut EditorState) {
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
