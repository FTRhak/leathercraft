use serde::{Deserialize, Serialize};

pub const MIN_THICKNESS_MM: f64 = 0.3;
pub const MAX_THICKNESS_MM: f64 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Calf,
    #[default]
    Cow,
    Bull,
    Goat,
    Sheep,
    Ram,
    Pig,
    Horse,
    Deer,
    Elk,
    Crocodile,
    Snake,
    Ostrich,
}

impl Origin {
    pub const ALL: [Self; 13] = [
        Self::Calf,
        Self::Cow,
        Self::Bull,
        Self::Goat,
        Self::Sheep,
        Self::Ram,
        Self::Pig,
        Self::Horse,
        Self::Deer,
        Self::Elk,
        Self::Crocodile,
        Self::Snake,
        Self::Ostrich,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Treatment {
    Raw,
    Dried,
    Soaked,
    #[default]
    VegetableTanned,
    ChromeTanned,
    AldehydeTanned,
    OilTanned,
    CombinationTanned,
    Suede,
    Nubuck,
    Patent,
    Impregnated,
}

impl Treatment {
    pub const ALL: [Self; 12] = [
        Self::Raw,
        Self::Dried,
        Self::Soaked,
        Self::VegetableTanned,
        Self::ChromeTanned,
        Self::AldehydeTanned,
        Self::OilTanned,
        Self::CombinationTanned,
        Self::Suede,
        Self::Nubuck,
        Self::Patent,
        Self::Impregnated,
    ];
}

/// Derived from the thickness in millimeters, so it is never stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThicknessCategory {
    ExtraThin,
    Thin,
    Medium,
    Dense,
    Thick,
    ExtraThick,
    Heavy,
}

impl ThicknessCategory {
    /// A lower bound belongs to its category, except that exactly 5 mm is not yet `Heavy`.
    pub fn from_millimeters(thickness_mm: f64) -> Self {
        match thickness_mm {
            value if value < 0.6 => Self::ExtraThin,
            value if value < 1.0 => Self::Thin,
            value if value < 1.5 => Self::Medium,
            value if value < 2.0 => Self::Dense,
            value if value < 3.0 => Self::Thick,
            value if value <= 5.0 => Self::ExtraThick,
            _ => Self::Heavy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorGroup {
    White,
    Black,
    Brown,
    Ginger,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
    Unusual,
}

impl ColorGroup {
    pub const ALL: [Self; 10] = [
        Self::White,
        Self::Black,
        Self::Brown,
        Self::Ginger,
        Self::Red,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Purple,
        Self::Unusual,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeatherColor {
    White,
    Milky,
    Cream,
    Black,
    Charcoal,
    Graphite,
    LightBrown,
    Chocolate,
    #[default]
    Chestnut,
    Ginger,
    Copper,
    Terracotta,
    Red,
    Burgundy,
    Wine,
    Yellow,
    Golden,
    Amber,
    Olive,
    Emerald,
    Malachite,
    LightBlue,
    Blue,
    DarkBlue,
    Lavender,
    Magenta,
    Violet,
    Pearl,
    Iridescent,
    Metallic,
}

impl LeatherColor {
    pub const ALL: [Self; 30] = [
        Self::White,
        Self::Milky,
        Self::Cream,
        Self::Black,
        Self::Charcoal,
        Self::Graphite,
        Self::LightBrown,
        Self::Chocolate,
        Self::Chestnut,
        Self::Ginger,
        Self::Copper,
        Self::Terracotta,
        Self::Red,
        Self::Burgundy,
        Self::Wine,
        Self::Yellow,
        Self::Golden,
        Self::Amber,
        Self::Olive,
        Self::Emerald,
        Self::Malachite,
        Self::LightBlue,
        Self::Blue,
        Self::DarkBlue,
        Self::Lavender,
        Self::Magenta,
        Self::Violet,
        Self::Pearl,
        Self::Iridescent,
        Self::Metallic,
    ];

    pub fn group(self) -> ColorGroup {
        match self {
            Self::White | Self::Milky | Self::Cream => ColorGroup::White,
            Self::Black | Self::Charcoal | Self::Graphite => ColorGroup::Black,
            Self::LightBrown | Self::Chocolate | Self::Chestnut => ColorGroup::Brown,
            Self::Ginger | Self::Copper | Self::Terracotta => ColorGroup::Ginger,
            Self::Red | Self::Burgundy | Self::Wine => ColorGroup::Red,
            Self::Yellow | Self::Golden | Self::Amber => ColorGroup::Yellow,
            Self::Olive | Self::Emerald | Self::Malachite => ColorGroup::Green,
            Self::LightBlue | Self::Blue | Self::DarkBlue => ColorGroup::Blue,
            Self::Lavender | Self::Magenta | Self::Violet => ColorGroup::Purple,
            Self::Pearl | Self::Iridescent | Self::Metallic => ColorGroup::Unusual,
        }
    }

    /// The sRGB color shown in the interface and the 3D preview.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::White => [245, 245, 240],
            Self::Milky => [240, 234, 214],
            Self::Cream => [238, 223, 186],
            Self::Black => [20, 20, 20],
            Self::Charcoal => [54, 54, 56],
            Self::Graphite => [82, 86, 90],
            Self::LightBrown => [170, 122, 80],
            Self::Chocolate => [78, 46, 28],
            Self::Chestnut => [122, 59, 27],
            Self::Ginger => [190, 94, 36],
            Self::Copper => [184, 115, 51],
            Self::Terracotta => [190, 96, 68],
            Self::Red => [178, 34, 34],
            Self::Burgundy => [110, 20, 40],
            Self::Wine => [114, 47, 55],
            Self::Yellow => [232, 196, 60],
            Self::Golden => [212, 170, 60],
            Self::Amber => [214, 140, 30],
            Self::Olive => [107, 112, 50],
            Self::Emerald => [30, 130, 90],
            Self::Malachite => [20, 150, 110],
            Self::LightBlue => [130, 180, 220],
            Self::Blue => [40, 80, 170],
            Self::DarkBlue => [24, 38, 86],
            Self::Lavender => [170, 150, 210],
            Self::Magenta => [130, 40, 120],
            Self::Violet => [90, 50, 150],
            Self::Pearl => [226, 222, 214],
            Self::Iridescent => [150, 170, 200],
            Self::Metallic => [160, 162, 166],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Texture {
    Matte,
    #[default]
    SemiMatte,
    Rough,
    Porous,
    Sanded,
    Embossed,
    Perforated,
    Scaly,
    Fibrous,
    Marbled,
    Cracked,
    NaturalScars,
}

impl Texture {
    pub const ALL: [Self; 12] = [
        Self::Matte,
        Self::SemiMatte,
        Self::Rough,
        Self::Porous,
        Self::Sanded,
        Self::Embossed,
        Self::Perforated,
        Self::Scaly,
        Self::Fibrous,
        Self::Marbled,
        Self::Cracked,
        Self::NaturalScars,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strength {
    VeryLow,
    Low,
    Medium,
    #[default]
    High,
    VeryHigh,
}

impl Strength {
    pub const ALL: [Self; 5] = [
        Self::VeryLow,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::VeryHigh,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Flexibility {
    VerySoft,
    Soft,
    SemiSoft,
    #[default]
    Medium,
    Stiff,
    VeryStiff,
    Rigid,
}

impl Flexibility {
    pub const ALL: [Self; 7] = [
        Self::VerySoft,
        Self::Soft,
        Self::SemiSoft,
        Self::Medium,
        Self::Stiff,
        Self::VeryStiff,
        Self::Rigid,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterResistance {
    None,
    #[default]
    Low,
    Moderate,
    High,
    Waterproof,
    Absolute,
}

impl WaterResistance {
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Low,
        Self::Moderate,
        Self::High,
        Self::Waterproof,
        Self::Absolute,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatResistance {
    None,
    Low,
    #[default]
    Moderate,
    High,
    VeryHigh,
}

impl HeatResistance {
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Low,
        Self::Moderate,
        Self::High,
        Self::VeryHigh,
    ];
}

/// The leather a piece is cut from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub origin: Origin,
    pub treatment: Treatment,
    pub thickness_mm: f64,
    pub color: LeatherColor,
    pub texture: Texture,
    pub strength: Strength,
    pub flexibility: Flexibility,
    pub water_resistance: WaterResistance,
    pub heat_resistance: HeatResistance,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            origin: Origin::default(),
            treatment: Treatment::default(),
            thickness_mm: 1.2,
            color: LeatherColor::default(),
            texture: Texture::default(),
            strength: Strength::default(),
            flexibility: Flexibility::default(),
            water_resistance: WaterResistance::default(),
            heat_resistance: HeatResistance::default(),
        }
    }
}

impl Material {
    pub fn thickness_category(&self) -> ThicknessCategory {
        ThicknessCategory::from_millimeters(self.thickness_mm)
    }

    pub fn has_valid_thickness(&self) -> bool {
        (MIN_THICKNESS_MM..=MAX_THICKNESS_MM).contains(&self.thickness_mm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thickness_categories_follow_their_bounds() {
        let category = ThicknessCategory::from_millimeters;
        assert_eq!(category(0.3), ThicknessCategory::ExtraThin);
        assert_eq!(category(0.6), ThicknessCategory::Thin);
        assert_eq!(category(1.0), ThicknessCategory::Medium);
        assert_eq!(category(1.5), ThicknessCategory::Dense);
        assert_eq!(category(2.0), ThicknessCategory::Thick);
        assert_eq!(category(3.0), ThicknessCategory::ExtraThick);
        assert_eq!(category(5.0), ThicknessCategory::ExtraThick);
        assert_eq!(category(5.1), ThicknessCategory::Heavy);
    }

    #[test]
    fn every_color_group_has_three_colors() {
        for group in ColorGroup::ALL {
            let colors = LeatherColor::ALL
                .into_iter()
                .filter(|color| color.group() == group)
                .count();
            assert_eq!(colors, 3, "{group:?}");
        }
    }

    #[test]
    fn thickness_limits_reject_values_outside_the_range() {
        let with = |thickness_mm| Material {
            thickness_mm,
            ..Material::default()
        };
        assert!(Material::default().has_valid_thickness());
        assert!(with(MIN_THICKNESS_MM).has_valid_thickness());
        assert!(with(MAX_THICKNESS_MM).has_valid_thickness());
        assert!(!with(0.1).has_valid_thickness());
        assert!(!with(21.0).has_valid_thickness());
        assert!(!with(f64::NAN).has_valid_thickness());
    }
}
