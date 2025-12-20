//! Abstract dynamics types.
//!
//! This module provides abstract musical dynamics that are independent of
//! any specific output format like MIDI. Dynamics represent relative loudness
//! levels using standard musical terminology.

/// An abstract dynamics marking.
///
/// These are the standard musical dynamics, independent of MIDI velocity.
/// Conversion to MIDI velocity happens in the MIDI export module.
///
/// The variants are ordered from softest to loudest, allowing comparison.
///
/// # Examples
///
/// ```
/// # use abc::types::Dynamics;
/// let soft = Dynamics::PP;
/// let loud = Dynamics::FF;
/// assert!(soft < loud);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dynamics {
    /// Pianissississimo (as soft as possible).
    PPPP,
    /// Pianississimo (very very soft).
    PPP,
    /// Pianissimo (very soft).
    PP,
    /// Piano (soft).
    P,
    /// Mezzo-piano (moderately soft).
    MP,
    /// Mezzo-forte (moderately loud).
    MF,
    /// Forte (loud).
    F,
    /// Fortissimo (very loud).
    FF,
    /// Fortississimo (very very loud).
    FFF,
    /// Fortissississimo (as loud as possible).
    FFFF,
}

/// Direction of dynamic change (hairpin/wedge markings).
///
/// This represents whether a note is part of a crescendo (getting louder)
/// or diminuendo (getting softer) passage. Hairpins are graphical indicators
/// in music notation that show gradual changes in dynamics.
///
/// # Examples
///
/// ```
/// # use abc::types::DynamicDirection;
/// let growing = DynamicDirection::Crescendo;
/// let fading = DynamicDirection::Diminuendo;
/// let steady = DynamicDirection::None;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DynamicDirection {
    /// No dynamic direction change - steady dynamics.
    #[default]
    None,
    /// Crescendo - gradually getting louder.
    ///
    /// Marked in ABC notation with `!crescendo(!` or `!<(!` at the start
    /// and `!crescendo)!` or `!<)!` at the end.
    Crescendo,
    /// Diminuendo (or decrescendo) - gradually getting softer.
    ///
    /// Marked in ABC notation with `!diminuendo(!` or `!>(!` at the start
    /// and `!diminuendo)!` or `!>)!` at the end.
    Diminuendo,
}

impl Default for Dynamics {
    /// Returns the default dynamics level (mezzo-forte).
    ///
    /// Mezzo-forte is the conventional default when no dynamics marking
    /// is specified.
    fn default() -> Self {
        Self::MF
    }
}

impl Dynamics {
    /// Returns a human-readable name for this dynamics level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Dynamics;
    /// assert_eq!(Dynamics::PP.name(), "pianissimo");
    /// assert_eq!(Dynamics::F.name(), "forte");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PPPP => "pianissississimo",
            Self::PPP => "pianississimo",
            Self::PP => "pianissimo",
            Self::P => "piano",
            Self::MP => "mezzo-piano",
            Self::MF => "mezzo-forte",
            Self::F => "forte",
            Self::FF => "fortissimo",
            Self::FFF => "fortississimo",
            Self::FFFF => "fortissississimo",
        }
    }

    /// Returns the standard abbreviation for this dynamics level.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Dynamics;
    /// assert_eq!(Dynamics::PP.abbreviation(), "pp");
    /// assert_eq!(Dynamics::MF.abbreviation(), "mf");
    /// ```
    #[must_use]
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::PPPP => "pppp",
            Self::PPP => "ppp",
            Self::PP => "pp",
            Self::P => "p",
            Self::MP => "mp",
            Self::MF => "mf",
            Self::F => "f",
            Self::FF => "ff",
            Self::FFF => "fff",
            Self::FFFF => "ffff",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamics_ordering() {
        let mut dynamics = vec![
            Dynamics::MF,
            Dynamics::FFFF,
            Dynamics::PP,
            Dynamics::F,
            Dynamics::PPPP,
            Dynamics::MP,
            Dynamics::FFF,
            Dynamics::P,
            Dynamics::PPP,
            Dynamics::FF,
        ];
        dynamics.sort();
        assert_eq!(
            dynamics,
            vec![
                Dynamics::PPPP,
                Dynamics::PPP,
                Dynamics::PP,
                Dynamics::P,
                Dynamics::MP,
                Dynamics::MF,
                Dynamics::F,
                Dynamics::FF,
                Dynamics::FFF,
                Dynamics::FFFF,
            ]
        );
    }

    #[test]
    fn default_is_mezzo_forte() {
        assert_eq!(Dynamics::default(), Dynamics::MF);
    }
}
