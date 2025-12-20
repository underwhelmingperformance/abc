//! Rhythm parsers for tuplets and broken rhythm.
//!
//! This module provides parsers for rhythmic notations including tuplets
//! (irregular note groupings) and broken rhythm (dotted patterns).

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, map_res, opt, value},
    error::context,
    sequence::preceded,
};

use crate::types::ast::{BrokenRhythm, Tuplet};

/// Parse a tuplet specification.
///
/// Tuplets allow irregular rhythmic groupings such as triplets (3 in the
/// time of 2) or duplets (2 in the time of 3). The notation is `(p:q:r`
/// where p is the number of notes, q is the number they replace, and r
/// is their total duration. Both q and r are optional and inferred if omitted.
pub(crate) fn tuplet(input: &str) -> IResult<&str, Tuplet> {
    context(
        "tuplet",
        map(
            (
                char('('),
                map_res(digit1, |s: &str| s.parse::<u8>()),
                opt(preceded(
                    char(':'),
                    map_res(digit1, |s: &str| s.parse::<u8>()),
                )),
                opt(preceded(
                    char(':'),
                    map_res(digit1, |s: &str| s.parse::<u32>()),
                )),
            ),
            |(_, p, q, r)| Tuplet { p, q, r },
        ),
    )
    .parse(input)
}

/// Parse a broken rhythm operator.
///
/// Broken rhythm provides shorthand for dotted note patterns. The `>`
/// operator dots the first note and halves the second, whilst `<` does
/// the reverse. Multiple operators increase the effect.
pub(crate) fn broken_rhythm(input: &str) -> IResult<&str, BrokenRhythm> {
    context(
        "broken rhythm",
        alt((
            value(BrokenRhythm::DoubleDotFirst, tag(">>")),
            value(BrokenRhythm::DoubleDotSecond, tag("<<")),
            value(BrokenRhythm::DotFirst, char('>')),
            value(BrokenRhythm::DotSecond, char('<')),
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
    #[case("(3", ("", Tuplet { p: 3, q: None, r: None }))]
    #[case("(2", ("", Tuplet { p: 2, q: None, r: None }))]
    #[case("(3:2", ("", Tuplet { p: 3, q: Some(2), r: None }))]
    #[case("(3:2:4", ("", Tuplet { p: 3, q: Some(2), r: Some(4) }))]
    #[case("(4", ("", Tuplet { p: 4, q: None, r: None }))]
    #[case("(5:4:6", ("", Tuplet { p: 5, q: Some(4), r: Some(6) }))]
    #[case("(6:4", ("", Tuplet { p: 6, q: Some(4), r: None }))]
    fn test_tuplet(#[case] input: &str, #[case] expected: (&str, Tuplet)) {
        let result = tuplet(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(">", ("", BrokenRhythm::DotFirst))]
    #[case("<", ("", BrokenRhythm::DotSecond))]
    #[case(">>", ("", BrokenRhythm::DoubleDotFirst))]
    #[case("<<", ("", BrokenRhythm::DoubleDotSecond))]
    fn test_broken_rhythm(#[case] input: &str, #[case] expected: (&str, BrokenRhythm)) {
        let result = broken_rhythm(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tuplet_with_trailing() {
        let (remaining, t) = tuplet("(3ABC").unwrap();
        assert_eq!(
            t,
            Tuplet {
                p: 3,
                q: None,
                r: None
            }
        );
        assert_eq!(remaining, "ABC");
    }

    #[test]
    fn test_broken_rhythm_with_trailing() {
        let (remaining, br) = broken_rhythm(">DEF").unwrap();
        assert_eq!(br, BrokenRhythm::DotFirst);
        assert_eq!(remaining, "DEF");
    }
}
