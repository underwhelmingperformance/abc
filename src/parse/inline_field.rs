//! Inline field parsers.
//!
//! This module provides parsers for inline fields, which are information
//! fields that appear within square brackets in the tune body to modify
//! the current musical context (e.g., `[K:D]`, `[M:3/4]`, `[Q:1/4=120]`).

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    error::context,
    sequence::{delimited, preceded},
};

use crate::{
    parse::information_field::{key_signature, meter_symbol, tempo_marking, unit_note_length},
    types::ast::InlineField,
};

/// Parse an inline field.
///
/// Inline fields appear within square brackets in the tune body and modify
/// the current musical context from that point forward in the current voice.
/// They support key changes, metre changes, tempo changes, and unit note
/// length changes.
pub(crate) fn inline_field(input: &str) -> IResult<&str, InlineField<'_>> {
    context(
        "inline field",
        delimited(
            char('['),
            cut(alt((
                map(preceded(tag("K:"), cut(key_signature)), InlineField::Key),
                map(preceded(tag("M:"), cut(meter_symbol)), InlineField::Meter),
                map(preceded(tag("Q:"), cut(tempo_marking)), InlineField::Tempo),
                map(
                    preceded(tag("L:"), cut(unit_note_length)),
                    InlineField::UnitNoteLength,
                ),
            ))),
            cut(char(']')),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::{
        Duration, KeySignature, Meter, MeterSymbol, Mode, PitchClass, TempoMarking,
    };

    #[test]
    fn test_inline_key_change() {
        let result = inline_field("[K:D]").unwrap();
        assert_eq!(
            result,
            (
                "",
                InlineField::Key(KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                })
            )
        );
    }

    #[test]
    fn test_inline_meter_change() {
        let result = inline_field("[M:3/4]").unwrap();
        assert_eq!(
            result,
            (
                "",
                InlineField::Meter(MeterSymbol::Explicit(Meter {
                    numerator: 3,
                    denominator: 4
                }))
            )
        );
    }

    #[test]
    fn test_inline_tempo_change() {
        let result = inline_field("[Q:1/4=120]").unwrap();
        assert_eq!(
            result,
            (
                "",
                InlineField::Tempo(TempoMarking {
                    note_value: Duration::new(1, 4),
                    beats_per_minute: 120
                })
            )
        );
    }

    #[test]
    fn test_inline_unit_length_change() {
        let result = inline_field("[L:1/16]").unwrap();
        assert_eq!(
            result,
            ("", InlineField::UnitNoteLength(Duration::new(1, 16)))
        );
    }

    #[test]
    fn test_inline_field_with_trailing() {
        let result = inline_field("[K:G]ABC").unwrap();
        assert_eq!(
            result,
            (
                "ABC",
                InlineField::Key(KeySignature {
                    tonic: PitchClass::G,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                })
            )
        );
    }
}
