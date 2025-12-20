//! Music elements in the tune body.
//!
//! This module defines the `BodyElement` enum, which represents all possible
//! elements that can appear in the body of an ABC tune. This is the primary
//! union type for the AST.

use super::{
    Annotation, BarLine, Chord, Decoration, Directive, GraceNotes, GuitarChord, LyricLine, Note,
    Rest, Tuplet, VariantEnding, VoiceDeclaration,
};
use crate::types::{Duration, KeySignature, MeterSymbol, PartLabel, TempoMarking};

/// An element in the body of an ABC tune.
///
/// The body of a tune consists of a sequence of elements including notes,
/// rests, chords, bar lines, and various directives that control notation
/// and performance. This enum represents all possible element types.
///
/// # Lifetimes
///
/// The `'input` lifetime represents borrowed data from the original ABC
/// notation source. Most text-based elements borrow from the input to
/// minimize allocations.
#[derive(Debug, PartialEq)]
pub enum BodyElement<'input> {
    /// A single note.
    Note(Note<'input>),
    /// A rest (silence).
    Rest(Rest),
    /// A chord (multiple notes sounded simultaneously).
    Chord(Chord<'input>),
    /// A bar line, possibly with repeat markers.
    BarLine(BarLine),
    /// A variant ending (first/second time ending).
    ///
    /// Variant endings specify which measures are played during different
    /// repetitions through a repeated section.
    VariantEnding(VariantEnding),
    /// A guitar chord symbol (e.g., "Am7", "G/B").
    ///
    /// Guitar chord symbols provide harmonic information for accompaniment,
    /// typically displayed above the staff.
    GuitarChord(GuitarChord<'input>),
    /// An inline field that modifies the current context.
    InlineField(InlineField<'input>),
    /// Switch to a different voice (with optional declaration of voice attributes).
    VoiceSwitch(VoiceDeclaration<'input>),
    /// Begin a voice overlay (& symbol).
    ///
    /// Voice overlays allow multiple melodic lines to be written on the
    /// same staff, typically used for brief polyphonic passages.
    VoiceOverlay,
    /// A decoration (ornament, dynamic, or articulation mark).
    Decoration(Decoration<'input>),
    /// Grace notes preceding a main note.
    GraceNotes(GraceNotes<'input>),
    /// A tuplet (irregular rhythmic grouping).
    ///
    /// The tuplet contains the notes that should be grouped together.
    /// This creates a recursive structure as tuplets contain body elements.
    Tuplet {
        /// The tuplet specification (p:q:r values).
        spec: Tuplet,
        /// The elements contained within this tuplet.
        elements: Vec<BodyElement<'input>>,
    },
    /// A macro invocation.
    ///
    /// Macros allow frequently-used sequences to be defined once and
    /// referenced by a single character. The symbol is parsed here; the
    /// actual definition is looked up during transformation from the
    /// macro definitions in the tune header.
    MacroInvocation(char),
    /// Start of a tie between notes.
    ///
    /// Ties connect two notes of the same pitch, causing them to be
    /// performed as a single sustained note.
    TieStart,
    /// End of a tie between notes.
    TieEnd,
    /// Start of a slur grouping.
    ///
    /// Slurs connect multiple notes (possibly of different pitches),
    /// indicating they should be played smoothly without separation.
    /// The boolean flag indicates whether this is a dotted slur (true) or regular slur (false).
    SlurStart { dotted: bool },
    /// End of a slur grouping.
    SlurEnd,
    /// A text annotation placed around a note.
    Annotation(Annotation<'input>),
    /// A line of lyrics aligned with notes.
    LyricLine(LyricLine<'input>),
    /// A part marker indicating the start of a named section.
    ///
    /// Part markers allow tunes to be divided into sections (A, B, C, etc.)
    /// that can be referenced in the part sequence.
    PartMarker(PartLabel),
    /// A directive (`%%...`) embedded in the tune body.
    ///
    /// Directives control notation rendering, MIDI playback, and other
    /// processing aspects. They may appear at any point in the tune body.
    Directive(Directive<'input>),
}

/// An inline field that modifies the current musical context.
///
/// Inline fields appear within square brackets in the tune body (e.g., `[K:D]`,
/// `[M:3/4]`) and change the key, metre, tempo, or other properties for the
/// music that follows. Unlike header fields, inline fields affect only the
/// current voice from that point forward.
#[derive(Debug, PartialEq)]
pub enum InlineField<'input> {
    /// Key signature change.
    Key(KeySignature),
    /// Metre (time signature) change.
    Meter(MeterSymbol),
    /// Tempo change.
    Tempo(TempoMarking),
    /// Unit note length change.
    UnitNoteLength(Duration),
    /// Voice declaration or switch.
    Voice(VoiceDeclaration<'input>),
}

/// A macro definition.
///
/// Macros allow sequences of body elements to be defined once and reused
/// by invoking a single character. This is useful for frequently-repeated
/// ornaments, rhythmic patterns, or melodic fragments.
///
/// # Examples
///
/// ```text
/// H: m = !trill!C2D2    % Define macro 'm'
/// ABC m DEF m            % Use macro twice
/// ```
#[derive(Debug, PartialEq)]
pub struct MacroDefinition<'input> {
    /// The character used to invoke this macro.
    pub symbol: char,
    /// The sequence of elements this macro expands to.
    pub content: Vec<BodyElement<'input>>,
}
