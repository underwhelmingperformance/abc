//! Text directive parsing.
//!
//! This module parses text directives that add annotations, centered text,
//! headers, footers, and page breaks to ABC output.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_till, take_while1},
    character::complete::{char, digit1, space0, space1},
    combinator::{opt, value},
};

use crate::types::ast::{Measurement, TextDirective, VSkipAmount};

/// Parse a text directive.
pub(super) fn text_directive(input: &str) -> IResult<&str, TextDirective<'_>> {
    alt((
        // Simple flags
        value(TextDirective::NewPage, tag_no_case("newpage")),
        value(TextDirective::BeginText, tag_no_case("begintext")),
        value(TextDirective::EndText, tag_no_case("endtext")),
        // Text with content
        prefixed_text("text", TextDirective::Text),
        prefixed_text("center", TextDirective::Center),
        prefixed_text("right", TextDirective::Right),
        prefixed_text("header", TextDirective::Header),
        prefixed_text("footer", TextDirective::Footer),
        // vskip
        parse_vskip,
        // sep (separator)
        parse_sep,
    ))
    .parse(input)
}

/// Create a parser for a text directive with content.
fn prefixed_text<'a, F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, TextDirective<'a>>
where
    F: Fn(&'a str) -> TextDirective<'a> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, content) = take_till(|c| c == '\n' || c == '\r').parse(input)?;
        Ok((input, constructor(content.trim())))
    }
}

/// Parse `%%vskip`.
fn parse_vskip(input: &str) -> IResult<&str, TextDirective<'_>> {
    let (input, _) = tag_no_case("vskip").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, amount) = parse_vskip_amount(input)?;
    Ok((input, TextDirective::VSkip(amount)))
}

/// Parse vskip amount (either measurement or lines).
fn parse_vskip_amount(input: &str) -> IResult<&str, VSkipAmount<'_>> {
    let (input, value) = parse_float(input)?;
    let (input, unit) = opt(take_while1(|c: char| c.is_alphabetic())).parse(input)?;

    let amount = if unit.is_some() {
        VSkipAmount::Measurement(Measurement::new(value, unit))
    } else {
        VSkipAmount::Lines(value)
    };

    Ok((input, amount))
}

/// Parse `%%sep`.
fn parse_sep(input: &str) -> IResult<&str, TextDirective<'_>> {
    let (input, _) = tag_no_case("sep").parse(input)?;
    let (input, _) = space0.parse(input)?;

    // sep may have optional parameters
    let (input, rest) = take_till(|c| c == '\n' || c == '\r').parse(input)?;

    // Parse optional parameters (height width length)
    let params: Vec<&str> = rest.split_whitespace().collect();

    let (height, width, length) = match params.len() {
        0 => (None, None, None),
        1 => (Some(parse_measurement_from_str(params[0])), None, None),
        2 => (
            Some(parse_measurement_from_str(params[0])),
            Some(parse_measurement_from_str(params[1])),
            None,
        ),
        _ => (
            Some(parse_measurement_from_str(params[0])),
            Some(parse_measurement_from_str(params[1])),
            Some(parse_measurement_from_str(params[2])),
        ),
    };

    Ok((
        input,
        TextDirective::Sep {
            height,
            width,
            length,
        },
    ))
}

/// Parse a measurement from a string slice (for sep parameters).
fn parse_measurement_from_str(s: &str) -> Measurement<'_> {
    // Find where the number ends
    let num_end = s
        .find(|c: char| !c.is_numeric() && c != '.' && c != '-')
        .unwrap_or(s.len());
    let (num_str, unit_str) = s.split_at(num_end);

    let value = num_str.parse().unwrap_or(0.0);
    let unit = if unit_str.is_empty() {
        None
    } else {
        Some(unit_str)
    };

    Measurement::new(value, unit)
}

/// Parse a float value.
fn parse_float(input: &str) -> IResult<&str, f32> {
    let (input, sign) = opt(alt((char('+'), char('-')))).parse(input)?;
    let (input, integer) = digit1.parse(input)?;
    let (input, fractional) = opt((char('.'), digit1)).parse(input)?;

    let mut s = String::new();
    if sign == Some('-') {
        s.push('-');
    }
    s.push_str(integer);
    if let Some((_, frac)) = fractional {
        s.push('.');
        s.push_str(frac);
    }

    let value: f32 = s.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
    })?;

    Ok((input, value))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_text() {
        let (remaining, result) = text_directive("text Hello World").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, TextDirective::Text("Hello World"));
    }

    #[test]
    fn test_center() {
        let (remaining, result) = text_directive("center My Tune Collection").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, TextDirective::Center("My Tune Collection"));
    }

    #[test]
    fn test_newpage() {
        let (remaining, result) = text_directive("newpage").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, TextDirective::NewPage);
    }

    #[test]
    fn test_vskip_lines() {
        let (remaining, result) = text_directive("vskip 2").unwrap();
        assert!(remaining.is_empty());
        if let TextDirective::VSkip(VSkipAmount::Lines(lines)) = result {
            assert!((lines - 2.0).abs() < 0.001);
        } else {
            panic!("Expected VSkip with Lines");
        }
    }

    #[test]
    fn test_vskip_measurement() {
        let (remaining, result) = text_directive("vskip 1cm").unwrap();
        assert!(remaining.is_empty());
        if let TextDirective::VSkip(VSkipAmount::Measurement(m)) = result {
            assert!((m.value - 1.0).abs() < 0.001);
            assert_eq!(m.unit, Some("cm"));
        } else {
            panic!("Expected VSkip with Measurement");
        }
    }

    #[test]
    fn test_sep_no_params() {
        let (remaining, result) = text_directive("sep").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            TextDirective::Sep {
                height: None,
                width: None,
                length: None
            }
        );
    }

    #[test]
    fn test_sep_with_params() {
        let (remaining, result) = text_directive("sep 1pt 2pt 10cm").unwrap();
        assert!(remaining.is_empty());
        if let TextDirective::Sep {
            height,
            width,
            length,
        } = result
        {
            assert!(height.is_some());
            assert!(width.is_some());
            assert!(length.is_some());
            assert_eq!(height.as_ref().unwrap().unit, Some("pt"));
            assert_eq!(length.as_ref().unwrap().unit, Some("cm"));
        } else {
            panic!("Expected Sep");
        }
    }
}
