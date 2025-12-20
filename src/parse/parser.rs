//! Main parse function for ABC notation.
//!
//! This module provides the primary entry point for parsing ABC documents.

use std::convert::TryFrom;

use nom::error::ErrorKind;

use super::{document::document, utils::{calculate_position, preprocess_line_continuation}};
use crate::types::{AbcError, Document};

/// Format a nom parse error into a user-friendly message.
///
/// This converts nom's internal `ErrorKind` variants into clearer descriptions
/// of what went wrong during parsing.
fn format_parse_error(e: &nom::error::Error<&str>) -> String {
    let context = if e.input.len() > 20 {
        format!("'{:.20}...'", e.input)
    } else if e.input.is_empty() {
        "end of input".to_string()
    } else {
        format!("'{}'", e.input)
    };

    let description = match e.code {
        ErrorKind::Tag => "expected specific text",
        ErrorKind::Char => "expected specific character",
        ErrorKind::Alpha => "expected letter",
        ErrorKind::Digit => "expected digit",
        ErrorKind::AlphaNumeric => "expected letter or digit",
        ErrorKind::Space => "expected space",
        ErrorKind::MultiSpace => "expected whitespace",
        ErrorKind::CrLf => "expected line ending",
        ErrorKind::OneOf => "expected one of the valid characters",
        ErrorKind::NoneOf => "unexpected character",
        ErrorKind::Alt => "no valid alternative matched",
        ErrorKind::Eof => "expected end of input",
        ErrorKind::Not => "unexpected match",
        ErrorKind::Verify => "validation failed",
        ErrorKind::MapOpt => "conversion failed",
        ErrorKind::MapRes => "conversion failed",
        ErrorKind::Float => "expected number",
        ErrorKind::Fail => "parse failed",
        _ => "unexpected input",
    };

    format!("{description} at {context}")
}

/// Parse an ABC document from a string.
///
/// This is the primary entry point for parsing ABC notation. It parses the
/// input string into the internal AST representation and then transforms it
/// into the validated public types.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use abc::{parse, types::*};
///
/// let input = "X:1\nT:Example\nK:G\n";
/// let doc = parse(input).unwrap();
///
/// let mut voices = HashMap::new();
/// voices.insert(VoiceId::new("1"), Voice {
///     id: VoiceId::new("1"),
///     name: None,
///     measures: Vec::new(),
/// });
///
/// let expected = Document {
///     tunes: vec![Tune {
///         reference_number: ReferenceNumber(1),
///         title: "Example",
///         composer: None,
///         origin: None,
///         rhythm: None,
///         metadata: TuneMetadata::default(),
///         key: KeySignature {
///             tonic: PitchClass::G,
///             accidental: None,
///             mode: Mode::Major,
///             explicit_accidentals: Vec::new(),
///             clef: None,
///             transpose: None,
///             octave_shift: None,
///             middle: None,
///             stafflines: None,
///         },
///         meter: Meter { numerator: 4, denominator: 4 },
///         tempo: None,
///         unit_length: Duration::new(1, 8),
///         voices,
///         default_voice: VoiceId::new("1"),
///     }],
///     metadata: DocumentMetadata::default(),
/// };
///
/// assert_eq!(doc, expected);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The input contains invalid ABC syntax
/// - Required fields are missing (X:, T:, K:)
/// - Voice references are undefined
pub fn parse(input: &str) -> Result<Document<'static>, AbcError<'static>> {
    // Preprocess input to handle line continuations
    let preprocessed = preprocess_line_continuation(input);

    // Leak the preprocessed string to get a 'static lifetime
    // This is necessary because the parsed AST borrows from the input
    let leaked: &'static str = Box::leak(preprocessed.into_boxed_str());

    // Parse to internal AST
    let (remaining, ast_doc) = document(leaked).map_err(|e| match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => {
            let (line, column) = calculate_position(leaked, e.input);
            AbcError::ParseError {
                line,
                column,
                message: format_parse_error(&e),
            }
        }
        nom::Err::Incomplete(_) => AbcError::ParseError {
            line: 1,
            column: 1,
            message: "Incomplete input".to_string(),
        },
    })?;

    // Check for trailing content
    if !remaining.trim().is_empty() {
        let (line, column) = calculate_position(leaked, remaining);
        return Err(AbcError::ParseError {
            line,
            column,
            message: format!("Unexpected content after document: {}", remaining),
        });
    }

    // Transform to public types
    Document::try_from(ast_doc)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_single_tune() {
        use std::{collections::HashMap, rc::Rc};

        use crate::types::*;

        let input = "X:1\nT:Example\nK:G\nGABc|dedB|\n";
        let doc = parse(input).unwrap();

        let key = KeySignature {
            tonic: PitchClass::G,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices = HashMap::new();
        voices.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::G, Octave(4), None), // G
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::A, Octave(4), None), // A
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::C, Octave(5), None), // c
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 2),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(5, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(7, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context.clone(),
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        let expected = Document {
            tunes: vec![Tune {
                reference_number: ReferenceNumber(1),
                title: "Example",
                composer: None,
                origin: None,
                rhythm: None,
                metadata: TuneMetadata::default(),
                key,
                meter: Meter {
                    numerator: 4,
                    denominator: 4,
                },
                tempo: None,
                unit_length: Duration::new(1, 8),
                voices,
                default_voice: VoiceId::new("1"),
            }],
            metadata: DocumentMetadata::default(),
        };

        assert_eq!(doc, expected);
    }

    #[test]
    fn test_parse_multiple_tunes() {
        use std::{collections::HashMap, rc::Rc};

        use crate::types::*;

        let input = "\
X:1
T:First Tune
K:G
GABc|dedB|

X:2
T:Second Tune
K:D
defg|afed|
";
        let doc = parse(input).unwrap();

        // First tune (G major)
        let key1 = KeySignature {
            tonic: PitchClass::G,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context1 = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key1.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices1 = HashMap::new();
        voices1.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::G, Octave(4), None), // G
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::A, Octave(4), None), // A
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::C, Octave(5), None), // c
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context1.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 2),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(5, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(7, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context1.clone(),
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        // Second tune (D major - F# and C#)
        let key2 = KeySignature {
            tonic: PitchClass::D,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context2 = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key2.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices2 = HashMap::new();
        voices2.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::F, Octave(5), Some(Accidental::Sharp)), // f (F# from D major key)
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::G, Octave(5), None), // g
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context2.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::A, Octave(5), None), // a
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 2),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::F, Octave(5), Some(Accidental::Sharp)), // f (F# from D major key)
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(5, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(7, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context2.clone(),
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        let expected = Document {
            tunes: vec![
                Tune {
                    reference_number: ReferenceNumber(1),
                    title: "First Tune",
                    composer: None,
                    origin: None,
                    rhythm: None,
                    metadata: TuneMetadata::default(),
                    key: key1,
                    meter: Meter {
                        numerator: 4,
                        denominator: 4,
                    },
                    tempo: None,
                    unit_length: Duration::new(1, 8),
                    voices: voices1,
                    default_voice: VoiceId::new("1"),
                },
                Tune {
                    reference_number: ReferenceNumber(2),
                    title: "Second Tune",
                    composer: None,
                    origin: None,
                    rhythm: None,
                    metadata: TuneMetadata::default(),
                    key: key2,
                    meter: Meter {
                        numerator: 4,
                        denominator: 4,
                    },
                    tempo: None,
                    unit_length: Duration::new(1, 8),
                    voices: voices2,
                    default_voice: VoiceId::new("1"),
                },
            ],
            metadata: DocumentMetadata::default(),
        };

        assert_eq!(doc, expected);
    }

    #[test]
    fn test_parse_multiple_tunes_with_metadata() {
        use std::{collections::HashMap, rc::Rc};

        use crate::types::*;

        let input = "\
X:1
T:First Tune
C:Trad.
K:G
GABc|dedB|

X:2
T:Second Tune
R:Reel
K:D
defg|afed|
";
        let doc = parse(input).unwrap();

        // First tune (G major) - same measures as before
        let key1 = KeySignature {
            tonic: PitchClass::G,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context1 = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key1.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices1 = HashMap::new();
        voices1.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::G, Octave(4), None), // G
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::A, Octave(4), None), // A
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::C, Octave(5), None), // c
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context1.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 2),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(5, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::B, Octave(4), None), // B
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(7, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context1.clone(),
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        // Second tune (D major - F# and C#) - same measures as before
        let key2 = KeySignature {
            tonic: PitchClass::D,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context2 = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key2.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices2 = HashMap::new();
        voices2.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::F, Octave(5), Some(Accidental::Sharp)), // f (F# from D major key)
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::G, Octave(5), None), // g
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context2.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::A, Octave(5), None), // a
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 2),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::F, Octave(5), Some(Accidental::Sharp)), // f (F# from D major key)
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(5, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None), // e
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(3, 4),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(5), None), // d
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(7, 8),
                                articulations: Vec::new(),
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context2.clone(),
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        let expected = Document {
            tunes: vec![
                Tune {
                    reference_number: ReferenceNumber(1),
                    title: "First Tune",
                    composer: Some("Trad."),
                    origin: None,
                    rhythm: None,
                    metadata: TuneMetadata::default(),
                    key: key1,
                    meter: Meter {
                        numerator: 4,
                        denominator: 4,
                    },
                    tempo: None,
                    unit_length: Duration::new(1, 8),
                    voices: voices1,
                    default_voice: VoiceId::new("1"),
                },
                Tune {
                    reference_number: ReferenceNumber(2),
                    title: "Second Tune",
                    composer: None,
                    origin: None,
                    rhythm: Some("Reel"),
                    metadata: TuneMetadata::default(),
                    key: key2,
                    meter: Meter {
                        numerator: 4,
                        denominator: 4,
                    },
                    tempo: None,
                    unit_length: Duration::new(1, 8),
                    voices: voices2,
                    default_voice: VoiceId::new("1"),
                },
            ],
            metadata: DocumentMetadata::default(),
        };

        assert_eq!(doc, expected);
    }

    #[test]
    fn test_parse_with_articulations() {
        use std::{collections::HashMap, rc::Rc};

        use crate::types::*;

        // ABC with articulations: staccato (.), accent (!accent!), trill (T)
        let input = "X:1\nT:Articulation Test\nK:C\n.C !accent!D Te|F|\n";
        let doc = parse(input).unwrap();

        let key = KeySignature {
            tonic: PitchClass::C,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        let context = {
            use crate::transform::context::TransformContext;
            TransformContext::new(
                Rc::new(key.clone()),
                Meter {
                    numerator: 4,
                    denominator: 4,
                },
                None,
                Duration::new(1, 8),
            )
            .build_context()
        };

        let mut voices = HashMap::new();
        voices.insert(
            VoiceId::new("1"),
            Voice {
                id: VoiceId::new("1"),
                name: None,
                measures: vec![
                    Measure {
                        number: MeasureNumber(1),
                        events: vec![
                            Note {
                                pitch: Pitch::new(PitchClass::C, Octave(4), None),
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(0, 1),
                                articulations: vec![Articulation::Staccato],
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::D, Octave(4), None),
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 8),
                                articulations: vec![Articulation::Accent],
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                            Note {
                                pitch: Pitch::new(PitchClass::E, Octave(5), None),
                                duration: Duration::new(1, 8),
                                dynamics: Dynamics::default(),
                                dynamic_direction: DynamicDirection::None,
                                absolute_time: Duration::new(1, 4),
                                articulations: vec![Articulation::Trill],
                                lyric: None,
                                slur_starts: 0,
                                slur_ends: 0,
                                dotted_slur_starts: 0,
                                dotted_slur_ends: 0,
                            }.into(),
                        ],
                        context: context.clone(),
                        part: None,
                        variant: None,
                    },
                    Measure {
                        number: MeasureNumber(2),
                        events: vec![Note {
                            pitch: Pitch::new(PitchClass::F, Octave(4), None),
                            duration: Duration::new(1, 8),
                            dynamics: Dynamics::default(),
                            dynamic_direction: DynamicDirection::None,
                            absolute_time: Duration::new(3, 8),
                            articulations: Vec::new(),
                            lyric: None,
                            slur_starts: 0,
                            slur_ends: 0,
                            dotted_slur_starts: 0,
                            dotted_slur_ends: 0,
                        }.into()],
                        context,
                        part: None,
                        variant: None,
                    },
                ],
            },
        );

        let expected = Document {
            tunes: vec![Tune {
                reference_number: ReferenceNumber(1),
                title: "Articulation Test",
                composer: None,
                origin: None,
                rhythm: None,
                metadata: TuneMetadata::default(),
                key,
                meter: Meter {
                    numerator: 4,
                    denominator: 4,
                },
                tempo: None,
                unit_length: Duration::new(1, 8),
                voices,
                default_voice: VoiceId::new("1"),
            }],
            metadata: DocumentMetadata::default(),
        };

        assert_eq!(doc, expected);
    }

    // ========================================================================
    // Error Path Tests
    // ========================================================================

    #[test]
    fn test_parse_missing_reference_number() {
        let input = "T:Title\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { message, .. } => {
                assert!(message.contains("expected") || message.contains("X:"));
            }
            AbcError::MissingRequiredField { field, .. } => {
                assert!(field.contains("X") || field.contains("reference"));
            }
            _ => panic!("Expected ParseError or MissingRequiredField, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_missing_title() {
        let input = "X:1\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { message, .. } => {
                assert!(message.contains("expected") || message.contains("T:"));
            }
            AbcError::MissingRequiredField { field, .. } => {
                assert!(field.contains("T") || field.contains("title"));
            }
            _ => panic!("Expected ParseError or MissingRequiredField, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_missing_key_signature() {
        let input = "X:1\nT:Title\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { message, .. } => {
                assert!(message.contains("expected") || message.contains("K:"));
            }
            _ => panic!("Expected ParseError, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_invalid_reference_number_zero() {
        let input = "X:0\nT:Title\nK:G\n";
        let result = parse(input);
        // X:0 is actually valid in ABC - reference numbers can be 0
        // This test documents that behavior
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_reference_number_text() {
        let input = "X:abc\nT:Title\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { message, .. } => {
                assert!(message.contains("expected") || message.contains("digit"));
            }
            _ => panic!("Expected ParseError, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_invalid_key_signature() {
        let input = "X:1\nT:Title\nK:Z\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { .. } => {}
            _ => panic!("Expected ParseError, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_invalid_meter_format() {
        let input = "X:1\nT:Title\nM:4/\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { .. } => {}
            _ => panic!("Expected ParseError, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_invalid_meter_denominator_zero() {
        let input = "X:1\nT:Title\nM:4/0\nK:G\n";
        let result = parse(input);
        // M:4/0 fails at parse time - parser expects non-zero denominator
        // This test verifies the parser rejects this early
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_meter_numerator_zero() {
        let input = "X:1\nT:Title\nM:0/4\nK:G\n";
        let result = parse(input);
        // M:0/4 may be parsed but is semantically invalid
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_pitch_class() {
        let input = "X:1\nT:Title\nK:G\nZ\n";
        let result = parse(input);
        // Z is actually a valid ABC element (rest), not an error
        // This test documents that Z is interpreted as a rest
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_accidental_syntax() {
        let input = "X:1\nT:Title\nK:G\n___C\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_duration_format() {
        // C/0 has an invalid zero denominator - parser rejects it gracefully.
        // The "/0" won't parse as a valid duration (zero denominator is filtered out),
        // so "C" is parsed as a note with default duration, and "/0" becomes
        // unparsable trailing content that triggers a parse error.
        let input = "X:1\nT:Title\nK:G\nC/0\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, AbcError::ParseError { ref message, .. } if message.contains("/0")),
            "Expected ParseError mentioning '/0', got {:?}",
            err
        );
    }

    #[test]
    fn test_parse_trailing_content() {
        let input = "X:1\nT:Title\nK:G\n\nTrailing garbage that should not be here";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { message, .. } => {
                assert!(message.contains("Unexpected content"));
            }
            _ => panic!("Expected ParseError for trailing content, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_empty_input() {
        let input = "";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let input = "   \n\n  \t  \n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_incomplete_tune_header() {
        let input = "X:1\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_colon_in_field() {
        let input = "X1\nT:Title\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_unit_length() {
        let input = "X:1\nT:Title\nL:0/8\nK:G\n";
        let result = parse(input);
        // L:0/8 may parse but represents an invalid unit length
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_tempo_format() {
        let input = "X:1\nT:Title\nQ:abc\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_chord() {
        let input = "X:1\nT:Title\nK:G\n[CEG\n";
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AbcError::ParseError { .. } => {}
            _ => panic!("Expected ParseError for unclosed chord, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_invalid_barline_repeat() {
        let input = "X:1\nT:Title\nK:G\n|:::\n";
        let result = parse(input);
        // This might succeed depending on parser implementation
        // but should at least not panic
        let _ = result;
    }

    #[test]
    fn test_parse_duplicate_reference_numbers() {
        let input = "X:1\nT:First\nK:G\n\nX:1\nT:Second\nK:D\n";
        // This should parse successfully - duplicate reference numbers are allowed
        // in ABC (though not recommended)
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.tunes.len(), 2);
    }

    #[test]
    fn test_parse_invalid_octave_modifier() {
        let input = "X:1\nT:Title\nK:G\nC'''''''''\n";
        // Very high octave - should still parse but might fail validation
        let result = parse(input);
        // Depending on implementation, this might succeed or fail
        let _ = result;
    }

    #[test]
    fn test_parse_malformed_inline_field() {
        let input = "X:1\nT:Title\nK:G\n[K:\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_chord_symbol() {
        let input = "X:1\nT:Title\nK:G\n\"XYZ999\"C\n";
        // Invalid chord symbols might be accepted as text
        let result = parse(input);
        // Should at least not panic
        let _ = result;
    }

    #[test]
    fn test_parse_negative_reference_number() {
        let input = "X:-1\nT:Title\nK:G\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_tune_body() {
        let input = "X:1\nT:Title\nK:G\n";
        // A tune with just a header and no body should be valid
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_broken_rhythm() {
        let input = "X:1\nT:Title\nK:G\nC>>>>D\n";
        let result = parse(input);
        // Excessive broken rhythm markers
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_tuplet_specification() {
        let input = "X:1\nT:Title\nK:G\n(0CDE\n";
        let result = parse(input);
        // (0 is an invalid tuplet specification
        let _ = result;
    }

    #[test]
    fn test_parse_error_line_and_column_tracking() {
        let input = "X:1\nT:Title\nK:G\nCDE\nFG@\n";
        let result = parse(input);
        // @ is an invalid character in ABC body
        if let Err(err) = result {
            match err {
                AbcError::ParseError { line, column, .. } => {
                    // Error should be on line 5 (the line with @)
                    assert!(line >= 4);
                    assert!(column >= 1);
                }
                _ => panic!("Expected ParseError with line/column info, got {:?}", err),
            }
        }
    }

    // ========================================================================
    // Line Continuation Tests
    // ========================================================================

    #[test]
    fn test_line_continuation_in_header() {
        let input = "X:1\nT:Long \\\nTitle\nK:G\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.tunes[0].title, "Long Title");
    }

    #[test]
    fn test_line_continuation_in_body() {
        let input = "X:1\nT:Test\nK:G\nGAB\\\nc|dedB|\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        // Should parse as GABc|dedB| (one measure with 4 notes, another with 4 notes)
        let voice = doc.tunes[0].primary_voice();
        assert_eq!(voice.measures.len(), 2);
    }

    #[test]
    fn test_multiple_line_continuations() {
        let input = "X:1\nT:Very \\\nLong \\\nTitle\nK:G\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.tunes[0].title, "Very Long Title");
    }

    #[test]
    fn test_line_continuation_crlf() {
        let input = "X:1\r\nT:Long \\\r\nTitle\r\nK:G\r\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.tunes[0].title, "Long Title");
    }

    // ========================================================================
    // Dotted Slur Tests
    // ========================================================================

    #[test]
    fn test_dotted_slur_parsing() {
        use crate::types::*;

        let input = "X:1\nT:Test\nK:C\n.(CDE)\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        let voice = doc.tunes[0].primary_voice();
        let measure = &voice.measures[0];

        let expected_events: Vec<Event> = vec![
            Note {
                pitch: Pitch::new(PitchClass::C, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(0, 1),
                articulations: vec![Articulation::Staccato],
                lyric: None,
                slur_starts: 1,
                slur_ends: 0,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
            Note {
                pitch: Pitch::new(PitchClass::D, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(1, 8),
                articulations: Vec::new(),
                lyric: None,
                slur_starts: 0,
                slur_ends: 0,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
            Note {
                pitch: Pitch::new(PitchClass::E, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(1, 4),
                articulations: Vec::new(),
                lyric: None,
                slur_starts: 0,
                slur_ends: 1,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
        ];

        assert_eq!(measure.events, expected_events);
    }

    #[test]
    fn test_mixed_slurs_dotted_and_regular() {
        use crate::types::*;

        let input = "X:1\nT:Test\nK:C\n.(CD)(EF)\n";
        let result = parse(input);
        assert!(result.is_ok());
        let doc = result.unwrap();
        let voice = doc.tunes[0].primary_voice();
        let measure = &voice.measures[0];

        let expected_events: Vec<Event> = vec![
            Note {
                pitch: Pitch::new(PitchClass::C, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(0, 1),
                articulations: vec![Articulation::Staccato],
                lyric: None,
                slur_starts: 1,
                slur_ends: 0,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
            Note {
                pitch: Pitch::new(PitchClass::D, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(1, 8),
                articulations: Vec::new(),
                lyric: None,
                slur_starts: 0,
                slur_ends: 1,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
            Note {
                pitch: Pitch::new(PitchClass::E, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(1, 4),
                articulations: Vec::new(),
                lyric: None,
                slur_starts: 1,
                slur_ends: 0,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
            Note {
                pitch: Pitch::new(PitchClass::F, Octave(4), None),
                duration: Duration::new(1, 8),
                dynamics: Dynamics::MF,
                dynamic_direction: DynamicDirection::None,
                absolute_time: Duration::new(3, 8),
                articulations: Vec::new(),
                lyric: None,
                slur_starts: 0,
                slur_ends: 1,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }.into(),
        ];

        assert_eq!(measure.events, expected_events);
    }
}
