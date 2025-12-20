//! Tie and slur parsers.
//!
//! This module provides parsers for ties and slurs in ABC notation. Ties connect
//! two notes of the same pitch, whilst slurs connect notes of different pitches.

use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, character::complete::char, combinator::map, error::context,
};

use crate::types::ast::BodyElement;

/// Parse a tie or slur marker.
///
/// - `-` indicates a tie (connects same-pitch notes)
/// - `.(` begins a dotted slur (connects different-pitch notes with dotted line)
/// - `(` begins a regular slur (connects different-pitch notes smoothly)
/// - `)` ends a slur
pub(crate) fn tie_or_slur(input: &str) -> IResult<&str, BodyElement<'_>> {
    context(
        "tie or slur",
        alt((
            map(char('-'), |_| BodyElement::TieStart),
            // Dotted slur must come before regular slur to match correctly
            map(tag(".("), |_| BodyElement::SlurStart { dotted: true }),
            map(char('('), |_| BodyElement::SlurStart { dotted: false }),
            map(char(')'), |_| BodyElement::SlurEnd),
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
    #[case("-", ("", BodyElement::TieStart))]
    #[case("(", ("", BodyElement::SlurStart { dotted: false }))]
    #[case(".(", ("", BodyElement::SlurStart { dotted: true }))]
    #[case(")", ("", BodyElement::SlurEnd))]
    fn test_tie_or_slur(#[case] input: &str, #[case] expected: (&str, BodyElement)) {
        let result = tie_or_slur(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tie_with_trailing() {
        let result = tie_or_slur("-ABC").unwrap();
        assert_eq!(result, ("ABC", BodyElement::TieStart));
    }

    #[test]
    fn test_dotted_slur_with_trailing() {
        let result = tie_or_slur(".(ABC").unwrap();
        assert_eq!(result, ("ABC", BodyElement::SlurStart { dotted: true }));
    }

    #[test]
    fn test_regular_slur_with_trailing() {
        let result = tie_or_slur("(ABC").unwrap();
        assert_eq!(result, ("ABC", BodyElement::SlurStart { dotted: false }));
    }
}
