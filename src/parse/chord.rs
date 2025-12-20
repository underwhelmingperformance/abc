//! Chord parsers.
//!
//! This module provides parsers for chords in ABC notation, including both
//! note chords (multiple simultaneous notes) and guitar chord symbols.

use nom::{
    IResult, Parser,
    bytes::complete::{is_not, take_till},
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::many1,
    sequence::{delimited, preceded},
};

use crate::{
    parse::{note::note, primitives::duration_modifier},
    types::ast::{Chord, GuitarChord, NoteDuration},
};

/// Parse a note chord.
///
/// Note chords are multiple notes played simultaneously, written in square
/// brackets. Each note can have its own accidental, octave, and duration.
/// The entire chord can also have a duration applied after the closing bracket.
pub(crate) fn note_chord(input: &str) -> IResult<&str, Chord<'_>> {
    context(
        "note chord",
        map(
            (
                delimited(char('['), many1(note), char(']')),
                opt(duration_modifier),
            ),
            |(notes, dur_mod)| Chord {
                notes,
                duration: dur_mod.map(|d| NoteDuration::Explicit {
                    numerator: d.numerator(),
                    denominator: d.denominator(),
                }),
            },
        ),
    )
    .parse(input)
}

/// Parse a guitar chord symbol.
///
/// Guitar chords are written in double quotes and represent chord names
/// rather than specific notes. They can include a bass note after a slash.
pub(crate) fn guitar_chord(input: &str) -> IResult<&str, GuitarChord<'_>> {
    context(
        "guitar chord",
        delimited(
            char('"'),
            cut(map(
                (
                    take_until_slash_or_quote,
                    opt(preceded(char('/'), take_until_quote)),
                ),
                |(symbol, bass_note)| GuitarChord { symbol, bass_note },
            )),
            cut(char('"')),
        ),
    )
    .parse(input)
}

/// Helper: Parse until '/' or '"'.
fn take_until_slash_or_quote(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '/' || c == '"')(input)
}

/// Helper: Parse until '"'.
fn take_until_quote(input: &str) -> IResult<&str, &str> {
    is_not("\"")(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::types::{
        Accidental, Octave, PitchClass,
        ast::{Note, NoteDuration, NotePitch},
    };

    #[test]
    fn test_simple_chord() {
        let result = note_chord("[CEG]").unwrap();
        assert_eq!(
            result,
            (
                "",
                Chord {
                    notes: vec![
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::C,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::E,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::G,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                    ],
                    duration: None,
                }
            )
        );
    }

    #[test]
    fn test_chord_with_accidentals() {
        let result = note_chord("[^FAc]").unwrap();
        assert_eq!(
            result,
            (
                "",
                Chord {
                    notes: vec![
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::F,
                                accidental: Some(Accidental::Sharp),
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::A,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::C,
                                accidental: None,
                                octave: Octave(1)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                    ],
                    duration: None,
                }
            )
        );
    }

    #[test]
    fn test_chord_with_durations() {
        let result = note_chord("[C2E2G2]").unwrap();
        assert_eq!(
            result,
            (
                "",
                Chord {
                    notes: vec![
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
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::E,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Explicit {
                                numerator: 2,
                                denominator: 1
                            },
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::G,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Explicit {
                                numerator: 2,
                                denominator: 1
                            },
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                    ],
                    duration: None,
                }
            )
        );
    }

    #[rstest]
    #[case("\"C\"", ("", GuitarChord { symbol: "C", bass_note: None }))]
    #[case("\"Am\"", ("", GuitarChord { symbol: "Am", bass_note: None }))]
    #[case("\"G7\"", ("", GuitarChord { symbol: "G7", bass_note: None }))]
    #[case("\"Cmaj7\"", ("", GuitarChord { symbol: "Cmaj7", bass_note: None }))]
    #[case("\"D/F#\"", ("", GuitarChord { symbol: "D", bass_note: Some("F#") }))]
    #[case("\"C/G\"", ("", GuitarChord { symbol: "C", bass_note: Some("G") }))]
    fn test_guitar_chord(#[case] input: &str, #[case] expected: (&str, GuitarChord)) {
        let result = guitar_chord(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_chord_with_trailing() {
        let result = note_chord("[CEG]ABC").unwrap();
        assert_eq!(
            result,
            (
                "ABC",
                Chord {
                    notes: vec![
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::C,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::E,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::G,
                                accidental: None,
                                octave: Octave(0)
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                    ],
                    duration: None,
                }
            )
        );
    }
}
