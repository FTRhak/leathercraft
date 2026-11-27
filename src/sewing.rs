//! How a seam link is sewn: the stitch, the backstitched holes at each end, how the two pieces
//! are laid together, and the thread. Part of [`crate::seams::SeamLink`] and saved with it; the
//! editor shows it in the Link seams dialog and estimates the thread from it.

use crate::i18n::Message;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The stitch the seam is sewn with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StitchMethod {
    /// Two needles through every hole from both sides: the strongest, and it doesn't unravel
    /// when the thread breaks.
    #[default]
    Saddle,
    /// One needle, in and out: quick, but one break undoes the seam.
    Running,
    /// Two needles across two butted edges, the seam lying flat.
    Baseball,
    /// One needle wrapping over the edge.
    Whip,
    /// Crosses between two rows of holes, as decoration.
    Cross,
}

impl StitchMethod {
    pub const ALL: [Self; 5] = [
        Self::Saddle,
        Self::Running,
        Self::Baseball,
        Self::Whip,
        Self::Cross,
    ];

    /// The thread a stitch takes per length of seam, in tenths, tails included: the leather
    /// workers' rules of thumb, not a measurement.
    pub fn thread_factor_tenths(self) -> u32 {
        match self {
            Self::Saddle => 40,
            Self::Running => 20,
            Self::Baseball => 50,
            Self::Whip => 35,
            Self::Cross => 60,
        }
    }
}

/// How the two pieces lie against each other while they are sewn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assembly {
    /// Flesh side to flesh side, the edges flush: wallets, card holders.
    #[default]
    FleshToFlesh,
    /// One edge laid over the other by `Sewing::overlap_mm`.
    Lapped,
    /// Edge to edge in one plane.
    Butted,
    /// Grain side to grain side, sewn inside out and turned.
    Turned,
    /// One piece folded, its two edges sewn together.
    Folded,
}

impl Assembly {
    pub const ALL: [Self; 5] = [
        Self::FleshToFlesh,
        Self::Lapped,
        Self::Butted,
        Self::Turned,
        Self::Folded,
    ];

    /// Whether the overlap applies.
    pub fn uses_overlap(self) -> bool {
        self == Self::Lapped
    }

    /// Whether one piece lies on the other, so that it matters which is on top.
    pub fn stacks(self) -> bool {
        matches!(self, Self::FleshToFlesh | Self::Lapped | Self::Turned)
    }
}

/// Which side of the link lies on top.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnTop {
    #[default]
    A,
    B,
}

impl OnTop {
    pub fn letter(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
        }
    }

    /// The letter of the side underneath.
    pub fn other_letter(self) -> char {
        match self {
            Self::A => 'B',
            Self::B => 'A',
        }
    }
}

/// What the thread is made of.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadKind {
    /// Braided waxed polyester: round, strong, the usual thread for saddle stitching.
    #[default]
    WaxedPolyester,
    /// Waxed linen: matte, the traditional thread.
    WaxedLinen,
    /// Bonded nylon: smooth and hard wearing, sewn by machine as well.
    BondedNylon,
    /// Flat waxed artificial sinew, split for finer work.
    ArtificialSinew,
}

impl ThreadKind {
    pub const ALL: [Self; 4] = [
        Self::WaxedPolyester,
        Self::WaxedLinen,
        Self::BondedNylon,
        Self::ArtificialSinew,
    ];

    /// Whether the thread is a flat tape rather than a round cord.
    pub fn is_flat(self) -> bool {
        self == Self::ArtificialSinew
    }
}

/// A thread color in sRGB, saved as `"#RRGGBB"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThreadColor(pub [u8; 3]);

impl ThreadColor {
    pub fn rgb(self) -> [u8; 3] {
        self.0
    }

    /// `#RRGGBB`, in capitals.
    pub fn to_hex(self) -> String {
        let [red, green, blue] = self.0;
        format!("#{red:02X}{green:02X}{blue:02X}")
    }

    /// Reads `#RRGGBB` or `RRGGBB` in either case, with spaces around it.
    pub fn from_hex(text: &str) -> Option<Self> {
        let text = text.trim();
        let digits = text.strip_prefix('#').unwrap_or(text);
        if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
        Some(Self([channel(0)?, channel(2)?, channel(4)?]))
    }

    /// The named shade of exactly this color, if it is one.
    pub fn shade(self) -> Option<ThreadShade> {
        ThreadShade::ALL
            .into_iter()
            .find(|shade| shade.color() == self)
    }
}

impl Default for ThreadColor {
    fn default() -> Self {
        ThreadShade::Natural.color()
    }
}

impl Serialize for ThreadColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ThreadColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::from_hex(&text)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid thread color {text:?}")))
    }
}

/// The thread colors offered as swatches in the Link seams dialog; any other can be picked too.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThreadShade {
    Natural,
    White,
    Black,
    Brown,
    Tan,
    Red,
    Navy,
    Olive,
}

impl ThreadShade {
    pub const ALL: [Self; 8] = [
        Self::Natural,
        Self::White,
        Self::Black,
        Self::Brown,
        Self::Tan,
        Self::Red,
        Self::Navy,
        Self::Olive,
    ];

    pub const fn color(self) -> ThreadColor {
        ThreadColor(match self {
            Self::Natural => [0xE8, 0xDC, 0xC0],
            Self::White => [0xF2, 0xF0, 0xEA],
            Self::Black => [0x1C, 0x1C, 0x1E],
            Self::Brown => [0x6B, 0x42, 0x26],
            Self::Tan => [0xB0, 0x7A, 0x45],
            Self::Red => [0xA3, 0x26, 0x2A],
            Self::Navy => [0x24, 0x3A, 0x63],
            Self::Olive => [0x5C, 0x6B, 0x35],
        })
    }
}

pub const MIN_THREAD_MM: f64 = 0.2;
pub const MAX_THREAD_MM: f64 = 1.5;
const DEFAULT_THREAD_MM: f64 = 0.55;

/// The thread a seam is sewn with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Thread {
    pub kind: ThreadKind,
    /// Its diameter, in mm.
    pub diameter_mm: f64,
    pub color: ThreadColor,
}

/// How much wider than its diameter a flat thread lies, and how much thinner.
const FLAT_WIDTH: f64 = 1.6;
const FLAT_HEIGHT: f64 = 0.5;

impl Thread {
    /// Its cross-section as it lies on the leather, as (width, height) in mm: a round thread
    /// is its diameter both ways; a flat one is wider and thinner, about the same in area.
    pub fn profile_mm(&self) -> (f64, f64) {
        if self.kind.is_flat() {
            (
                self.diameter_mm * FLAT_WIDTH,
                self.diameter_mm * FLAT_HEIGHT,
            )
        } else {
            (self.diameter_mm, self.diameter_mm)
        }
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self {
            kind: ThreadKind::default(),
            diameter_mm: DEFAULT_THREAD_MM,
            color: ThreadColor::default(),
        }
    }
}

pub const MAX_LOCK_HOLES: u32 = 10;
pub const MIN_OVERLAP_MM: f64 = 1.0;
pub const MAX_OVERLAP_MM: f64 = 100.0;
const DEFAULT_LOCK_HOLES: u32 = 2;
const DEFAULT_OVERLAP_MM: f64 = 8.0;

/// How a link is sewn. Every field has a default, so links saved before it existed load as a
/// saddle stitch on pieces laid flesh to flesh, sewn with natural waxed polyester.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sewing {
    pub stitch: StitchMethod,
    /// How many holes are sewn back over at each end to lock the thread.
    pub lock_holes: u32,
    pub assembly: Assembly,
    /// How far the edges overlap when `assembly` is `Lapped`, in mm. Kept for the other
    /// assemblies, so switching back restores it.
    pub overlap_mm: f64,
    pub on_top: OnTop,
    pub thread: Thread,
}

impl Default for Sewing {
    fn default() -> Self {
        Self {
            stitch: StitchMethod::default(),
            lock_holes: DEFAULT_LOCK_HOLES,
            assembly: Assembly::default(),
            overlap_mm: DEFAULT_OVERLAP_MM,
            on_top: OnTop::default(),
            thread: Thread::default(),
        }
    }
}

impl Sewing {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The thread the seam takes, in mm: the seam's length plus the locked holes at both ends,
    /// times the stitch's factor.
    pub fn thread_mm(&self, seam_mm: f64, pitch_mm: f64) -> f64 {
        let locked = 2.0 * f64::from(self.lock_holes) * pitch_mm;
        (seam_mm + locked) * f64::from(self.stitch.thread_factor_tenths()) / 10.0
    }

    pub(crate) fn validate(&self) -> Result<(), Message> {
        if self.lock_holes > MAX_LOCK_HOLES {
            return Err(Message::LockHolesOutOfRange);
        }
        if !(MIN_OVERLAP_MM..=MAX_OVERLAP_MM).contains(&self.overlap_mm) {
            return Err(Message::OverlapOutOfRange);
        }
        if !(MIN_THREAD_MM..=MAX_THREAD_MM).contains(&self.thread.diameter_mm) {
            return Err(Message::ThreadDiameterOutOfRange);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_their_defaults() {
        let sewing: Sewing = serde_json::from_str(r#"{ "stitch": "Running" }"#).unwrap();
        assert_eq!(sewing.stitch, StitchMethod::Running);
        assert_eq!(sewing.lock_holes, DEFAULT_LOCK_HOLES);
        assert_eq!(sewing.assembly, Assembly::FleshToFlesh);
        assert_eq!(sewing.overlap_mm, DEFAULT_OVERLAP_MM);
        assert_eq!(sewing.on_top, OnTop::A);
        assert_eq!(sewing.thread, Thread::default());
    }

    #[test]
    fn the_thread_color_is_saved_as_hex() {
        let sewing = Sewing {
            thread: Thread {
                kind: ThreadKind::WaxedLinen,
                diameter_mm: 0.65,
                color: ThreadColor([0x12, 0xAB, 0xEF]),
            },
            ..Sewing::default()
        };
        let json = serde_json::to_string(&sewing).unwrap();
        assert!(json.contains(r##""color":"#12ABEF""##), "{json}");
        let back: Sewing = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sewing);
        assert!(serde_json::from_str::<ThreadColor>(r##""#12AB""##).is_err());
    }

    #[test]
    fn hex_is_read_leniently() {
        let natural = ThreadShade::Natural.color();
        assert_eq!(ThreadColor::from_hex("e8dcc0"), Some(natural));
        assert_eq!(ThreadColor::from_hex(" #E8DCC0 "), Some(natural));
        assert_eq!(ThreadColor::from_hex("#E8DCCG"), None);
        assert_eq!(ThreadColor::from_hex("#E8DCC0F"), None);
        assert_eq!(ThreadColor::from_hex("#ЕЕЕ"), None);
        assert_eq!(natural.to_hex(), "#E8DCC0");
        assert_eq!(ThreadShade::Navy.color().shade(), Some(ThreadShade::Navy));
        assert_eq!(ThreadColor([1, 2, 3]).shade(), None);
    }

    #[test]
    fn thread_grows_with_the_stitch_and_the_locks() {
        let mut sewing = Sewing {
            lock_holes: 0,
            ..Sewing::default()
        };
        assert!((sewing.thread_mm(200.0, 4.0) - 800.0).abs() < 1.0e-9);
        sewing.lock_holes = 2;
        // 16 mm more of seam, at both ends: (200 + 16) × 4.
        assert!((sewing.thread_mm(200.0, 4.0) - 864.0).abs() < 1.0e-9);
        sewing.stitch = StitchMethod::Running;
        assert!((sewing.thread_mm(200.0, 4.0) - 432.0).abs() < 1.0e-9);
    }

    #[test]
    fn limits_are_checked() {
        assert!(Sewing::default().validate().is_ok());
        let locks = Sewing {
            lock_holes: MAX_LOCK_HOLES + 1,
            ..Sewing::default()
        };
        assert_eq!(locks.validate(), Err(Message::LockHolesOutOfRange));
        for overlap_mm in [0.0, f64::NAN, MAX_OVERLAP_MM + 1.0] {
            let overlap = Sewing {
                overlap_mm,
                ..Sewing::default()
            };
            assert_eq!(overlap.validate(), Err(Message::OverlapOutOfRange));
        }
        for diameter_mm in [0.1, f64::NAN, MAX_THREAD_MM + 0.1] {
            let thread = Sewing {
                thread: Thread {
                    diameter_mm,
                    ..Thread::default()
                },
                ..Sewing::default()
            };
            assert_eq!(thread.validate(), Err(Message::ThreadDiameterOutOfRange));
        }
    }

    #[test]
    fn sinew_lies_flat_and_round_threads_do_not() {
        let round = Thread::default();
        assert_eq!(round.profile_mm(), (round.diameter_mm, round.diameter_mm));
        let sinew = Thread {
            kind: ThreadKind::ArtificialSinew,
            diameter_mm: 1.0,
            ..Thread::default()
        };
        let (width, height) = sinew.profile_mm();
        assert!(width > 1.0 && height < 1.0);
    }

    #[test]
    fn only_stacked_assemblies_have_a_top() {
        assert!(Assembly::Lapped.uses_overlap() && Assembly::Lapped.stacks());
        assert!(!Assembly::Butted.stacks() && !Assembly::Folded.stacks());
        assert!(!Assembly::FleshToFlesh.uses_overlap());
    }
}
