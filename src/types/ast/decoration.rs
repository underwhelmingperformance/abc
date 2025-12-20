//! Decorations, ornaments, and articulation marks in the AST.
//!
//! This module defines types for representing decorations (ornaments, dynamics,
//! articulation marks) and grace notes as they appear in ABC notation.

use std::borrow::Cow;

use super::Note;
use crate::types::Dynamics;

/// A decoration applied to a note.
///
/// Decorations include ornaments (trills, mordents), dynamics (loud/soft),
/// articulation marks (staccato, accent), and other symbols that modify
/// how a note is performed. ABC notation supports both shorthand notation
/// (single characters) and explicit notation (`!decoration!` or `+decoration+`).
///
/// # Examples
///
/// ```text
/// ~C      - Shorthand trill on C
/// !trill!C - Explicit trill on C
/// .C      - Staccato
/// !f!C    - Forte (loud)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoration<'a> {
    /// Shorthand decoration using a single character (e.g., `~`, `.`, `T`).
    Shorthand(char),
    /// Explicit decoration name (e.g., `!trill!`, `!staccato!`, `!accent!`).
    /// Uses `Cow` to allow zero-copy parsing when borrowing from input.
    Explicit(Cow<'a, str>),
    /// A dynamic marking (volume level, accent, or expression).
    Dynamic(Dynamic),
}

/// A dynamic marking in ABC notation.
///
/// This represents dynamic markings as they appear in notation, which can be:
/// - Static volume levels (pp, mf, ff, etc.) - these set a sustained loudness
/// - Sforzando (sfz) - a sudden accent on a single note
/// - Crescendo/diminuendo markers - gradual changes spanning multiple notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dynamic {
    /// A static volume level (pppp through ffff).
    ///
    /// These set the sustained loudness for subsequent notes until changed.
    Level(Dynamics),
    /// `!sfz!` - Sforzando (sudden accent on a single note).
    ///
    /// Unlike volume levels, sfz is a momentary accent that doesn't change
    /// the underlying dynamic level.
    Sfz,
    /// `!crescendo(!` or `!<(!` - Begin crescendo (gradually louder).
    CrescendoStart,
    /// `!crescendo)!` or `!<)!` - End crescendo.
    CrescendoEnd,
    /// `!diminuendo(!` or `!>(!` - Begin diminuendo (gradually softer).
    DiminuendoStart,
    /// `!diminuendo)!` or `!>)!` - End diminuendo.
    DiminuendoEnd,
}

/// Grace notes.
///
/// Grace notes are ornamental notes played quickly before a main note,
/// typically borrowing time from the main note or the preceding note.
/// In ABC notation, grace notes appear in curly braces.
///
/// # Examples
///
/// ```text
/// {A}C       - Single grace note before C
/// {AB}C      - Multiple grace notes
/// {/A}C      - Acciaccatura (slashed grace note, played very quickly)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraceNotes<'a> {
    /// The grace notes to be played.
    pub notes: Vec<Note<'a>>,
    /// Whether this is an acciaccatura (slashed grace note).
    ///
    /// Acciaccaturas are played very quickly and are notated with a slash
    /// through the stem in traditional notation, written as `{/...}` in ABC.
    pub acciaccatura: bool,
}
