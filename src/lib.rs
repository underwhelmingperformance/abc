//! ABC notation parser and MIDI generator.
//!
//! This library provides comprehensive support for parsing ABC notation (v2.1)
//! and generating MIDI output. ABC notation is a text-based music notation
//! system designed for notating folk and traditional music, though it has
//! evolved to support complex classical scores as well.
//!
//! # Examples
//!
//! Parse an ABC tune:
//!
//! ```
//! use std::collections::HashMap;
//! use abc::{parse, types::*};
//!
//! let input = "X:1\nT:Example Tune\nR:Jig\nK:G\n";
//! let doc = parse(input).unwrap();
//!
//! let mut expected_voices = HashMap::new();
//! expected_voices.insert(
//!     VoiceId::new("1"),
//!     Voice::builder().id(VoiceId::new("1")).build(),
//! );
//!
//! let expected = Document::builder()
//!     .tunes(vec![
//!         Tune::builder()
//!             .reference_number(ReferenceNumber(1))
//!             .title("Example Tune")
//!             .rhythm("Jig")
//!             .key(KeySignature::builder().tonic(PitchClass::G).build())
//!             .unit_length(Duration::new(1, 8))
//!             .voices(expected_voices)
//!             .default_voice(VoiceId::new("1"))
//!             .build()
//!     ])
//!     .build();
//!
//! assert_eq!(doc, expected);
//! ```

pub mod types;

pub(crate) mod parse;
pub(crate) mod transform;

// Re-export the main parse function at crate root
pub use parse::parse;
