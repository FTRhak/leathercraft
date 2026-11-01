use leathercraft::i18n::{Label, Language};
use leathercraft::model::Project;

pub(super) fn confirm_discard(project: &Project, language: Language) -> bool {
    if !project.modified {
        return true;
    }
    rfd::MessageDialog::new()
        .set_title(language.label(Label::UnsavedChanges))
        .set_description(language.label(Label::DiscardChanges))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}
