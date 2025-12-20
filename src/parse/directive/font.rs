//! Font directive parsing.
//!
//! This module parses font directives that control typography for various
//! elements like titles, composers, annotations, etc.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_till, take_while1},
    character::complete::{digit1, space1},
    combinator::{map_res, opt},
};

use crate::types::ast::{FontDirective, FontSpec};

/// Parse a font directive.
pub(super) fn font_directive(input: &str) -> IResult<&str, FontDirective<'_>> {
    alt((
        prefixed_font("titlefont", FontDirective::TitleFont),
        prefixed_font("subtitlefont", FontDirective::SubtitleFont),
        prefixed_font("composerfont", FontDirective::ComposerFont),
        prefixed_font("partsfont", FontDirective::PartsFont),
        prefixed_font("tempofont", FontDirective::TempoFont),
        prefixed_font("gchordfont", FontDirective::GchordFont),
        prefixed_font("annotationfont", FontDirective::AnnotationFont),
        prefixed_font("infofont", FontDirective::InfoFont),
        prefixed_font("textfont", FontDirective::TextFont),
        prefixed_font("vocalfont", FontDirective::VocalFont),
        prefixed_font("wordsfont", FontDirective::WordsFont),
        prefixed_font("historyfont", FontDirective::HistoryFont),
        prefixed_font("footerfont", FontDirective::FooterFont),
        prefixed_font("headerfont", FontDirective::HeaderFont),
    ))
    .parse(input)
}

/// Create a parser for a font directive.
fn prefixed_font<'a, F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, FontDirective<'a>>
where
    F: Fn(FontSpec<'a>) -> FontDirective<'a> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, spec) = parse_font_spec(input)?;
        Ok((input, constructor(spec)))
    }
}

/// Parse a font specification.
fn parse_font_spec(input: &str) -> IResult<&str, FontSpec<'_>> {
    // Font family (may include hyphens like "Times-Bold")
    let (input, family) =
        take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_').parse(input)?;
    let (input, size) = opt((space1, parse_u8)).parse(input)?;
    let (input, modifiers) = opt((space1, take_till(|c| c == '\n' || c == '\r'))).parse(input)?;

    let spec = if let Some((_, mods)) = modifiers {
        let mods = mods.trim();
        if mods.is_empty() {
            FontSpec::new(family, size.map(|(_, s)| s))
        } else {
            FontSpec::with_modifiers(family, size.map(|(_, s)| s), mods)
        }
    } else {
        FontSpec::new(family, size.map(|(_, s)| s))
    };

    Ok((input, spec))
}

/// Parse a u8 value.
fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>()).parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_titlefont() {
        let (remaining, result) = font_directive("titlefont Times-Bold 16").unwrap();
        assert!(remaining.is_empty());
        if let FontDirective::TitleFont(spec) = result {
            assert_eq!(spec.family, "Times-Bold");
            assert_eq!(spec.size, Some(16));
        } else {
            panic!("Expected TitleFont");
        }
    }

    #[test]
    fn test_composerfont_no_size() {
        let (remaining, result) = font_directive("composerfont Helvetica").unwrap();
        assert!(remaining.is_empty());
        if let FontDirective::ComposerFont(spec) = result {
            assert_eq!(spec.family, "Helvetica");
            assert_eq!(spec.size, None);
        } else {
            panic!("Expected ComposerFont");
        }
    }

    #[test]
    fn test_font_with_modifiers() {
        let (remaining, result) = font_directive("textfont Arial 12 italic").unwrap();
        assert!(remaining.is_empty());
        if let FontDirective::TextFont(spec) = result {
            assert_eq!(spec.family, "Arial");
            assert_eq!(spec.size, Some(12));
            assert_eq!(spec.modifiers, Some("italic"));
        } else {
            panic!("Expected TextFont");
        }
    }
}
