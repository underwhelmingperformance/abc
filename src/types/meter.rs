//! Metre (time signature) and tempo representation in the AST.
//!
//! This module defines types for representing time signatures and tempo markings
//! as they appear in ABC notation.

use super::Duration;

/// A metre (time signature).
///
/// The metre specifies how many beats are in each measure and which note value
/// constitutes one beat. In ABC notation, metres are specified with the `M:` field.
///
/// # Examples
///
/// ```text
/// M:4/4     - Four quarter-note beats per measure (common time)
/// M:6/8     - Six eighth-note beats per measure
/// M:3/4     - Three quarter-note beats per measure (waltz time)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Meter {
    /// The number of beats per measure (numerator).
    pub numerator: u8,
    /// The note value that gets one beat (denominator).
    pub denominator: u8,
}

impl Default for Meter {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

/// A metre symbol, which may be a standard notation shorthand or an explicit fraction.
///
/// ABC notation supports both symbolic representations (like `C` for common time)
/// and explicit fractional time signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeterSymbol {
    /// `C` - Common time (4/4).
    CommonTime,
    /// `C|` - Cut time or alla breve (2/2).
    CutTime,
    /// An explicit time signature (e.g., `3/4`, `6/8`).
    Explicit(Meter),
}

/// A tempo marking specifying the speed of performance.
///
/// Tempo in ABC notation is specified with the `Q:` field, indicating how many
/// beats of a given note value occur per minute.
///
/// # Examples
///
/// ```text
/// Q:1/4=120     - 120 quarter notes per minute
/// Q:1/8=180     - 180 eighth notes per minute
/// Q:3/8=50      - 50 dotted quarter notes per minute
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempoMarking {
    /// The note value that the tempo refers to.
    pub note_value: Duration,
    /// The number of beats per minute.
    pub beats_per_minute: u32,
}
