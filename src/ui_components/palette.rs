//! The canvas colors that depend on the theme.

use bevy_egui::egui;

/// The canvas colors that depend on the theme, so that they stay readable on its background.
pub(crate) struct Palette {
    pub(crate) grid_line: egui::Color32,
    pub(crate) grid_axis: egui::Color32,
    pub(crate) snap_dot: egui::Color32,
    pub(crate) outline: egui::Color32,
    pub(crate) label: egui::Color32,
    pub(crate) hole: egui::Color32,
    pub(crate) centerline: egui::Color32,
    pub(crate) edge: egui::Color32,
}

impl Palette {
    const DARK: Self = Self {
        grid_line: egui::Color32::from_gray(60),
        grid_axis: egui::Color32::from_gray(110),
        snap_dot: egui::Color32::from_gray(95),
        outline: egui::Color32::from_rgb(190, 190, 200),
        label: egui::Color32::from_gray(220),
        hole: egui::Color32::from_gray(205),
        centerline: egui::Color32::from_rgb(120, 195, 215),
        edge: egui::Color32::from_rgb(250, 215, 90),
    };
    const LIGHT: Self = Self {
        grid_line: egui::Color32::from_gray(222),
        grid_axis: egui::Color32::from_gray(165),
        snap_dot: egui::Color32::from_gray(170),
        outline: egui::Color32::from_rgb(70, 70, 82),
        label: egui::Color32::from_gray(40),
        hole: egui::Color32::from_gray(60),
        centerline: egui::Color32::from_rgb(30, 130, 165),
        edge: egui::Color32::from_rgb(200, 145, 0),
    };

    pub(crate) fn of(painter: &egui::Painter) -> &'static Self {
        if painter.ctx().style().visuals.dark_mode {
            &Self::DARK
        } else {
            &Self::LIGHT
        }
    }
}
