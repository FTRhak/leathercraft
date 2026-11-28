//! The Background images panel, from the "Background Images Panel" boards of the Brilliant
//! design: an Add image button, then a card per image with its header (caret, thumbnail, name,
//! show, lock and remove) and, while open, its Source, Display and Placement sections. A missing
//! file keeps its card, drawn as a warning. See `docs/background-image.md`.

use crate::background_textures::{BackgroundTextures, EXTENSIONS, Picture};
use crate::panels::properties::fields::{
    FIELD_WIDTH, ROW_GAP, angle_field, divider, note, param_row, section_header,
};
use crate::ui_components::button::Button;
use crate::ui_components::checkbox::Checkbox;
use crate::ui_components::group_button::GroupButton;
use crate::ui_components::icons::{
    glyph, icon_button, paint_close, paint_eye, paint_folder, paint_glyph, paint_image, paint_link,
    paint_reset, paint_warning,
};
use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use crate::ui_components::slider::Slider;
use bevy_egui::egui;
use leathercraft::background::{BackgroundImage, Depth, FrameMethod, MAX_SCALE, MIN_SCALE};
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{Point, Project, Units};

const PADDING: f32 = 12.0;
const CARD_GAP: f32 = 8.0;
const CARD_RADIUS: u8 = 6;
const CARD_PADDING: f32 = 8.0;
const HEADER_HEIGHT: f32 = 28.0;
const ICON_BUTTON: f32 = 22.0;
const THUMBNAIL: egui::Vec2 = egui::vec2(22.0, 16.0);
const NAME_SIZE: f32 = 12.0;
/// The smallest box side the size fields accept, in mm.
const MIN_SIDE_MM: f64 = 0.1;
/// The view a new image is fitted to when the canvas hasn't been shown yet: 300 mm square.
const FALLBACK_VIEW_MM: f64 = 300.0;

/// What the panel reads and keeps besides the project.
pub struct PanelState<'a> {
    pub selected: Option<u64>,
    pub textures: &'a mut BackgroundTextures,
    /// The world center of the canvas and its size in mm.
    pub view_area: Option<(Point, (f64, f64))>,
}

/// What the panel asks of the editor, besides edits.
pub enum Request {
    Select(u64),
    Calibrate(u64),
    Report(Message),
}

/// Shows the panel; returns an edit to send and a request for the editor.
pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    panel: PanelState,
    language: Language,
) -> (Option<ProjectEvent>, Option<Request>) {
    let mut event = None;
    let mut request = None;
    egui::Frame::new().inner_margin(PADDING).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = CARD_GAP;
        let add = Button::secondary(format!("+  {}", language.label(Label::AddBackgroundImage)));
        if ui
            .add_sized([ui.available_width(), HEADER_HEIGHT], add)
            .clicked()
        {
            (event, request) =
                add_image(ui.ctx(), project, panel.textures, panel.view_area, language);
        }
        if project.background_images.is_empty() {
            empty_hint(ui, language);
            return;
        }
        for image in &project.background_images {
            let selected = panel.selected == Some(image.id);
            let (edit, ask) = ui
                .push_id(("background_image", image.id), |ui| {
                    card(ui, image, selected, project.units, panel.textures, language)
                })
                .inner;
            event = event.take().or(edit);
            request = request.take().or(ask);
        }
    });
    (event, request)
}

/// Picks a file and adds it, fitted to the canvas view. A file that can't be decoded isn't
/// added; the status bar says why.
fn add_image(
    ctx: &egui::Context,
    project: &Project,
    textures: &mut BackgroundTextures,
    view_area: Option<(Point, (f64, f64))>,
    language: Language,
) -> (Option<ProjectEvent>, Option<Request>) {
    let Some(path) = image_dialog(language).pick_file() else {
        return (None, None);
    };
    let size_px = match textures.get(ctx, &path) {
        Picture::Loaded { size_px, .. } => *size_px,
        Picture::Failed(message) => {
            let message = message.clone();
            // Kept out of the cache, so that a fixed file can be tried again.
            textures.reload(&path);
            return (None, Some(Request::Report(message)));
        }
    };
    let (center, size) =
        view_area.unwrap_or((Point::default(), (FALLBACK_VIEW_MM, FALLBACK_VIEW_MM)));
    let image = BackgroundImage::new(project.next_background_id(), path, size_px, center, size);
    (Some(ProjectEvent::AddBackgroundImage { image }), None)
}

fn image_dialog(language: Language) -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter(language.label(Label::ImageFileType), &EXTENSIONS)
}

/// The dashed box with a hint, while there are no images.
fn empty_hint(ui: &mut egui::Ui, language: Language) {
    let palette = Palette::of(ui.painter());
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 110.0), egui::Sense::hover());
    let stroke = egui::Stroke::new(1.0_f32, palette.background.empty_border);
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    ui.painter()
        .extend(egui::Shape::dashed_line(&corners, stroke, 4.0, 3.0));
    let icon = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 32.0),
        egui::Vec2::splat(28.0),
    );
    paint_image(
        ui.painter(),
        icon,
        egui::Stroke::new(1.5_f32, palette.text.muted),
    );
    let text = egui::RichText::new(language.label(Label::BackgroundEmptyHint))
        .size(11.0)
        .color(palette.text.secondary);
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 16.0, icon.bottom() + 8.0),
        egui::pos2(rect.right() - 16.0, rect.bottom() - 8.0),
    );
    ui.put(
        text_rect,
        egui::Label::new(text).wrap().halign(egui::Align::Center),
    );
}

/// Whether an image's card is open; the first one shown starts open.
fn open_id(ui: &egui::Ui) -> egui::Id {
    ui.make_persistent_id("open")
}

/// An image's card: its header, and its sections while open.
fn card(
    ui: &mut egui::Ui,
    image: &BackgroundImage,
    selected: bool,
    units: Units,
    textures: &mut BackgroundTextures,
    language: Language,
) -> (Option<ProjectEvent>, Option<Request>) {
    let palette = Palette::of(ui.painter());
    let picture = textures.get(ui.ctx(), &image.path);
    let missing = matches!(picture, Picture::Failed(_));
    let border = if missing {
        palette.background.missing_border
    } else if selected {
        palette.background.frame
    } else {
        palette.background.entry_border
    };
    let fill = if missing {
        palette.status.warning_container
    } else {
        palette.background.entry_fill
    };
    let mut changed = image.clone();
    let mut request = None;
    let mut remove = false;
    let open_id = open_id(ui);
    let mut open = ui.data_mut(|data| *data.get_persisted_mut_or(open_id, true));

    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, border))
        .corner_radius(CARD_RADIUS)
        .inner_margin(egui::Margin::symmetric(CARD_PADDING as i8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let header = header(
                ui,
                &mut changed,
                textures.get(ui.ctx(), &image.path),
                open,
                language,
            );
            if header.toggle {
                open = !open;
                ui.data_mut(|data| data.insert_persisted(open_id, open));
            }
            remove = header.remove;
            if header.select {
                request = Some(Request::Select(image.id));
            }
            if !open {
                return;
            }
            ui.add_space(2.0);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 0.0, palette.background.entry_divider);
            ui.add_space(6.0);
            ui.spacing_mut().item_spacing.y = ROW_GAP;
            source_section(ui, &mut changed, textures, language);
            divider(ui);
            display_section(ui, &mut changed, language);
            divider(ui);
            if placement_section(ui, &mut changed, units, language) {
                request = Some(Request::Calibrate(image.id));
            }
            ui.add_space(4.0);
        });

    if remove {
        return (
            Some(ProjectEvent::RemoveBackgroundImage { id: image.id }),
            None,
        );
    }
    let event =
        (changed != *image).then_some(ProjectEvent::UpdateBackgroundImage { image: changed });
    (event, request)
}

/// What a click on the header did.
struct Header {
    toggle: bool,
    select: bool,
    remove: bool,
}

/// The caret, the thumbnail, the name, and the show, lock and remove buttons.
fn header(
    ui: &mut egui::Ui,
    image: &mut BackgroundImage,
    picture: &Picture,
    open: bool,
    language: Language,
) -> Header {
    let palette = Palette::of(ui.painter());
    let mut result = Header {
        toggle: false,
        select: false,
        remove: false,
    };
    let missing = matches!(picture, Picture::Failed(_));
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), HEADER_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let caret = icon_button(ui, 16.0, false, "", |painter, rect, stroke| {
                paint_caret(painter, rect, stroke.color, open);
            });
            result.toggle = caret.clicked();

            let (thumb, _) = ui.allocate_exact_size(THUMBNAIL, egui::Sense::hover());
            match picture {
                Picture::Loaded { texture, .. } => {
                    ui.painter().image(
                        texture.id(),
                        thumb,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    ui.painter().rect_stroke(
                        thumb,
                        2,
                        egui::Stroke::new(1.0_f32, palette.border.field),
                        egui::StrokeKind::Inside,
                    );
                }
                Picture::Failed(_) => paint_warning(
                    ui.painter(),
                    thumb.shrink(1.0),
                    egui::Stroke::new(1.3_f32, palette.status.warning),
                ),
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let remove = icon_button(
                    ui,
                    ICON_BUTTON,
                    false,
                    language.label(Label::RemoveImage),
                    paint_close,
                );
                result.remove = remove.clicked();
                if !missing {
                    let lock_name = if image.locked {
                        Label::UnlockImage
                    } else {
                        Label::LockImage
                    };
                    let lock = icon_button(
                        ui,
                        ICON_BUTTON,
                        image.locked,
                        language.label(lock_name),
                        |painter, rect, stroke| {
                            let glyph = if image.locked {
                                glyph::LOCK
                            } else {
                                glyph::UNLOCK
                            };
                            paint_glyph(painter, rect, stroke, glyph);
                        },
                    );
                    if lock.clicked() {
                        image.locked = !image.locked;
                    }
                    let eye_name = if image.visible {
                        Label::HideImage
                    } else {
                        Label::ShowImage
                    };
                    let visible = image.visible;
                    let eye = icon_button(
                        ui,
                        ICON_BUTTON,
                        false,
                        language.label(eye_name),
                        |painter, rect, stroke| {
                            let color = if visible {
                                palette.accent.base
                            } else {
                                stroke.color
                            };
                            paint_eye(painter, rect, egui::Stroke::new(1.2_f32, color), visible);
                        },
                    );
                    if eye.clicked() {
                        image.visible = !image.visible;
                    }
                }
                let mut name = image.name();
                let color = if missing {
                    name = format!("{name} — {}", language.label(Label::ImageNotFound));
                    palette.status.warning
                } else if image.visible {
                    palette.text.primary
                } else {
                    palette.text.muted
                };
                let text = egui::RichText::new(name).size(NAME_SIZE).color(color);
                let label = ui.add(
                    egui::Label::new(text)
                        .truncate()
                        .sense(egui::Sense::click()),
                );
                result.select = label.clicked();
                if label.double_clicked() {
                    result.toggle = true;
                }
            });
        },
    );
    result
}

/// A caret pointing right when closed, down when open.
fn paint_caret(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, open: bool) {
    let center = rect.center();
    let size = 3.5;
    let points = if open {
        vec![
            center + egui::vec2(-size, -size * 0.5),
            center + egui::vec2(size, -size * 0.5),
            center + egui::vec2(0.0, size * 0.7),
        ]
    } else {
        vec![
            center + egui::vec2(-size * 0.5, -size),
            center + egui::vec2(size * 0.7, 0.0),
            center + egui::vec2(-size * 0.5, size),
        ]
    };
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

/// The path with Browse and Reload, and the picture's size.
fn source_section(
    ui: &mut egui::Ui,
    image: &mut BackgroundImage,
    textures: &mut BackgroundTextures,
    language: Language,
) {
    let palette = Palette::of(ui.painter());
    section_header(ui, language.label(Label::ImageSource), None);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 26.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let reload = icon_button(
                ui,
                26.0,
                false,
                language.label(Label::ReloadImage),
                paint_reset,
            );
            if reload.clicked() {
                textures.reload(&image.path);
            }
            let browse = icon_button(
                ui,
                26.0,
                false,
                language.label(Label::BrowseImage),
                paint_folder,
            );
            if browse.clicked()
                && let Some(path) = image_dialog(language).pick_file()
            {
                textures.reload(&path);
                image.path = path;
            }
            let path = image.path.display().to_string();
            let text = egui::RichText::new(&path)
                .monospace()
                .size(11.0)
                .color(palette.text.strong);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(text).truncate())
                    .on_hover_text(path);
            });
        },
    );
    match textures.get(ui.ctx(), &image.path) {
        Picture::Loaded {
            size_px, kilobytes, ..
        } => {
            let info = language.message(&Message::ImageInfo {
                width: size_px[0],
                height: size_px[1],
                kilobytes: *kilobytes,
            });
            note(ui, &info);
        }
        Picture::Failed(message) => {
            let text = egui::RichText::new(language.message(message))
                .size(11.0)
                .color(palette.status.warning);
            ui.add(egui::Label::new(text).wrap());
        }
    }
}

/// Opacity, depth and frame method.
fn display_section(ui: &mut egui::Ui, image: &mut BackgroundImage, language: Language) {
    section_header(ui, language.label(Label::ImageDisplay), None);
    param_row(ui, language.label(Label::Opacity), None, |ui| {
        ui.add(
            Slider::new(&mut image.opacity, 0.0..=1.0)
                .percent()
                .width(140.0),
        );
    });
    param_row(ui, language.label(Label::ImageDepth), None, |ui| {
        let mut group = GroupButton::new(&mut image.depth);
        for depth in [Depth::Back, Depth::Front] {
            group = group.option(depth, language.label(Label::ImageDepthOption(depth)));
        }
        ui.add(group);
    });
    note(ui, language.label(Label::FrameMethodField));
    let mut group = GroupButton::new(&mut image.frame);
    for frame in FrameMethod::ALL {
        group = group.option(frame, language.label(Label::FrameMethod(frame)));
    }
    ui.add(group);
}

/// Size, offset, rotation, scale and flips. Returns whether Calibrate was clicked.
fn placement_section(
    ui: &mut egui::Ui,
    image: &mut BackgroundImage,
    units: Units,
    language: Language,
) -> bool {
    let mut calibrate = false;
    ui.horizontal(|ui| {
        section_header(ui, language.label(Label::ImagePlacement), None);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let button = ui
                .add_enabled(
                    !image.locked,
                    Button::outline(language.label(Label::Calibrate)),
                )
                .on_hover_text(language.label(Label::CalibrateHint));
            calibrate = button.clicked();
        });
    });

    let keep_id = ui.make_persistent_id("keep_aspect");
    let mut keep = ui.data_mut(|data| *data.get_persisted_mut_or(keep_id, true));
    let aspect = image.width / image.height;
    param_row(ui, language.label(Label::Width), None, |ui| {
        let before = image.width;
        length_input(ui, "width", &mut image.width, MIN_SIDE_MM, units, language);
        let link = icon_button(
            ui,
            20.0,
            keep,
            language.label(Label::KeepAspect),
            paint_link,
        );
        if link.clicked() {
            keep = !keep;
            ui.data_mut(|data| data.insert_persisted(keep_id, keep));
        }
        if keep && image.width != before {
            image.height = image.width / aspect;
        }
    });
    param_row(ui, language.label(Label::Height), None, |ui| {
        let before = image.height;
        length_input(
            ui,
            "height",
            &mut image.height,
            MIN_SIDE_MM,
            units,
            language,
        );
        ui.add_space(24.0);
        if keep && image.height != before {
            image.width = image.height * aspect;
        }
    });
    let offset = language.label(Label::Offset);
    param_row(ui, &format!("{offset} X"), None, |ui| {
        length_input(
            ui,
            "offset_x",
            &mut image.offset.x,
            f64::MIN,
            units,
            language,
        );
        ui.add_space(24.0);
    });
    param_row(ui, &format!("{offset} Y"), None, |ui| {
        length_input(
            ui,
            "offset_y",
            &mut image.offset.y,
            f64::MIN,
            units,
            language,
        );
        ui.add_space(24.0);
    });
    angle_field(ui, Label::Rotation, &mut image.rotation, language);
    param_row(ui, language.label(Label::Scale), None, |ui| {
        let field = NumberInput::new(&mut image.scale)
            .id_salt("scale")
            .range(MIN_SCALE..=MAX_SCALE)
            .step(0.05)
            .decimals(3)
            .unit("×")
            .width(FIELD_WIDTH);
        ui.add(field);
    });
    param_row(ui, language.label(Label::Flip), None, |ui| {
        ui.add(Checkbox::new(&mut image.flip_y, "Y"));
        ui.add(Checkbox::new(&mut image.flip_x, "X"));
    });
    calibrate
}

/// A length field in display units; writes back only a changed value, so that an untouched one
/// isn't rounded through the units.
fn length_input(
    ui: &mut egui::Ui,
    id_salt: &str,
    millimeters: &mut f64,
    min: f64,
    units: Units,
    language: Language,
) {
    let scale = units.from_millimeters(1.0);
    let current = *millimeters * scale;
    let mut value = current;
    let (step, decimals) = match units {
        Units::Millimeters => (1.0, 1),
        Units::Inches => (0.05, 3),
    };
    let field = NumberInput::new(&mut value)
        .id_salt(id_salt)
        .range(min * scale..=1_000_000.0 * scale)
        .step(step)
        .decimals(decimals)
        .unit(language.unit_symbol(units))
        .width(FIELD_WIDTH);
    ui.add(field);
    if value != current {
        *millimeters = value / scale;
    }
}
