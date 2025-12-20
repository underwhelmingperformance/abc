//! Stylesheet directive parsing.
//!
//! This module parses stylesheet directives that control page layout, margins,
//! spacing, and scaling.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{char, digit1, space0, space1},
    combinator::{map_res, opt, value},
};

use crate::types::ast::{Measurement, StylesheetDirective};

/// Parse a stylesheet directive.
pub(super) fn stylesheet_directive(input: &str) -> IResult<&str, StylesheetDirective<'_>> {
    // Split into groups to stay within nom's alt() tuple size limit (21)
    alt((
        // Page dimensions and margins
        alt((
            prefixed_measurement("pagewidth", StylesheetDirective::PageWidth),
            prefixed_measurement("pageheight", StylesheetDirective::PageHeight),
            prefixed_measurement("topmargin", StylesheetDirective::TopMargin),
            prefixed_measurement("botmargin", StylesheetDirective::BottomMargin),
            prefixed_measurement("leftmargin", StylesheetDirective::LeftMargin),
            prefixed_measurement("rightmargin", StylesheetDirective::RightMargin),
        )),
        // Staff dimensions and spacing
        alt((
            prefixed_measurement("staffsep", StylesheetDirective::StaffSep),
            prefixed_measurement("sysstaffsep", StylesheetDirective::SysStaffSep),
            prefixed_measurement("staffwidth", StylesheetDirective::StaffWidth),
            prefixed_measurement("indent", StylesheetDirective::Indent),
            prefixed_measurement("musicspace", StylesheetDirective::MusicSpace),
            prefixed_measurement("vocalspace", StylesheetDirective::VocalSpace),
            prefixed_measurement("textspace", StylesheetDirective::TextSpace),
        )),
        // Scaling factors, integers, and booleans
        alt((
            prefixed_float("scale", StylesheetDirective::Scale),
            prefixed_float("lineskipfac", StylesheetDirective::LineSkipFac),
            prefixed_float("parskipfac", StylesheetDirective::ParSkipFac),
            prefixed_float("notespacingfactor", StylesheetDirective::NoteSpacingFactor),
            prefixed_float("maxshrink", StylesheetDirective::MaxShrink),
            prefixed_float("stretchlast", StylesheetDirective::StretchLast),
            prefixed_u8("barsperstaff", StylesheetDirective::BarsPerStaff),
            prefixed_bool("linewarn", StylesheetDirective::LineWarn),
            prefixed_bool("continueall", StylesheetDirective::ContinueAll),
            prefixed_bool("landscape", StylesheetDirective::Landscape),
            prefixed_bool("stretchstaff", StylesheetDirective::StretchStaff),
        )),
        // Line breaking and measure numbering
        alt((
            parse_linebreak,
            prefixed_u32("measurenb", StylesheetDirective::MeasureNb),
            parse_breaklimit,
        )),
    ))
    .parse(input)
}

/// Create a parser for a directive with a measurement value.
fn prefixed_measurement<'a, F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, StylesheetDirective<'a>>
where
    F: Fn(Measurement<'a>) -> StylesheetDirective<'a> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, measurement) = parse_measurement(input)?;
        Ok((input, constructor(measurement)))
    }
}

/// Create a parser for a directive with a float value.
fn prefixed_float<F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&str) -> IResult<&str, StylesheetDirective<'_>>
where
    F: Fn(f32) -> StylesheetDirective<'static> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, value) = parse_float(input)?;
        Ok((input, constructor(value)))
    }
}

/// Create a parser for a directive with a u8 value.
fn prefixed_u8<F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&str) -> IResult<&str, StylesheetDirective<'_>>
where
    F: Fn(u8) -> StylesheetDirective<'static> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, value) = parse_u8(input)?;
        Ok((input, constructor(value)))
    }
}

/// Create a parser for a directive with a u32 value.
fn prefixed_u32<F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&str) -> IResult<&str, StylesheetDirective<'_>>
where
    F: Fn(u32) -> StylesheetDirective<'static> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, value) = parse_u32(input)?;
        Ok((input, constructor(value)))
    }
}

/// Create a parser for a directive with a boolean value.
fn prefixed_bool<F>(
    prefix: &'static str,
    constructor: F,
) -> impl FnMut(&str) -> IResult<&str, StylesheetDirective<'_>>
where
    F: Fn(bool) -> StylesheetDirective<'static> + Copy,
{
    move |input| {
        let (input, _) = tag_no_case(prefix).parse(input)?;
        let (input, _) = space1.parse(input)?;
        let (input, value) = parse_bool(input)?;
        Ok((input, constructor(value)))
    }
}

/// Parse a measurement with optional unit.
fn parse_measurement(input: &str) -> IResult<&str, Measurement<'_>> {
    use nom::bytes::complete::take_while1;

    let (input, value) = parse_float(input)?;
    let (input, unit) = opt(take_while1(|c: char| c.is_alphabetic())).parse(input)?;
    Ok((input, Measurement::new(value, unit)))
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

/// Parse a u8 value.
fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>()).parse(input)
}

/// Parse a u32 value.
fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)
}

/// Parse a boolean value (0/1, true/false, yes/no).
fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag_no_case("true")),
        value(false, tag_no_case("false")),
        value(true, tag_no_case("yes")),
        value(false, tag_no_case("no")),
        value(true, tag("1")),
        value(false, tag("0")),
    ))
    .parse(input)
}

/// Parse the `%%linebreak` directive.
///
/// Parses space-separated line break control symbols. Common symbols include:
/// - `<` - decrease line break preference
/// - `!` - prevent line break
/// - `$` - force line break
/// - EOL or empty - end of line marker
fn parse_linebreak(input: &str) -> IResult<&str, StylesheetDirective<'_>> {
    use nom::character::complete::satisfy;

    let (input, _) = tag_no_case("linebreak").parse(input)?;
    let (input, _) = space1.parse(input)?;

    // Parse space-separated symbols until end of input
    let mut symbols = Vec::new();
    let mut remaining = input;

    loop {
        // Skip any whitespace
        let (rest, _) = space0(remaining)?;

        // Check if we're at the end or if we hit "EOL"
        if rest.is_empty() {
            remaining = rest;
            break;
        }

        // Check for special "EOL" token
        if let Ok((rest, _)) = tag_no_case::<_, _, nom::error::Error<_>>("EOL")(rest) {
            // EOL represents a special marker, we can add a newline character
            symbols.push('\n');
            remaining = rest;
            continue;
        }

        // Parse a single character symbol
        match satisfy::<_, _, nom::error::Error<_>>(|c: char| !c.is_whitespace())(rest) {
            Ok((rest, c)) => {
                symbols.push(c);
                remaining = rest;
            }
            Err(_) => break,
        }
    }

    Ok((remaining, StylesheetDirective::LineBreak { symbols }))
}

/// Parse the `%%breaklimit` directive.
///
/// Parses a float value representing the line fullness threshold (0.0-1.0).
/// Values outside this range will be accepted but may be validated later.
fn parse_breaklimit(input: &str) -> IResult<&str, StylesheetDirective<'_>> {
    let (input, _) = tag_no_case("breaklimit").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, value) = parse_float(input)?;

    // Note: Validation of the 0.0-1.0 range should be done at a higher level
    // to provide better error messages. We accept any float here.
    Ok((input, StylesheetDirective::BreakLimit(value)))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_pagewidth() {
        let (remaining, result) = stylesheet_directive("pagewidth 21cm").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::PageWidth(m) = result {
            assert!((m.value - 21.0).abs() < 0.001);
            assert_eq!(m.unit, Some("cm"));
        } else {
            panic!("Expected PageWidth");
        }
    }

    #[test]
    fn test_scale() {
        let (remaining, result) = stylesheet_directive("scale 0.75").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::Scale(value) = result {
            assert!((value - 0.75).abs() < 0.001);
        } else {
            panic!("Expected Scale");
        }
    }

    #[test]
    fn test_barsperstaff() {
        let (remaining, result) = stylesheet_directive("barsperstaff 4").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::BarsPerStaff(4));
    }

    #[test]
    fn test_landscape_true() {
        let (remaining, result) = stylesheet_directive("landscape true").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::Landscape(true));
    }

    #[test]
    fn test_landscape_1() {
        let (remaining, result) = stylesheet_directive("landscape 1").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::Landscape(true));
    }

    #[test]
    fn test_measurement_without_unit() {
        let (remaining, result) = parse_measurement("100").unwrap();
        assert!(remaining.is_empty());
        assert!((result.value - 100.0).abs() < 0.001);
        assert_eq!(result.unit, None);
    }

    #[test]
    fn test_linebreak_single_symbol() {
        let (remaining, result) = stylesheet_directive("linebreak $").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::LineBreak { symbols } = result {
            assert_eq!(symbols, vec!['$']);
        } else {
            panic!("Expected LineBreak");
        }
    }

    #[test]
    fn test_linebreak_multiple_symbols() {
        let (remaining, result) = stylesheet_directive("linebreak < ! $").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::LineBreak { symbols } = result {
            assert_eq!(symbols, vec!['<', '!', '$']);
        } else {
            panic!("Expected LineBreak");
        }
    }

    #[test]
    fn test_linebreak_with_eol() {
        let (remaining, result) = stylesheet_directive("linebreak $ EOL").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::LineBreak { symbols } = result {
            assert_eq!(symbols, vec!['$', '\n']);
        } else {
            panic!("Expected LineBreak");
        }
    }

    #[test]
    fn test_linebreak_all_symbols() {
        let (remaining, result) = stylesheet_directive("linebreak < ! $ EOL").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::LineBreak { symbols } = result {
            assert_eq!(symbols, vec!['<', '!', '$', '\n']);
        } else {
            panic!("Expected LineBreak");
        }
    }

    #[test]
    fn test_measurenb() {
        let (remaining, result) = stylesheet_directive("measurenb 5").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::MeasureNb(5));
    }

    #[test]
    fn test_measurenb_zero() {
        let (remaining, result) = stylesheet_directive("measurenb 0").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::MeasureNb(0));
    }

    #[test]
    fn test_measurenb_large() {
        let (remaining, result) = stylesheet_directive("measurenb 100").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, StylesheetDirective::MeasureNb(100));
    }

    #[test]
    fn test_breaklimit() {
        let (remaining, result) = stylesheet_directive("breaklimit 0.75").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::BreakLimit(value) = result {
            assert!((value - 0.75).abs() < 0.001);
        } else {
            panic!("Expected BreakLimit");
        }
    }

    #[test]
    fn test_breaklimit_zero() {
        let (remaining, result) = stylesheet_directive("breaklimit 0.0").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::BreakLimit(value) = result {
            assert!((value - 0.0).abs() < 0.001);
        } else {
            panic!("Expected BreakLimit");
        }
    }

    #[test]
    fn test_breaklimit_one() {
        let (remaining, result) = stylesheet_directive("breaklimit 1.0").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::BreakLimit(value) = result {
            assert!((value - 1.0).abs() < 0.001);
        } else {
            panic!("Expected BreakLimit");
        }
    }

    #[test]
    #[should_panic]
    fn test_breaklimit_no_leading_zero() {
        // This should fail - we require a leading digit before decimal point
        let (_remaining, _result) = stylesheet_directive("breaklimit .5").unwrap();
    }

    #[test]
    fn test_breaklimit_integer() {
        let (remaining, result) = stylesheet_directive("breaklimit 1").unwrap();
        assert!(remaining.is_empty());
        if let StylesheetDirective::BreakLimit(value) = result {
            assert!((value - 1.0).abs() < 0.001);
        } else {
            panic!("Expected BreakLimit");
        }
    }
}
