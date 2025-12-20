//! Bar line and repeat parsers.
//!
//! This module provides parsers for bar lines, repeat markers, and variant
//! endings in ABC notation.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, map_res, value},
    error::context,
    multi::separated_list1,
};

use crate::types::ast::{BarLine, VariantEnding};

/// Parse a bar line.
///
/// Bar lines mark measure boundaries and can include repeat markers.
/// ABC notation supports various bar line types.
pub(crate) fn barline(input: &str) -> IResult<&str, BarLine> {
    context(
        "bar line",
        alt((
            // Must check longer patterns first to avoid partial matches
            value(BarLine::RepeatBoth, tag("::")),
            value(BarLine::RepeatStart, tag("|:")),
            value(BarLine::RepeatEnd, tag(":|")),
            value(BarLine::FinalDouble, tag("|]")),
            value(BarLine::StartDouble, tag("[|")),
            value(BarLine::Double, tag("||")),
            value(BarLine::Single, char('|')),
        )),
    )
    .parse(input)
}

/// Parse a variant ending marker with barline prefix.
///
/// Variant endings specify which measures are played during different
/// repetitions. They are written as a bar line followed by ending numbers.
/// Example: `|1` or `:|1` or `|1,2`
fn barline_variant_ending(input: &str) -> IResult<&str, VariantEnding> {
    context(
        "barline variant ending",
        map(
            (
                barline,
                separated_list1(char(','), map_res(digit1, |s: &str| s.parse::<u8>())),
            ),
            |(bar, variants)| VariantEnding { bar, variants },
        ),
    )
    .parse(input)
}

/// Parse a bracket variant ending marker.
///
/// ABC notation also supports bracket-style variant endings: `[1`, `[2`, etc.
/// These are used for first/second ending markers without an explicit barline.
/// The bracket is treated as an implicit "bracket start" (similar to `[|`).
/// Example: `[1` for first ending, `[2` for second ending
fn bracket_variant_ending(input: &str) -> IResult<&str, VariantEnding> {
    context(
        "bracket variant ending",
        map(
            (
                char('['),
                separated_list1(char(','), map_res(digit1, |s: &str| s.parse::<u8>())),
            ),
            |(_, variants)| VariantEnding {
                bar: BarLine::StartDouble, // `[` acts like `[|` for variant start
                variants,
            },
        ),
    )
    .parse(input)
}

/// Parse a variant ending marker (either barline or bracket style).
///
/// Variant endings specify which measures are played during different
/// repetitions. They can be written as:
/// - Bar line followed by ending numbers: `|1`, `:|1,2`
/// - Bracket followed by ending numbers: `[1`, `[2`
pub(crate) fn variant_ending(input: &str) -> IResult<&str, VariantEnding> {
    context(
        "variant ending",
        alt((bracket_variant_ending, barline_variant_ending)),
    )
    .parse(input)
}

/// Parse a bar line or variant ending.
///
/// Attempts to parse a variant ending first (bar line with numbers),
/// falling back to a plain bar line if no numbers are present.
pub(crate) fn barline_or_variant(input: &str) -> IResult<&str, (BarLine, Option<VariantEnding>)> {
    context(
        "bar line or variant",
        alt((
            map(variant_ending, |ve| {
                let bar = ve.bar;
                (bar, Some(ve))
            }),
            map(barline, |bar| (bar, None)),
        )),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("|", ("", BarLine::Single))]
    #[case("||", ("", BarLine::Double))]
    #[case("|]", ("", BarLine::FinalDouble))]
    #[case("[|", ("", BarLine::StartDouble))]
    #[case("|:", ("", BarLine::RepeatStart))]
    #[case(":|", ("", BarLine::RepeatEnd))]
    #[case("::", ("", BarLine::RepeatBoth))]
    fn test_barline(#[case] input: &str, #[case] expected: (&str, BarLine)) {
        let result = barline(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_variant_ending_single() {
        let result = variant_ending("|1").unwrap();
        assert_eq!(
            result,
            (
                "",
                VariantEnding {
                    bar: BarLine::Single,
                    variants: vec![1],
                }
            )
        );
    }

    #[test]
    fn test_variant_ending_multiple() {
        let result = variant_ending("|1,2,3").unwrap();
        assert_eq!(
            result,
            (
                "",
                VariantEnding {
                    bar: BarLine::Single,
                    variants: vec![1, 2, 3],
                }
            )
        );
    }

    #[test]
    fn test_variant_ending_with_repeat() {
        let result = variant_ending("|:1").unwrap();
        assert_eq!(
            result,
            (
                "",
                VariantEnding {
                    bar: BarLine::RepeatStart,
                    variants: vec![1],
                }
            )
        );
    }

    #[test]
    fn test_barline_with_trailing() {
        let result = barline("|GAB").unwrap();
        assert_eq!(result, ("GAB", BarLine::Single));
    }
}
