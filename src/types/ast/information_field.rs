//! Information fields in the AST.
//!
//! This module defines types for representing ABC information fields, which
//! provide metadata and musical parameters for tunes. Information fields
//! appear in the file header, tune header, or tune body.

use std::rc::Rc;

use super::{Directive, LyricLine, MacroDefinition, VoiceDeclaration};
use crate::types::{Duration, KeySignature, MeterSymbol, ReferenceNumber, TempoMarking};

/// An ABC information field.
///
/// Information fields provide metadata about tunes and control various aspects
/// of notation and performance. They are written as a field identifier (one or
/// two characters) followed by a colon and the field content.
///
/// Required fields for a valid tune are:
/// - `X:` (reference number)
/// - `T:` (title)
/// - `K:` (key signature, must be last in header)
///
/// # Examples
///
/// ```text
/// X:1
/// T:A Simple Tune
/// C:Traditional
/// M:4/4
/// L:1/8
/// K:G
/// ```
#[derive(Debug, PartialEq)]
pub enum InformationField<'input> {
    /// `X:` - Reference number uniquely identifying this tune.
    ///
    /// This field is required and must be the first field in a tune.
    ReferenceNumber(ReferenceNumber),

    /// `T:` - Title of the tune.
    ///
    /// This field is required. Multiple title fields may be present to
    /// provide alternative titles.
    Title(&'input str),

    /// `K:` - Key signature.
    ///
    /// This field is required and must be the last field in the tune header.
    Key(KeySignature),

    /// `M:` - Metre (time signature).
    Meter(MeterSymbol),

    /// `L:` - Unit note length (default duration for notes).
    ///
    /// Specifies the default duration for notes without explicit length.
    /// Common values are `1/8` and `1/16`.
    UnitNoteLength(Duration),

    /// `Q:` - Tempo marking.
    Tempo(TempoMarking),

    /// `R:` - Rhythm type (e.g., "reel", "jig", "waltz").
    Rhythm(&'input str),

    /// `P:` - Parts specification.
    ///
    /// Defines the sequence of parts in the tune (e.g., `AABB` or `A(AB)3`).
    Parts(&'input str),

    /// `C:` - Composer of the tune.
    Composer(&'input str),

    /// `O:` - Origin or geographical source.
    Origin(&'input str),

    /// `A:` - Area or region.
    Area(&'input str),

    /// `B:` - Book or collection reference.
    Book(&'input str),

    /// `D:` - Discography reference (recording information).
    Discography(&'input str),

    /// `F:` - File URL or path to the original source.
    FileUrl(&'input str),

    /// `G:` - Group (e.g., band or ensemble).
    Group(&'input str),

    /// `H:` - History or background information.
    History(&'input str),

    /// `N:` - Notes or comments about the tune.
    Notes(&'input str),

    /// `S:` - Source of the transcription.
    Source(&'input str),

    /// `Z:` - Transcription information (who transcribed, when).
    Transcription(&'input str),

    /// `W:` - Words (aligned lyrics in the header).
    ///
    /// Header lyrics are typically displayed as verses below the tune.
    Words(&'input str),

    /// `w:` - Inline lyrics (aligned with notes in the body).
    InlineLyrics(LyricLine<'input>),

    /// `V:` - Voice declaration or switch.
    Voice(VoiceDeclaration<'input>),

    /// `m:` - Macro definition.
    ///
    /// Defines a reusable sequence of music elements that can be invoked
    /// by a single character.
    Macro(Rc<MacroDefinition<'input>>),

    /// `I:` - Instruction or directive.
    ///
    /// Used for processing instructions and stylesheet directives.
    Instruction(&'input str),

    /// `s:` - Symbol line (for chord symbols).
    SymbolLine(&'input str),

    /// `r:` - Remark (comment).
    Remark(&'input str),

    /// `U:` - User-defined symbol.
    ///
    /// Allows custom decoration symbols to be defined.
    UserDefined {
        /// The symbol character being defined.
        symbol: char,
        /// The decoration name it maps to.
        decoration: &'input str,
    },

    /// `%%` - A directive controlling notation or playback.
    ///
    /// Directives are pseudo-fields that control rendering, MIDI output,
    /// and other processing aspects.
    Directive(Directive<'input>),
}
