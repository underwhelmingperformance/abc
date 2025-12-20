//! Chord representation in the AST.
//!
//! This module defines types for representing chords in ABC notation, including
//! both simultaneous notes (vertical harmony) and guitar chord symbols.

use super::{Note, NoteDuration};

/// A chord of simultaneously played notes.
///
/// In ABC notation, chords are written with square brackets containing multiple
/// notes, e.g., `[CEG]` for a C major chord. All notes in the chord are played
/// at the same time.
///
/// # Examples
///
/// ```text
/// [CEG]     - C major chord with default duration
/// [C2E2G2]  - C major chord, each note held for double length
/// [^F_A]    - F♯ and A♭ played together
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord<'a> {
    /// The notes that make up this chord, played simultaneously.
    pub notes: Vec<Note<'a>>,
    /// The duration of the entire chord.
    ///
    /// If present, this overrides individual note durations. If not specified,
    /// each note's individual duration is used.
    pub duration: Option<NoteDuration>,
}

/// A guitar chord symbol.
///
/// Guitar chord symbols appear in ABC notation as quoted text above the staff,
/// indicating the harmony to be played as accompaniment. For example, `"Am7"`
/// indicates an A minor 7th chord.
///
/// # Examples
///
/// ```text
/// "C"       - C major
/// "Am7"     - A minor 7th
/// "G/B"     - G major with B in the bass
/// "Dmaj7"   - D major 7th
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct GuitarChord<'input> {
    /// The chord symbol text (e.g., "Am7", "G/B").
    pub symbol: &'input str,
    /// Optional bass note specification after a slash (e.g., the "B" in "G/B").
    pub bass_note: Option<&'input str>,
}
