//! How a seam link is sewn: the stitch, the backstitched holes at each end, and how the two
//! pieces are laid together. Part of [`crate::seams::SeamLink`] and saved with it; the editor
//! shows it in the Link seams dialog and estimates the thread from it.

use crate::i18n::Message;
use serde::{Deserialize, Serialize};

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

pub const MAX_LOCK_HOLES: u32 = 10;
pub const MIN_OVERLAP_MM: f64 = 1.0;
pub const MAX_OVERLAP_MM: f64 = 100.0;
const DEFAULT_LOCK_HOLES: u32 = 2;
const DEFAULT_OVERLAP_MM: f64 = 8.0;

/// How a link is sewn. Every field has a default, so links saved before it existed load as a
/// saddle stitch on pieces laid flesh to flesh.
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
}

impl Default for Sewing {
    fn default() -> Self {
        Self {
            stitch: StitchMethod::default(),
            lock_holes: DEFAULT_LOCK_HOLES,
            assembly: Assembly::default(),
            overlap_mm: DEFAULT_OVERLAP_MM,
            on_top: OnTop::default(),
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
    }

    #[test]
    fn only_stacked_assemblies_have_a_top() {
        assert!(Assembly::Lapped.uses_overlap() && Assembly::Lapped.stacks());
        assert!(!Assembly::Butted.stacks() && !Assembly::Folded.stacks());
        assert!(!Assembly::FleshToFlesh.uses_overlap());
    }
}
