use bevy_egui::egui;
use leathercraft::history::{History, HistoryCommand};
use leathercraft::i18n::{Label, Language, Message};

use crate::ui_components::icons::glyph;
use crate::ui_components::list::{ListItem, list};

/// Between the panel's edges and the list, as in the design.
const PANEL_MARGIN: i8 = 10;

/// Lists the recorded actions: the current one is highlighted, and those after it, which can
/// still be redone, are dimmed and struck through.
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
        egui::Frame::new()
            .inner_margin(PANEL_MARGIN)
            .show(ui, |ui| {
                list(ui, |ui| {
                    for (index, action) in history.actions().enumerate() {
                        let undone = index > position;
                        let mut item = ListItem::new(language.message(action))
                            .icon_opt(action_glyph(action))
                            .selected(index == position)
                            .dimmed(undone)
                            .strikethrough(undone);
                        if index == position {
                            item = item.trailing(language.label(Label::HistoryCurrent));
                        } else if undone {
                            item = item.trailing(language.label(Label::HistoryUndone));
                        }
                        let response = ui.add(item);
                        if response.clicked() {
                            command = Some(HistoryCommand::GoTo(index));
                        }
                        if moved && index == position {
                            response.scroll_to_me(None);
                        }
                    }
                });
            });
    });
    command
}

/// The icon of a history entry, from the icon font.
fn action_glyph(action: &Message) -> Option<char> {
    let glyph = match action {
        Message::ActionNewProject | Message::ActionOpened { .. } => glyph::HISTORY,
        Message::ActionUnits
        | Message::ActionRenamePiece { .. }
        | Message::ActionMaterial { .. } => glyph::PROPERTIES,
        Message::ActionAddPiece { .. } => glyph::NEW_PIECE,
        Message::ActionAddChild { .. } => glyph::UNION,
        Message::ActionEditBase { .. } | Message::ActionEditChild { .. } => glyph::EDGE,
        Message::ActionMovePiece { .. }
        | Message::ActionMovePieces { .. }
        | Message::ActionMoveChildren { .. } => glyph::SELECT,
        Message::ActionAlignPieces { .. } | Message::ActionAlignChildren { .. } => {
            glyph::ALIGN_LEFT
        }
        Message::ActionDistributePieces { .. } | Message::ActionDistributeChildren { .. } => {
            glyph::DISTRIBUTE_LEFT
        }
        Message::ActionLockPiece { .. } => glyph::LOCK,
        Message::ActionUnlockPiece { .. } => glyph::UNLOCK,
        Message::ActionSetChildOp { .. } => glyph::SUBTRACT,
        Message::ActionReorderChild { .. } => glyph::LAYOUT,
        Message::ActionRemovePiece { .. }
        | Message::ActionRemovePieces { .. }
        | Message::ActionRemoveChild { .. }
        | Message::ActionRemoveChildren { .. } => glyph::TRASH,
        _ => return None,
    };
    Some(glyph)
}
