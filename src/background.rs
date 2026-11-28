//! Background images: photos or scans shown under (or over) the pattern to trace from.
//!
//! They are stored in the project but never take part in the pattern: exports and the 3D
//! preview ignore them. Only the placement lives here; decoding and drawing are in the binary.
//! See `docs/background-image.md`.

use crate::i18n::Message;
use crate::model::{MAX_COORDINATE_MM, Point, Project};
use crate::shapes::{to_local, to_world};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The opacity of a newly added image.
pub const DEFAULT_OPACITY: f32 = 0.5;
/// The share of the view a newly added image fills.
const FIT_SHARE: f64 = 0.8;
pub const MIN_SCALE: f64 = 0.01;
pub const MAX_SCALE: f64 = 100.0;

/// Whether the image is drawn under the grid and pieces, or over them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Depth {
    #[default]
    Back,
    Front,
}

/// How the picture fills its box when their aspect ratios differ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameMethod {
    /// Distorts the picture to fill the box.
    Stretch,
    /// Shows the whole picture inside the box.
    #[default]
    Fit,
    /// Fills the box and cuts off what overflows it.
    Crop,
}

impl FrameMethod {
    pub const ALL: [Self; 3] = [Self::Stretch, Self::Fit, Self::Crop];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// Unique within the project.
    pub id: u64,
    /// Absolute in memory; saved relative to the project file when it is under its folder.
    pub path: PathBuf,
    #[serde(default = "yes")]
    pub visible: bool,
    /// A locked image can't be selected or moved on the canvas, only edited in the panel.
    #[serde(default)]
    pub locked: bool,
    /// 0.0..=1.0.
    pub opacity: f32,
    #[serde(default)]
    pub depth: Depth,
    #[serde(default)]
    pub frame: FrameMethod,
    /// The box the picture is framed in, in mm, before `scale`.
    pub width: f64,
    pub height: f64,
    /// The center of the box, in mm.
    pub offset: Point,
    /// Radians, around `offset`.
    #[serde(default)]
    pub rotation: f64,
    #[serde(default = "one")]
    pub scale: f64,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
}

fn yes() -> bool {
    true
}

fn one() -> f64 {
    1.0
}

/// A textured quad: corners in mm and their texture coordinates, in the order
/// top-left, top-right, bottom-right, bottom-left of the unrotated box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub corners: [Point; 4],
    pub uvs: [(f64, f64); 4],
}

/// A rectangle within 0..1 on both axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitRect {
    pub min: (f64, f64),
    pub max: (f64, f64),
}

impl UnitRect {
    pub const FULL: Self = Self {
        min: (0.0, 0.0),
        max: (1.0, 1.0),
    };

    fn centered(width: f64, height: f64) -> Self {
        Self {
            min: ((1.0 - width) * 0.5, (1.0 - height) * 0.5),
            max: ((1.0 + width) * 0.5, (1.0 + height) * 0.5),
        }
    }
}

/// Which part of the picture is shown (`uv`) and which part of the box it covers (`area`),
/// for a picture of `picture_px` framed in a box of `box_mm`.
pub fn frame_rects(
    frame: FrameMethod,
    picture_px: [u32; 2],
    box_mm: (f64, f64),
) -> (UnitRect, UnitRect) {
    let picture = f64::from(picture_px[0].max(1)) / f64::from(picture_px[1].max(1));
    let boxed = box_mm.0 / box_mm.1;
    if !boxed.is_finite() || boxed <= 0.0 {
        return (UnitRect::FULL, UnitRect::FULL);
    }
    match frame {
        FrameMethod::Stretch => (UnitRect::FULL, UnitRect::FULL),
        FrameMethod::Fit if picture > boxed => {
            (UnitRect::FULL, UnitRect::centered(1.0, boxed / picture))
        }
        FrameMethod::Fit => (UnitRect::FULL, UnitRect::centered(picture / boxed, 1.0)),
        FrameMethod::Crop if picture > boxed => {
            (UnitRect::centered(boxed / picture, 1.0), UnitRect::FULL)
        }
        FrameMethod::Crop => (UnitRect::centered(1.0, picture / boxed), UnitRect::FULL),
    }
}

impl BackgroundImage {
    /// A new image fitted inside most of the view at the picture's aspect ratio, centered on it.
    pub fn new(
        id: u64,
        path: PathBuf,
        picture_px: [u32; 2],
        view_center: Point,
        view_mm: (f64, f64),
    ) -> Self {
        let aspect = f64::from(picture_px[0].max(1)) / f64::from(picture_px[1].max(1));
        let (room_w, room_h) = (
            (view_mm.0 * FIT_SHARE).max(1.0),
            (view_mm.1 * FIT_SHARE).max(1.0),
        );
        let (width, height) = if room_w / room_h > aspect {
            (room_h * aspect, room_h)
        } else {
            (room_w, room_w / aspect)
        };
        Self {
            id,
            path,
            visible: true,
            locked: false,
            opacity: DEFAULT_OPACITY,
            depth: Depth::Back,
            frame: FrameMethod::Fit,
            width,
            height,
            offset: view_center,
            rotation: 0.0,
            scale: 1.0,
            flip_x: false,
            flip_y: false,
        }
    }

    /// The file name, for the panel and the history.
    pub fn name(&self) -> String {
        self.path.file_name().map_or_else(
            || self.path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// The box as drawn: its size times `scale`.
    pub fn drawn_size(&self) -> (f64, f64) {
        (self.width * self.scale, self.height * self.scale)
    }

    /// Turns a point of the box, in mm from its center before rotation, into the world.
    pub fn to_world(&self, x: f64, y: f64) -> Point {
        to_world(self.offset, self.rotation, x, y)
    }

    /// Turns a world point into the box's own axes, in mm from its center.
    pub fn to_local(&self, point: Point) -> (f64, f64) {
        to_local(self.offset, self.rotation, point)
    }

    /// The corners of the drawn box: top-left, top-right, bottom-right, bottom-left.
    pub fn corners(&self) -> [Point; 4] {
        let (w, h) = self.drawn_size();
        let (x, y) = (w * 0.5, h * 0.5);
        [
            self.to_world(-x, -y),
            self.to_world(x, -y),
            self.to_world(x, y),
            self.to_world(-x, y),
        ]
    }

    /// Whether a world point lies inside the drawn box.
    pub fn contains(&self, point: Point) -> bool {
        let (w, h) = self.drawn_size();
        let (x, y) = self.to_local(point);
        x.abs() <= w * 0.5 && y.abs() <= h * 0.5
    }

    /// The quad to draw for a picture of `picture_px`, after the frame method and the flips.
    pub fn quad(&self, picture_px: [u32; 2]) -> Quad {
        let (w, h) = self.drawn_size();
        let (uv, area) = frame_rects(self.frame, picture_px, (w, h));
        let local = |(ax, ay): (f64, f64)| self.to_world((ax - 0.5) * w, (ay - 0.5) * h);
        let flip = |(u, v): (f64, f64)| {
            (
                if self.flip_x { 1.0 - u } else { u },
                if self.flip_y { 1.0 - v } else { v },
            )
        };
        let (a0, a1) = (area.min, area.max);
        let (u0, u1) = (uv.min, uv.max);
        Quad {
            corners: [
                local((a0.0, a0.1)),
                local((a1.0, a0.1)),
                local((a1.0, a1.1)),
                local((a0.0, a1.1)),
            ],
            uvs: [
                flip((u0.0, u0.1)),
                flip((u1.0, u0.1)),
                flip((u1.0, u1.1)),
                flip((u0.0, u1.1)),
            ],
        }
    }

    /// Scales the box and its offset about `center` by `factor`, as calibration does.
    pub fn scaled_about(&self, center: Point, factor: f64) -> Self {
        Self {
            width: self.width * factor,
            height: self.height * factor,
            offset: Point::new(
                center.x + (self.offset.x - center.x) * factor,
                center.y + (self.offset.y - center.y) * factor,
            ),
            ..self.clone()
        }
    }

    fn is_valid(&self) -> bool {
        let (w, h) = self.drawn_size();
        let positive = |value: f64| value.is_finite() && value > 0.0;
        let within = |value: f64| value.is_finite() && value.abs() <= MAX_COORDINATE_MM;
        self.opacity.is_finite()
            && (0.0..=1.0).contains(&self.opacity)
            && positive(self.width)
            && positive(self.height)
            && positive(self.scale)
            && within(w)
            && within(h)
            && within(self.offset.x)
            && within(self.offset.y)
            && self.rotation.is_finite()
    }
}

/// The path to save: relative to the project file's folder when the image is inside it.
pub fn stored_path(image: &Path, project_file: Option<&Path>) -> PathBuf {
    project_file
        .and_then(Path::parent)
        .filter(|folder| !folder.as_os_str().is_empty())
        .and_then(|folder| image.strip_prefix(folder).ok())
        .map_or_else(|| image.to_path_buf(), Path::to_path_buf)
}

/// The path to load from: a relative path is taken from the project file's folder.
pub fn resolve_path(stored: &Path, project_file: Option<&Path>) -> PathBuf {
    if stored.is_absolute() {
        return stored.to_path_buf();
    }
    project_file
        .and_then(Path::parent)
        .map_or_else(|| stored.to_path_buf(), |folder| folder.join(stored))
}

impl Project {
    pub fn background_image(&self, id: u64) -> Option<&BackgroundImage> {
        self.background_images.iter().find(|image| image.id == id)
    }

    pub fn next_background_id(&self) -> u64 {
        self.background_images
            .iter()
            .map(|image| image.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    /// Checks each image's placement. Whether its file exists is not checked: a moved photo
    /// leaves a valid project, which the editor shows with a warning.
    pub(crate) fn validate_background_images(&self) -> Result<(), Message> {
        let mut ids = HashSet::with_capacity(self.background_images.len());
        for image in &self.background_images {
            if image.id == 0 || !ids.insert(image.id) {
                return Err(Message::InvalidBackgroundImageId { name: image.name() });
            }
            if !image.is_valid() {
                return Err(Message::InvalidBackgroundImage { name: image.name() });
            }
        }
        Ok(())
    }

    /// Rewrites each image path for saving next to `project_file`.
    pub(crate) fn with_stored_image_paths(&self, project_file: &Path) -> Project {
        let mut stored = self.clone();
        for image in &mut stored.background_images {
            image.path = stored_path(&image.path, Some(project_file));
        }
        stored
    }

    /// Resolves the image paths of a project loaded from `project_file`.
    pub(crate) fn resolve_image_paths(&mut self, project_file: &Path) {
        for image in &mut self.background_images {
            image.path = resolve_path(&image.path, Some(project_file));
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    pub(crate) fn image(id: u64) -> BackgroundImage {
        BackgroundImage::new(
            id,
            PathBuf::from(format!("/photos/skin_{id}.jpg")),
            [300, 400],
            Point::new(0.0, 0.0),
            (375.0, 500.0),
        )
    }

    fn close(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9
    }

    #[test]
    fn a_new_image_fits_most_of_the_view_at_its_aspect() {
        let image = image(1);
        assert!((image.width - 300.0).abs() < 1e-9);
        assert!((image.height - 400.0).abs() < 1e-9);
        let wide = BackgroundImage::new(
            1,
            PathBuf::new(),
            [400, 100],
            Point::default(),
            (100.0, 100.0),
        );
        assert!((wide.width - 80.0).abs() < 1e-9);
        assert!((wide.height - 20.0).abs() < 1e-9);
    }

    #[test]
    fn frame_methods_place_wide_and_tall_pictures() {
        let wide = [200, 100];
        let square = (100.0, 100.0);
        assert_eq!(
            frame_rects(FrameMethod::Stretch, wide, square),
            (UnitRect::FULL, UnitRect::FULL)
        );
        let (uv, area) = frame_rects(FrameMethod::Fit, wide, square);
        assert_eq!(uv, UnitRect::FULL);
        assert_eq!(
            area,
            UnitRect {
                min: (0.0, 0.25),
                max: (1.0, 0.75)
            }
        );
        let (uv, area) = frame_rects(FrameMethod::Crop, wide, square);
        assert_eq!(
            uv,
            UnitRect {
                min: (0.25, 0.0),
                max: (0.75, 1.0)
            }
        );
        assert_eq!(area, UnitRect::FULL);

        let tall = [100, 200];
        let (_, area) = frame_rects(FrameMethod::Fit, tall, square);
        assert_eq!(
            area,
            UnitRect {
                min: (0.25, 0.0),
                max: (0.75, 1.0)
            }
        );
        let (uv, _) = frame_rects(FrameMethod::Crop, tall, square);
        assert_eq!(
            uv,
            UnitRect {
                min: (0.0, 0.25),
                max: (1.0, 0.75)
            }
        );
    }

    #[test]
    fn corners_follow_rotation_and_scale() {
        let mut image = image(1);
        image.width = 20.0;
        image.height = 10.0;
        image.offset = Point::new(100.0, 50.0);
        assert!(close(image.corners()[0], Point::new(90.0, 45.0)));
        image.scale = 2.0;
        assert!(close(image.corners()[2], Point::new(120.0, 60.0)));
        image.scale = 1.0;
        image.rotation = FRAC_PI_2;
        // A quarter turn takes the top-left corner (-10, -5) to (5, -10).
        assert!(close(image.corners()[0], Point::new(105.0, 40.0)));
    }

    #[test]
    fn contains_tests_the_rotated_box() {
        let mut image = image(1);
        image.width = 20.0;
        image.height = 4.0;
        assert!(image.contains(Point::new(9.0, 0.0)));
        image.rotation = FRAC_PI_2;
        assert!(!image.contains(Point::new(9.0, 0.0)));
        assert!(image.contains(Point::new(0.0, 9.0)));
    }

    #[test]
    fn flips_swap_the_texture_coordinates() {
        let mut image = image(1);
        image.frame = FrameMethod::Stretch;
        assert_eq!(image.quad([1, 1]).uvs[0], (0.0, 0.0));
        image.flip_x = true;
        assert_eq!(image.quad([1, 1]).uvs[0], (1.0, 0.0));
        image.flip_y = true;
        assert_eq!(image.quad([1, 1]).uvs[0], (1.0, 1.0));
    }

    #[test]
    fn scaling_about_a_point_keeps_that_point() {
        let mut image = image(1);
        image.offset = Point::new(10.0, 0.0);
        let scaled = image.scaled_about(Point::new(0.0, 0.0), 2.0);
        assert!(close(scaled.offset, Point::new(20.0, 0.0)));
        assert!((scaled.width - image.width * 2.0).abs() < 1e-9);
    }

    #[test]
    fn paths_are_stored_relative_to_the_project_folder() {
        let project = Path::new("/work/wallet/wallet.leathercraft");
        let inside = Path::new("/work/wallet/res/skin.jpg");
        let outside = Path::new("/photos/skin.jpg");
        assert_eq!(
            stored_path(inside, Some(project)),
            PathBuf::from("res/skin.jpg")
        );
        assert_eq!(stored_path(outside, Some(project)), outside.to_path_buf());
        assert_eq!(stored_path(inside, None), inside.to_path_buf());
        assert_eq!(
            resolve_path(&stored_path(inside, Some(project)), Some(project)),
            inside.to_path_buf()
        );
        assert_eq!(resolve_path(outside, Some(project)), outside.to_path_buf());
    }

    #[test]
    fn invalid_images_are_rejected() {
        let mut project = Project::default();
        project.background_images.push(image(1));
        assert_eq!(project.validate(), Ok(()));

        project.background_images[0].opacity = 1.5;
        assert!(matches!(
            project.validate(),
            Err(Message::InvalidBackgroundImage { .. })
        ));
        project.background_images[0].opacity = 0.5;
        project.background_images[0].scale = 0.0;
        assert!(matches!(
            project.validate(),
            Err(Message::InvalidBackgroundImage { .. })
        ));
        project.background_images[0].scale = 1.0;
        project.background_images[0].offset.x = MAX_COORDINATE_MM * 2.0;
        assert!(matches!(
            project.validate(),
            Err(Message::InvalidBackgroundImage { .. })
        ));
        project.background_images[0].offset.x = 0.0;
        project.background_images.push(image(1));
        assert!(matches!(
            project.validate(),
            Err(Message::InvalidBackgroundImageId { .. })
        ));
    }

    #[test]
    fn a_project_without_images_omits_the_field() {
        let json = serde_json::to_string(&Project::default()).unwrap();
        assert!(!json.contains("background_images"));
        let mut project = Project::default();
        project.background_images.push(image(1));
        let json = serde_json::to_string(&project).unwrap();
        let loaded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.background_images, project.background_images);
    }
}
