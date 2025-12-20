//! Error types for ABC notation parsing and processing.
//!
//! This module defines all error types that can occur during ABC file parsing,
//! validation, and processing. Each error variant includes specific context
//! information to aid in debugging and provide clear error messages to users.

use std::path::PathBuf;

use thiserror::Error;

use super::{Channel, MeasureNumber, PartLabel, ReferenceNumber, VoiceId, midi::Velocity};

/// The result type for ABC operations.
pub type Result<'a, T> = std::result::Result<T, AbcError<'a>>;

/// Errors that can occur during ABC notation parsing and processing.
#[derive(Error, Debug)]
pub enum AbcError<'a> {
    // I/O errors
    /// Failed to read the specified file.
    #[error("Failed to read file {path}")]
    FileNotFound {
        /// The path to the file that could not be found.
        path: PathBuf,
    },

    /// An I/O error occurred whilst reading from input.
    #[error("Failed to read from input: {source}")]
    IoError {
        /// The underlying I/O error.
        #[from]
        source: std::io::Error,
    },

    // Parse errors
    /// Invalid ABC syntax was encountered.
    #[error("Invalid ABC syntax at line {line}, column {column}: {message}")]
    ParseError {
        /// The line number where the error occurred (1-indexed).
        line: usize,
        /// The column number where the error occurred (1-indexed).
        column: usize,
        /// A description of what went wrong.
        message: String,
    },

    /// A required field is missing from a tune header.
    #[error("Missing required field {field} in tune {reference}")]
    MissingRequiredField {
        /// The name of the missing field (e.g., "X:", "T:", "K:").
        field: String,
        /// The reference number of the tune with the missing field.
        reference: ReferenceNumber,
    },

    /// An invalid key signature was specified.
    #[error("Invalid key signature: {input}")]
    InvalidKeySignature {
        /// The invalid key signature string.
        input: String,
    },

    /// An invalid metre (time signature) was specified.
    #[error("Invalid metre: {input}")]
    InvalidMeter {
        /// The invalid metre string.
        input: String,
    },

    /// An invalid tempo marking was specified.
    #[error("Invalid tempo marking: {input}")]
    InvalidTempo {
        /// The invalid tempo string.
        input: String,
    },

    // Semantic errors
    /// A voice was referenced but never defined.
    #[error("Undefined voice {voice_id} referenced in tune {reference}")]
    UndefinedVoice {
        /// The identifier of the undefined voice.
        voice_id: VoiceId<'a>,
        /// The reference number of the tune containing the invalid reference.
        reference: ReferenceNumber,
    },

    /// A voice overlay (`&` operator) was used without an active voice.
    #[error("Voice overlay without active voice at measure {measure}")]
    VoiceOverlayWithoutVoice {
        /// The measure number where the invalid overlay occurred.
        measure: MeasureNumber,
    },

    /// A tie was started but never ended, or vice versa.
    #[error("Unmatched tie at measure {measure}")]
    UnmatchedTie {
        /// The measure number where the unmatched tie was found.
        measure: MeasureNumber,
    },

    /// A slur was started but never ended, or vice versa.
    #[error("Unmatched slur at measure {measure}")]
    UnmatchedSlur {
        /// The measure number where the unmatched slur was found.
        measure: MeasureNumber,
    },

    /// An invalid tuplet specification was provided.
    #[error("Invalid tuplet specification: p={p}, q={q:?}, r={r:?}")]
    InvalidTuplet {
        /// The number of notes in the tuplet.
        p: u8,
        /// The number of notes they replace (optional).
        q: Option<u8>,
        /// The total duration they occupy (optional).
        r: Option<u32>,
    },

    /// A part was referenced in the part sequence but never defined in the tune body.
    #[error("Part {label} not defined in tune")]
    UndefinedPart {
        /// The label of the undefined part.
        label: PartLabel,
    },

    // MIDI errors
    /// An invalid MIDI channel number was specified.
    #[error("Invalid MIDI channel {channel} (must be 1-16)")]
    InvalidMidiChannel {
        /// The invalid channel number.
        channel: u8,
    },


    /// An invalid MIDI velocity value was specified.
    #[error("Invalid MIDI velocity {velocity} (must be 0-127)")]
    InvalidMidiVelocity {
        /// The invalid velocity value.
        velocity: u8,
    },

    // Value errors
    /// An invalid pitch value was encountered.
    #[error("Invalid pitch: {message}")]
    InvalidPitch {
        /// A description of why the pitch is invalid.
        message: String,
    },

    /// An invalid duration was specified.
    #[error("Invalid duration: {numerator}/{denominator}")]
    InvalidDuration {
        /// The duration numerator.
        numerator: u32,
        /// The duration denominator.
        denominator: u32,
    },

    /// An invalid octave modifier was applied, resulting in a pitch outside the valid MIDI range.
    #[error("Octave {octave} would result in pitch outside valid MIDI range for note {note}")]
    OctaveOutOfRange {
        /// The octave offset that was invalid.
        octave: i8,
        /// The base note being modified.
        note: String,
    },

    /// A macro was invoked but not defined.
    #[error("Undefined macro symbol '{symbol}' in tune {reference}")]
    UndefinedMacro {
        /// The macro symbol that was not defined.
        symbol: char,
        /// The reference number of the tune containing the invalid invocation.
        reference: ReferenceNumber,
    },

    /// A redefined symbol conflicts with existing notation.
    #[error("Cannot redefine reserved symbol '{symbol}'")]
    ReservedSymbolRedefinition {
        /// The reserved symbol that cannot be redefined.
        symbol: char,
    },
}

/// Additional constructors for common error scenarios.
impl<'a> AbcError<'a> {
    /// Creates a parse error with the given position and message.
    #[must_use]
    pub fn parse(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Creates an error for an invalid MIDI channel.
    ///
    /// Returns `None` if the channel is actually valid (1–16).
    #[must_use]
    pub fn invalid_channel(channel: u8) -> Option<Self> {
        if Channel::new(channel).is_none() {
            Some(Self::InvalidMidiChannel { channel })
        } else {
            None
        }
    }

    /// Creates an error for an invalid MIDI velocity.
    ///
    /// Returns `None` if the velocity is actually valid (0–127).
    #[must_use]
    pub fn invalid_velocity(velocity: u8) -> Option<Self> {
        if Velocity::new(velocity).is_none() {
            Some(Self::InvalidMidiVelocity { velocity })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_error_display() {
        let err = AbcError::parse(10, 5, "unexpected token");
        assert_eq!(
            err.to_string(),
            "Invalid ABC syntax at line 10, column 5: unexpected token"
        );
    }

    #[rstest]
    #[case(0, true)]
    #[case(17, true)]
    #[case(1, false)]
    #[case(16, false)]
    #[case(10, false)] // Percussion channel
    fn test_invalid_channel_helper(#[case] channel: u8, #[case] should_error: bool) {
        assert_eq!(AbcError::invalid_channel(channel).is_some(), should_error);
    }

    #[rstest]
    #[case(128, true)]
    #[case(255, true)]
    #[case(0, false)]
    #[case(127, false)]
    #[case(64, false)] // Mid-range velocity
    fn test_invalid_velocity_helper(#[case] velocity: u8, #[case] should_error: bool) {
        assert_eq!(AbcError::invalid_velocity(velocity).is_some(), should_error);
    }
}
