//! Voice parsers.
//!
//! This module provides parsers for voice-related elements in ABC notation,
//! including voice declarations (V: field) and voice overlays.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, i8 as parse_i8, space0, space1},
    combinator::{map, value},
    error::context,
    multi::many0,
    sequence::{delimited, preceded},
};

use crate::types::{Clef, VoiceId, ast::{BodyElement, StemDirection, VoiceAttributes, VoiceDeclaration}};

/// Parse a voice overlay marker (`&`).
///
/// Voice overlays allow multiple melodic lines to be written on the same staff,
/// typically used for brief polyphonic passages. The `&` symbol indicates the
/// start of an overlaid voice.
pub(crate) fn voice_overlay(input: &str) -> IResult<&str, BodyElement<'_>> {
    context(
        "voice overlay",
        map(char('&'), |_| BodyElement::VoiceOverlay),
    )
    .parse(input)
}

/// Parse a voice ID (alphanumeric identifier).
///
/// Voice IDs can be numbers (e.g., "1", "2") or names (e.g., "soprano", "bass").
fn voice_id(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)
}

/// Parse a clef specification.
///
/// Clefs include treble, bass, alto, tenor, perc, and octave-shifted variants.
fn clef(input: &str) -> IResult<&str, Clef> {
    alt((
        value(Clef::TrebleMinus8, tag("treble-8")),
        value(Clef::TreblePlus8, tag("treble+8")),
        value(Clef::BassPlus8, tag("bass+8")),
        value(Clef::Treble, tag("treble")),
        value(Clef::Bass, tag("bass")),
        value(Clef::Alto, tag("alto")),
        value(Clef::Tenor, tag("tenor")),
        value(Clef::Perc, tag("perc")),
        value(Clef::Perc, tag("none")),
    ))
    .parse(input)
}

/// Parse a stem direction.
fn stem_direction(input: &str) -> IResult<&str, StemDirection> {
    alt((
        value(StemDirection::Up, tag("up")),
        value(StemDirection::Down, tag("down")),
        value(StemDirection::Auto, tag("auto")),
    ))
    .parse(input)
}

/// Parse a quoted string value (e.g., `"Soprano"`).
fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), is_not("\""), char('"')).parse(input)
}

/// A single voice attribute.
enum VoiceAttribute<'a> {
    Name(&'a str),
    Subname(&'a str),
    Clef(Clef),
    Stem(StemDirection),
    Octave(i8),
    Transpose(i8),
}

/// Parse a single voice attribute.
fn voice_attribute(input: &str) -> IResult<&str, VoiceAttribute<'_>> {
    alt((
        map(
            preceded(tag("name="), quoted_string),
            VoiceAttribute::Name,
        ),
        map(
            preceded(tag("subname="), quoted_string),
            VoiceAttribute::Subname,
        ),
        map(preceded(tag("clef="), clef), VoiceAttribute::Clef),
        map(
            preceded(tag("stem="), stem_direction),
            VoiceAttribute::Stem,
        ),
        map(preceded(tag("octave="), parse_i8), VoiceAttribute::Octave),
        map(
            preceded(tag("transpose="), parse_i8),
            VoiceAttribute::Transpose,
        ),
    ))
    .parse(input)
}

/// Parse a voice declaration.
///
/// Voice declarations specify attributes for a melodic line. The format is:
/// `V:id [name="..."] [clef=...] [stem=...] [octave=...] [transpose=...]`
///
/// # Examples
///
/// ```text
/// V:1
/// V:soprano name="Soprano" clef=treble
/// V:bass clef=bass octave=-1
/// ```
pub(crate) fn voice_declaration(input: &str) -> IResult<&str, VoiceDeclaration<'_>> {
    context("voice declaration", |input| {
        // Parse the voice ID
        let (input, id_str) = voice_id(input)?;
        let id = VoiceId::new(id_str);

        // Parse optional attributes (space-separated)
        let (input, attrs) = many0(preceded(space1, voice_attribute)).parse(input)?;

        // Consume trailing whitespace
        let (input, _) = space0(input)?;

        // Build VoiceAttributes if any were found
        let attributes = if attrs.is_empty() {
            None
        } else {
            let mut name = None;
            let mut subname = None;
            let mut clef_val = None;
            let mut stem = None;
            let mut octave = None;
            let mut transpose = None;

            for attr in attrs {
                match attr {
                    VoiceAttribute::Name(n) => name = Some(n),
                    VoiceAttribute::Subname(s) => subname = Some(s),
                    VoiceAttribute::Clef(c) => clef_val = Some(c),
                    VoiceAttribute::Stem(s) => stem = Some(s),
                    VoiceAttribute::Octave(o) => octave = Some(o),
                    VoiceAttribute::Transpose(t) => transpose = Some(t),
                }
            }

            Some(VoiceAttributes {
                name,
                subname,
                clef: clef_val,
                stem,
                octave,
                transpose,
                instrument: None, // Not parsed yet
                key: None,        // Not parsed yet
            })
        };

        Ok((input, VoiceDeclaration { id, attributes }))
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("&", ("", BodyElement::VoiceOverlay))]
    fn test_voice_overlay(#[case] input: &str, #[case] expected: (&str, BodyElement)) {
        let result = voice_overlay(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_voice_overlay_with_trailing() {
        let result = voice_overlay("&ABC").unwrap();
        assert_eq!(result, ("ABC", BodyElement::VoiceOverlay));
    }

    #[rstest]
    // Simple numeric ID
    #[case("1", ("", VoiceDeclaration { id: VoiceId::new("1"), attributes: None }))]
    // Named ID
    #[case("soprano", ("", VoiceDeclaration { id: VoiceId::new("soprano"), attributes: None }))]
    // With name attribute
    #[case("1 name=\"Soprano\"", ("", VoiceDeclaration {
        id: VoiceId::new("1"),
        attributes: Some(VoiceAttributes {
            name: Some("Soprano"),
            subname: None,
            clef: None,
            stem: None,
            octave: None,
            transpose: None,
            instrument: None,
            key: None,
        }),
    }))]
    // With clef
    #[case("bass clef=bass", ("", VoiceDeclaration {
        id: VoiceId::new("bass"),
        attributes: Some(VoiceAttributes {
            name: None,
            subname: None,
            clef: Some(Clef::Bass),
            stem: None,
            octave: None,
            transpose: None,
            instrument: None,
            key: None,
        }),
    }))]
    // With multiple attributes
    #[case("2 name=\"Alto\" clef=alto stem=down octave=-1", ("", VoiceDeclaration {
        id: VoiceId::new("2"),
        attributes: Some(VoiceAttributes {
            name: Some("Alto"),
            subname: None,
            clef: Some(Clef::Alto),
            stem: Some(StemDirection::Down),
            octave: Some(-1),
            transpose: None,
            instrument: None,
            key: None,
        }),
    }))]
    // With transpose
    #[case("clarinet transpose=-2", ("", VoiceDeclaration {
        id: VoiceId::new("clarinet"),
        attributes: Some(VoiceAttributes {
            name: None,
            subname: None,
            clef: None,
            stem: None,
            octave: None,
            transpose: Some(-2),
            instrument: None,
            key: None,
        }),
    }))]
    fn test_voice_declaration(
        #[case] input: &str,
        #[case] expected: (&str, VoiceDeclaration),
    ) {
        let result = voice_declaration(input).unwrap();
        assert_eq!(result, expected);
    }
}
