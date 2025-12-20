//! Note and rest parsers.
//!
//! This module provides parsers for notes and rests in ABC notation,
//! building on the primitive parsers for pitch, accidental, octave,
//! and duration.

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::{char, u32 as parse_u32},
    combinator::{map, not, opt, peek},
    error::context,
    sequence::preceded,
};

use crate::{
    parse::{
        primitives::{accidental, duration_modifier, octave_shift, pitch_class_with_case},
        rhythm::broken_rhythm,
    },
    types::{
        Octave,
        ast::{Note, NoteDuration, NotePitch, Rest},
    },
};

/// Parse a note pitch (base, accidental, octave).
///
/// A note pitch consists of:
/// 1. Optional accidental (^, _, =, ^^, __)
/// 2. Pitch class letter (C-B, case indicates octave)
/// 3. Optional octave modifiers (' or ,)
///
/// The case of the pitch class letter indicates the base octave:
/// - Uppercase (C-B) represents the octave from middle C to B above
/// - Lowercase (c-b) represents the octave from C above middle C upward
pub(crate) fn note_pitch(input: &str) -> IResult<&str, NotePitch> {
    context(
        "note pitch",
        map(
            (opt(accidental), pitch_class_with_case, octave_shift),
            |(acc, (base, is_lowercase), octave)| {
                // Adjust octave based on case: lowercase letters are one octave higher
                let case_offset = if is_lowercase { 1 } else { 0 };

                NotePitch {
                    base,
                    accidental: acc,
                    octave: Octave(octave.0 + case_offset),
                }
            },
        ),
    )
    .parse(input)
}

/// Parse a note duration specification.
///
/// Duration can be:
/// - Omitted (uses default unit note length): `C`
/// - Explicit multiplier: `C2` (double), `C3` (triple)
/// - Explicit division: `C/2` (half), `C/` (half)
/// - Multiple divisions: `C//` (quarter), `C///` (eighth)
/// - Fractional: `C3/2` (1.5×)
///
/// Note: This parser always succeeds, returning `NoteDuration::Default`
/// when no duration modifier is present.
pub(crate) fn note_duration(input: &str) -> IResult<&str, NoteDuration> {
    context(
        "note duration",
        map(opt(duration_modifier), |dur| match dur {
            None => NoteDuration::Default,
            Some(d) => NoteDuration::Explicit {
                numerator: d.numerator(),
                denominator: d.denominator(),
            },
        }),
    )
    .parse(input)
}

/// Parse a complete note.
///
/// A note consists of:
/// 1. Note pitch (optional accidental + pitch class + optional octave)
/// 2. Optional duration modifier
/// 3. Optional broken rhythm operator
///
/// Decorations are parsed separately and attached later.
pub(crate) fn note(input: &str) -> IResult<&str, Note<'_>> {
    context(
        "note",
        map(
            (note_pitch, note_duration, opt(broken_rhythm)),
            |(pitch, duration, broken_rhythm)| Note {
                pitch,
                duration,
                decorations: Vec::new(), // Decorations added separately
                broken_rhythm,
            },
        ),
    )
    .parse(input)
}

/// Parse a rest.
///
/// ABC notation supports several types of rests:
/// - `z` with optional duration: visible single-note rest (e.g., `z`, `z2`, `z/2`)
/// - `x` with optional duration: invisible single-note rest (spacing)
/// - `Z` with optional measure count: visible multi-measure rest (e.g., `Z`, `Z4`)
/// - `X` with optional measure count: invisible multi-measure rest
///
/// Note: `Z:` and `X:` are information fields (transcription and reference number),
/// not rests, so we use negative lookahead to avoid consuming them.
pub(crate) fn rest(input: &str) -> IResult<&str, Rest> {
    context(
        "rest",
        alt((
            // Multi-measure rests: Z or X followed by optional measure count
            // Use negative lookahead for ':' to avoid consuming Z: or X: information fields
            map(
                preceded((char('Z'), not(peek(char(':')))), opt(parse_u32)),
                |count| Rest::MultiMeasure {
                    measures: count.unwrap_or(1),
                    visible: true,
                },
            ),
            map(
                preceded((char('X'), not(peek(char(':')))), opt(parse_u32)),
                |count| Rest::MultiMeasure {
                    measures: count.unwrap_or(1),
                    visible: false,
                },
            ),
            // Single-beat rests: z or x followed by optional duration
            map(preceded(char('z'), note_duration), |duration| {
                Rest::Visible(duration)
            }),
            map(preceded(char('x'), note_duration), |duration| {
                Rest::Invisible(duration)
            }),
        )),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::types::{Accidental, PitchClass};

    #[rstest]
    #[case("C", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }))]
    #[case("D", ("", NotePitch { base: PitchClass::D, accidental: None, octave: Octave(0) }))]
    #[case("c", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(1) }))]
    #[case("d", ("", NotePitch { base: PitchClass::D, accidental: None, octave: Octave(1) }))]
    #[case("^F", ("", NotePitch { base: PitchClass::F, accidental: Some(Accidental::Sharp), octave: Octave(0) }))]
    #[case("_B", ("", NotePitch { base: PitchClass::B, accidental: Some(Accidental::Flat), octave: Octave(0) }))]
    #[case("=A", ("", NotePitch { base: PitchClass::A, accidental: Some(Accidental::Natural), octave: Octave(0) }))]
    #[case("^^G", ("", NotePitch { base: PitchClass::G, accidental: Some(Accidental::DoubleSharp), octave: Octave(0) }))]
    #[case("__E", ("", NotePitch { base: PitchClass::E, accidental: Some(Accidental::DoubleFlat), octave: Octave(0) }))]
    #[case("C'", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(1) }))]
    #[case("C''", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(2) }))]
    #[case("C,", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(-1) }))]
    #[case("C,,", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(-2) }))]
    #[case("c'", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(2) }))]
    #[case("c,", ("", NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }))]
    fn test_note_pitch(#[case] input: &str, #[case] expected: (&str, NotePitch)) {
        let result = note_pitch(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("C", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Default, decorations: vec![], broken_rhythm: None }))]
    #[case("C2", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Explicit { numerator: 2, denominator: 1 }, decorations: vec![], broken_rhythm: None }))]
    #[case("C/", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Explicit { numerator: 1, denominator: 2 }, decorations: vec![], broken_rhythm: None }))]
    #[case("C/2", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Explicit { numerator: 1, denominator: 2 }, decorations: vec![], broken_rhythm: None }))]
    #[case("C//", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Explicit { numerator: 1, denominator: 4 }, decorations: vec![], broken_rhythm: None }))]
    #[case("C3/2", ("", Note { pitch: NotePitch { base: PitchClass::C, accidental: None, octave: Octave(0) }, duration: NoteDuration::Explicit { numerator: 3, denominator: 2 }, decorations: vec![], broken_rhythm: None }))]
    fn test_note_with_duration(#[case] input: &str, #[case] expected: (&str, Note)) {
        let result = note(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complete_notes() {
        // Simple note
        let result = note("C").unwrap();
        assert_eq!(
            result,
            (
                "",
                Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0)
                    },
                    duration: NoteDuration::Default,
                    decorations: vec![],
                    broken_rhythm: None,
                }
            )
        );

        // Sharp note with duration
        let result = note("^F2").unwrap();
        assert_eq!(
            result,
            (
                "",
                Note {
                    pitch: NotePitch {
                        base: PitchClass::F,
                        accidental: Some(Accidental::Sharp),
                        octave: Octave(0)
                    },
                    duration: NoteDuration::Explicit {
                        numerator: 2,
                        denominator: 1
                    },
                    decorations: vec![],
                    broken_rhythm: None,
                }
            )
        );

        // High note with half duration
        let result = note("c'/").unwrap();
        assert_eq!(
            result,
            (
                "",
                Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(2)
                    },
                    duration: NoteDuration::Explicit {
                        numerator: 1,
                        denominator: 2
                    },
                    decorations: vec![],
                    broken_rhythm: None,
                }
            )
        );
    }

    #[rstest]
    // Visible single-beat rests (z)
    #[case("z", ("", Rest::Visible(NoteDuration::Default)))]
    #[case("z2", ("", Rest::Visible(NoteDuration::Explicit { numerator: 2, denominator: 1 })))]
    #[case("z/", ("", Rest::Visible(NoteDuration::Explicit { numerator: 1, denominator: 2 })))]
    #[case("z4", ("", Rest::Visible(NoteDuration::Explicit { numerator: 4, denominator: 1 })))]
    // Invisible single-beat rests (x)
    #[case("x", ("", Rest::Invisible(NoteDuration::Default)))]
    #[case("x2", ("", Rest::Invisible(NoteDuration::Explicit { numerator: 2, denominator: 1 })))]
    #[case("x/", ("", Rest::Invisible(NoteDuration::Explicit { numerator: 1, denominator: 2 })))]
    // Multi-measure visible rests (Z)
    #[case("Z", ("", Rest::MultiMeasure { measures: 1, visible: true }))]
    #[case("Z4", ("", Rest::MultiMeasure { measures: 4, visible: true }))]
    #[case("Z16", ("", Rest::MultiMeasure { measures: 16, visible: true }))]
    // Multi-measure invisible rests (X)
    #[case("X", ("", Rest::MultiMeasure { measures: 1, visible: false }))]
    #[case("X4", ("", Rest::MultiMeasure { measures: 4, visible: false }))]
    fn test_rest(#[case] input: &str, #[case] expected: (&str, Rest)) {
        let result = rest(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_note_with_remaining() {
        let result = note("C2D").unwrap();
        assert_eq!(
            result,
            (
                "D",
                Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0)
                    },
                    duration: NoteDuration::Explicit {
                        numerator: 2,
                        denominator: 1
                    },
                    decorations: vec![],
                    broken_rhythm: None,
                }
            )
        );
    }

    #[test]
    fn test_rest_does_not_match_info_fields() {
        // X: and Z: are information fields (reference number and transcription),
        // not multi-measure rests. Ensure the rest parser doesn't consume them.
        assert!(rest("X:1").is_err());
        assert!(rest("Z:transcriber").is_err());
    }
}
