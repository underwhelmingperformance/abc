//! Grace note parsers.
//!
//! This module provides parsers for grace notes in ABC notation. Grace notes
//! are ornamental notes played quickly before a main note.

use nom::{
    IResult, Parser,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::many1,
    sequence::delimited,
};

use crate::{parse::note::note, types::ast::GraceNotes};

/// Parse grace notes.
///
/// Grace notes are written in curly braces and can be regular (`{AB}`) or
/// acciaccaturas (`{/AB}`), which are played very quickly.
pub(crate) fn grace_notes(input: &str) -> IResult<&str, GraceNotes<'_>> {
    context(
        "grace notes",
        delimited(
            char('{'),
            cut(map((opt(char('/')), many1(note)), |(slash, notes)| {
                GraceNotes {
                    notes,
                    acciaccatura: slash.is_some(),
                }
            })),
            cut(char('}')),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::{
        Octave, PitchClass,
        ast::{Note, NoteDuration, NotePitch},
    };

    #[test]
    fn test_simple_grace_notes() {
        let result = grace_notes("{AB}").unwrap();
        assert_eq!(
            result,
            (
                "",
                GraceNotes {
                    notes: vec![
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::A,
                                accidental: None,
                                octave: Octave(0),
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                        Note {
                            pitch: NotePitch {
                                base: PitchClass::B,
                                accidental: None,
                                octave: Octave(0),
                            },
                            duration: NoteDuration::Default,
                            decorations: vec![],
                            broken_rhythm: None,
                        },
                    ],
                    acciaccatura: false,
                }
            )
        );
    }

    #[test]
    fn test_acciaccatura() {
        let result = grace_notes("{/A}").unwrap();
        assert_eq!(
            result,
            (
                "",
                GraceNotes {
                    notes: vec![Note {
                        pitch: NotePitch {
                            base: PitchClass::A,
                            accidental: None,
                            octave: Octave(0),
                        },
                        duration: NoteDuration::Default,
                        decorations: vec![],
                        broken_rhythm: None,
                    }],
                    acciaccatura: true,
                }
            )
        );
    }

    #[test]
    fn test_grace_notes_with_trailing() {
        let result = grace_notes("{CD}EFG").unwrap();
        let expected = GraceNotes {
            notes: vec![
                Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: vec![],
                    broken_rhythm: None,
                },
                Note {
                    pitch: NotePitch {
                        base: PitchClass::D,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: vec![],
                    broken_rhythm: None,
                },
            ],
            acciaccatura: false,
        };
        assert_eq!(result, ("EFG", expected));
    }
}
