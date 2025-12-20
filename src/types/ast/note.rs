//! Note and pitch representation in the AST.
//!
//! This module defines types for representing musical notes, pitches, and rests
//! as they appear in ABC notation, without interpretation or context resolution.

use super::{BrokenRhythm, Decoration};
use crate::types::{Accidental, Octave, PitchClass};

/// A musical note in ABC notation.
///
/// Notes in the AST preserve the original notation without interpretation.
/// Duration may be explicit (e.g., `A2`, `A/2`) or default (bare `A`), requiring
/// the unit note length from context to compute absolute duration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note<'a> {
    /// The pitch of the note.
    pub pitch: NotePitch,
    /// The duration of the note.
    pub duration: NoteDuration,
    /// Decorations applied to this note (ornaments, dynamics, articulation).
    pub decorations: Vec<Decoration<'a>>,
    /// Broken rhythm operator following this note, if any.
    ///
    /// Broken rhythm affects this note and the next note, adjusting their
    /// relative durations (e.g., `A>B` dots A and halves B).
    pub broken_rhythm: Option<BrokenRhythm>,
}

/// A pitch in ABC notation before conversion to MIDI.
///
/// Represents the pitch as written in ABC: a base pitch class (C–B),
/// an optional accidental, and an octave offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotePitch {
    /// The base pitch class (C, D, E, F, G, A, or B).
    pub base: PitchClass,
    /// The accidental applied to this note, if any.
    pub accidental: Option<Accidental>,
    /// The octave offset from the middle octave.
    pub octave: Octave,
}

/// The duration of a note as written in ABC notation.
///
/// Durations can be explicitly specified (e.g., `A2` for double length,
/// `A/2` for half length) or default (bare `A`), which requires the unit
/// note length from the tune's `L:` field or metre to interpret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteDuration {
    /// An explicitly specified duration (e.g., `A2`, `A/2`, `A3/4`).
    Explicit {
        /// The numerator of the duration multiplier.
        numerator: u32,
        /// The denominator of the duration multiplier.
        denominator: u32,
    },
    /// The default unit length (bare note, e.g., `A`).
    ///
    /// Requires the unit note length from context to compute the absolute duration.
    Default,
}

/// A rest in ABC notation.
///
/// Rests represent periods of silence and can be visible (notated in the score)
/// or invisible (used for spacing). Multi-measure rests represent multiple
/// measures of rest with a single symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rest {
    /// A visible rest (`z`) with the given duration.
    Visible(NoteDuration),
    /// An invisible rest (`x`) with the given duration.
    ///
    /// Invisible rests take up time but are not printed in the score.
    Invisible(NoteDuration),
    /// A multi-measure rest (`Z` or `X`) spanning multiple measures.
    MultiMeasure {
        /// The number of measures of rest.
        measures: u32,
        /// Whether the rest is visible (`Z`) or invisible (`X`).
        visible: bool,
    },
}
