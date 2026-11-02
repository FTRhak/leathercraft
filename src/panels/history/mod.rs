use bevy_egui::egui;
use leathercraft::history::{History, HistoryCommand};
use leathercraft::i18n::Language;

/// Lists the recorded actions; entries after the current one can still be redone and are dimmed.
pub fn show_panel(
    ui: &mut egui::Ui,
    history: &History,
    language: Language,
) -> Option<HistoryCommand> {
    let mut command = None;
    let position = history.position();
    // Scrolls to the current entry only when it changes, so the list can be browsed freely.
    let id = ui.id().with("history_position");
    let moved = ui.data_mut(|data| {
        let previous = data.get_temp::<usize>(id);
        data.insert_temp(id, position);
        previous != Some(position)
    });
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (index, action) in history.actions().enumerate() {
            let mut text = egui::RichText::new(language.message(action));
            if index > position {
                text = text.weak();
            }
            let row = egui::Button::selectable(index == position, text)
                .truncate()
                .min_size(egui::vec2(ui.available_width(), 0.0));
            let response = ui.add(row);
            if response.clicked() {
                command = Some(HistoryCommand::GoTo(index));
            }
            if moved && index == position {
                response.scroll_to_me(None);
            }
        }
    });
    command
}
