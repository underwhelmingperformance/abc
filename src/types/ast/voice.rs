//! Voice declarations and attributes in the AST.
//!
//! This module defines types for representing voice declarations and their
//! attributes as they appear in ABC notation. Voices allow multiple melodic
//! lines to be notated in a single tune.

use crate::types::{Clef, Instrument, KeySignature, VoiceId};

/// A voice declaration specifying attributes for a melodic line.
///
/// Voices in ABC notation allow multiple independent melodic lines to be
/// written in a single tune. Each voice has its own sequence of notes and
/// can have distinct attributes such as clef, stem direction, and transposition.
/// Voice declarations appear in the header with the `V:` field.
///
/// # Examples
///
/// ```text
/// V:1 name="Soprano" clef=treble
/// V:2 name="Alto" clef=alto stem=down
/// V:bass clef=bass octave=-1
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct VoiceDeclaration<'input> {
    /// The unique identifier for this voice.
    pub id: VoiceId<'input>,
    /// Optional attributes modifying the voice's behaviour and appearance.
    pub attributes: Option<VoiceAttributes<'input>>,
}

/// Attributes that modify a voice's behaviour and appearance.
///
/// Voice attributes control how the voice is displayed and performed,
/// including visual properties (clef, stem direction), performance
/// properties (transposition, MIDI instrument), and descriptive metadata
/// (name, subname).
#[derive(Debug, PartialEq, Eq)]
pub struct VoiceAttributes<'input> {
    /// The display name for this voice.
    pub name: Option<&'input str>,
    /// A secondary name or description.
    pub subname: Option<&'input str>,
    /// The clef to use for this voice.
    pub clef: Option<Clef>,
    /// The direction of note stems.
    pub stem: Option<StemDirection>,
    /// Octave shift for this voice (in octaves).
    ///
    /// Positive values shift up, negative values shift down.
    pub octave: Option<i8>,
    /// Transposition for this voice (in semitones).
    ///
    /// Positive values transpose up, negative values transpose down.
    pub transpose: Option<i8>,
    /// The instrument to use for this voice.
    pub instrument: Option<Instrument>,
    /// Voice-specific key signature override.
    pub key: Option<KeySignature>,
}

/// The direction of note stems in notation.
///
/// Stem direction affects the visual appearance of notes in traditional
/// notation. Stems normally point up for notes below the middle line and
/// down for notes above it, but can be forced to a specific direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StemDirection {
    /// Stems point upwards.
    Up,
    /// Stems point downwards.
    Down,
    /// Automatic stem direction based on note position (default).
    Auto,
}
