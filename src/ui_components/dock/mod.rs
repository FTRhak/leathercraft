mod style;

pub use style::{TabDecor, dock_style};

use crate::settings;
use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockState, NodeIndex, Surface, SurfaceIndex, Translations};
use leathercraft::i18n::{Label, Language};
use serde::{Deserialize, Serialize};

const REOPENED_PANEL_SIZE: egui::Vec2 = egui::vec2(280.0, 380.0);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelTab {
    Canvas,
    Properties,
    Layout,
    Material,
    History,
    Align,
    Snap,
}

impl PanelTab {
    /// The panels the Window menu lists; the canvas always stays open.
    pub const CLOSEABLE: [Self; 6] = [
        Self::Properties,
        Self::Align,
        Self::Snap,
        Self::Layout,
        Self::Material,
        Self::History,
    ];

    pub fn label(self) -> Label {
        match self {
            Self::Canvas => Label::Canvas,
            Self::Properties => Label::Properties,
            Self::Layout => Label::Layout,
            Self::Material => Label::Material,
            Self::History => Label::History,
            Self::Align => Label::Align,
            Self::Snap => Label::Snap,
        }
    }
}

/// The arrangement of the panels: docked, grouped into tabs, or floating.
#[derive(Resource)]
pub struct PanelDock {
    pub state: DockState<PanelTab>,
    /// What the tabs are painted with beyond the `egui_dock` style.
    pub decor: TabDecor,
    saved: Option<serde_json::Value>,
}

/// `DockState` doesn't record where its windows are, so their rects are kept beside it,
/// with one entry per surface.
#[derive(Serialize, Deserialize)]
struct SavedLayout {
    state: DockState<PanelTab>,
    window_rects: Vec<Option<egui::Rect>>,
}

impl PanelDock {
    /// The saved layout, else the default one if the saved one is missing or unusable.
    pub fn load(language: Language) -> Self {
        let saved = settings::load_dock();
        let layout = saved
            .clone()
            .and_then(|value| serde_json::from_value::<SavedLayout>(value).ok())
            .filter(|layout| is_usable(&layout.state));
        let mut state = match layout {
            Some(layout) => restore_windows(layout),
            None => default_state(),
        };
        state.translations = translations(language);
        Self {
            state,
            decor: TabDecor::default(),
            saved,
        }
    }

    pub fn is_open(&self, tab: PanelTab) -> bool {
        self.state.find_tab(&tab).is_some()
    }

    /// Closes an open panel; a closed one comes back floating at `center`, ready to be dragged.
    pub fn toggle(&mut self, tab: PanelTab, center: egui::Pos2) {
        if let Some(location) = self.state.find_tab(&tab) {
            self.state.remove_tab(location);
            return;
        }
        let surface = self.state.add_window(vec![tab]);
        if let Some(window) = self.state.get_window_state_mut(surface) {
            window
                .set_position(center - REOPENED_PANEL_SIZE * 0.5)
                .set_size(REOPENED_PANEL_SIZE);
        }
    }

    pub fn reset(&mut self, language: Language) {
        self.state = default_state();
        self.state.translations = translations(language);
    }

    pub fn set_language(&mut self, language: Language) {
        self.state.translations = translations(language);
    }

    /// Every layout change ends with a released pointer, so this needn't compare on other frames.
    pub fn save_on_release(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.pointer.any_released()) {
            return;
        }
        let window_rects = self
            .state
            .iter_surfaces()
            .enumerate()
            .map(|(index, surface)| match surface {
                Surface::Window(..) => ctx.memory(|memory| memory.area_rect(window_id(index))),
                _ => None,
            })
            .collect();
        let layout = SavedLayout {
            state: self.state.clone(),
            window_rects,
        };
        match serde_json::to_value(&layout) {
            Ok(value) if self.saved.as_ref() != Some(&value) => {
                settings::save_dock(&value);
                self.saved = Some(value);
            }
            Ok(_) => {}
            Err(error) => bevy::log::error!("Could not encode the panel layout: {error}"),
        }
    }
}

/// The id `egui_dock` gives the `egui::Window` of a surface.
fn window_id(index: usize) -> egui::Id {
    format!("window {:?}", SurfaceIndex(index)).into()
}

/// The canvas must stay on the main surface, which is what keeps that surface from emptying.
fn is_usable(state: &DockState<PanelTab>) -> bool {
    let mut tabs: Vec<PanelTab> = state.iter_all_tabs().map(|(_, tab)| *tab).collect();
    let count = tabs.len();
    tabs.sort_by_key(|tab| *tab as u8);
    tabs.dedup();
    tabs.len() == count && state.find_main_surface_tab(&PanelTab::Canvas).is_some()
}

fn restore_windows(mut layout: SavedLayout) -> DockState<PanelTab> {
    let margin = egui::Frame::window(&egui::Style::default())
        .total_margin()
        .sum();
    for (surface, rect) in layout.state.iter_surfaces_mut().zip(layout.window_rects) {
        if let (Surface::Window(_, window), Some(rect)) = (surface, rect) {
            window.set_position(rect.min).set_size(rect.size() - margin);
        }
    }
    layout.state
}

fn default_state() -> DockState<PanelTab> {
    let mut state = DockState::new(vec![PanelTab::Canvas]);
    let surface = state.main_surface_mut();
    let [canvas, right] = surface.split_right(
        NodeIndex::root(),
        0.78,
        vec![PanelTab::Layout, PanelTab::History],
    );
    surface.split_below(right, 0.35, vec![PanelTab::Material]);
    surface.split_left(
        canvas,
        0.2,
        vec![PanelTab::Properties, PanelTab::Align, PanelTab::Snap],
    );
    state
}

/// The window and leaf buttons are turned off in `editor_ui`, so only these strings are shown.
fn translations(language: Language) -> Translations {
    let mut translations = Translations::english();
    translations.tab_context_menu.close_button = language.label(Label::ClosePanel).to_owned();
    translations.tab_context_menu.eject_button = language.label(Label::FloatPanel).to_owned();
    translations
}
