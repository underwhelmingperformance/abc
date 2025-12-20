//! Main directive parser entry point.
//!
//! This module provides the top-level `directive` parser that handles the `%%`
//! prefix and dispatches to the appropriate sub-parser based on directive type.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_while1},
    character::complete::{space0, space1},
    combinator::map,
};

use super::{font, midi, stylesheet, text};
use crate::types::ast::Directive;

/// Parse a directive line starting with `%%`.
///
/// This is the main entry point for parsing directives. It handles the `%%`
/// prefix and dispatches to the appropriate sub-parser.
pub(crate) fn directive(input: &str) -> IResult<&str, Directive<'_>> {
    let (input, _) = tag("%%").parse(input)?;
    let (input, _) = space0.parse(input)?;

    alt((
        // MIDI directive
        map(
            (tag_no_case("MIDI"), space1, midi::midi_directive),
            |(_, _, d)| Directive::Midi(d),
        ),
        // Stylesheet directives
        map(stylesheet::stylesheet_directive, Directive::Stylesheet),
        // Font directives
        map(font::font_directive, Directive::Font),
        // Text directives
        map(text::text_directive, Directive::Text),
        // Unknown directive (fallback)
        unknown_directive,
    ))
    .parse(input)
}

/// Parse an unknown directive (fallback).
fn unknown_directive(input: &str) -> IResult<&str, Directive<'_>> {
    let (input, name) =
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-').parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, value) = take_till(|c| c == '\n' || c == '\r').parse(input)?;
    Ok((
        input,
        Directive::Unknown {
            name,
            value: value.trim(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::ast::{StylesheetDirective, TextDirective};

    #[test]
    fn test_directive_midi() {
        let (remaining, result) = directive("%%MIDI channel 1").unwrap();
        assert!(remaining.is_empty());
        assert!(matches!(result, Directive::Midi(_)));
    }

    #[test]
    fn test_directive_pagewidth() {
        let (remaining, result) = directive("%%pagewidth 21cm").unwrap();
        assert!(remaining.is_empty());
        assert!(matches!(
            result,
            Directive::Stylesheet(StylesheetDirective::PageWidth(_))
        ));
    }

    #[test]
    fn test_directive_scale() {
        let (remaining, result) = directive("%%scale 0.75").unwrap();
        assert!(remaining.is_empty());
        if let Directive::Stylesheet(StylesheetDirective::Scale(value)) = result {
            assert!((value - 0.75).abs() < 0.001);
        } else {
            panic!("Expected Scale directive");
        }
    }

    #[test]
    fn test_directive_titlefont() {
        let (remaining, result) = directive("%%titlefont Times-Bold 16").unwrap();
        assert!(remaining.is_empty());
        if let Directive::Font(crate::types::ast::FontDirective::TitleFont(spec)) = result {
            assert_eq!(spec.family, "Times-Bold");
            assert_eq!(spec.size, Some(16));
        } else {
            panic!("Expected TitleFont directive");
        }
    }

    #[test]
    fn test_directive_text() {
        let (remaining, result) = directive("%%text Hello World").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, Directive::Text(TextDirective::Text("Hello World")));
    }

    #[test]
    fn test_directive_center() {
        let (remaining, result) = directive("%%center My Tune Collection").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            Directive::Text(TextDirective::Center("My Tune Collection"))
        );
    }

    #[test]
    fn test_directive_newpage() {
        let (remaining, result) = directive("%%newpage").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, Directive::Text(TextDirective::NewPage));
    }

    #[test]
    fn test_directive_unknown() {
        let (remaining, result) = directive("%%customthing value").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            Directive::Unknown {
                name: "customthing",
                value: "value"
            }
        );
    }
}
