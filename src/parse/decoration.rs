//! Decoration parsers.
//!
//! This module provides parsers for decorations, ornaments, and articulation
//! marks in ABC notation.

use std::borrow::Cow;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::char,
    combinator::{cut, map},
    error::context,
    sequence::delimited,
};

use crate::types::{
    Dynamics,
    ast::{Decoration, Dynamic},
};

/// Parse a shorthand decoration.
///
/// Shorthand decorations are single characters that represent common ornaments
/// and articulations. Examples: `~` (trill), `.` (staccato), `T` (trill).
pub(crate) fn shorthand_decoration(input: &str) -> IResult<&str, Decoration<'_>> {
    context(
        "shorthand decoration",
        map(
            alt((
                char('~'), // Trill
                char('.'), // Staccato
                char('T'), // Turn
                char('M'), // Mordent
                char('P'), // Pralltriller
                char('S'), // Segno
                char('O'), // Coda
                char('L'), // Accent
                char('H'), // Fermata
                char('u'), // Upbow
                char('v'), // Downbow
            )),
            Decoration::Shorthand,
        ),
    )
    .parse(input)
}

/// Parse an explicit decoration or dynamic.
///
/// Explicit decorations are written between exclamation marks or plus signs.
/// Examples: `!trill!`, `!staccato!`, `!f!` (forte), `+accent+`.
pub(crate) fn explicit_decoration(input: &str) -> IResult<&str, Decoration<'_>> {
    context(
        "explicit decoration",
        alt((
            delimited(
                char('!'),
                cut(alt((
                    // Volume levels (use shared Dynamics type)
                    map(tag("pppp"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::PPPP))),
                    map(tag("ppp"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::PPP))),
                    map(tag("pp"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::PP))),
                    map(tag("mp"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::MP))),
                    map(tag("p"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::P))),
                    map(tag("mf"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::MF))),
                    map(tag("ffff"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::FFFF))),
                    map(tag("fff"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::FFF))),
                    map(tag("ff"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::FF))),
                    map(tag("f"), |_| Decoration::Dynamic(Dynamic::Level(Dynamics::F))),
                    // Sforzando (accent, not a volume level)
                    map(tag("sfz"), |_| Decoration::Dynamic(Dynamic::Sfz)),
                    // Hairpins (gradual dynamics)
                    map(tag("crescendo("), |_| Decoration::Dynamic(Dynamic::CrescendoStart)),
                    map(tag("crescendo)"), |_| Decoration::Dynamic(Dynamic::CrescendoEnd)),
                    map(tag("<("), |_| Decoration::Dynamic(Dynamic::CrescendoStart)),
                    map(tag("<)"), |_| Decoration::Dynamic(Dynamic::CrescendoEnd)),
                    map(tag("diminuendo("), |_| Decoration::Dynamic(Dynamic::DiminuendoStart)),
                    map(tag("diminuendo)"), |_| Decoration::Dynamic(Dynamic::DiminuendoEnd)),
                    map(tag(">("), |_| Decoration::Dynamic(Dynamic::DiminuendoStart)),
                    map(tag(">)"), |_| Decoration::Dynamic(Dynamic::DiminuendoEnd)),
                    // Fallback: any other explicit decoration
                    map(is_not("!"), |s: &str| {
                        Decoration::Explicit(Cow::Borrowed(s))
                    }),
                ))),
                cut(char('!')),
            ),
            delimited(
                char('+'),
                cut(map(is_not("+"), |s: &str| {
                    Decoration::Explicit(Cow::Borrowed(s))
                })),
                cut(char('+')),
            ),
        )),
    )
    .parse(input)
}

/// Parse any decoration (shorthand or explicit).
pub(crate) fn decoration(input: &str) -> IResult<&str, Decoration<'_>> {
    context(
        "decoration",
        alt((explicit_decoration, shorthand_decoration)),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::types::Dynamics;

    #[rstest]
    #[case("~", '~')]
    #[case(".", '.')]
    #[case("T", 'T')]
    #[case("M", 'M')]
    #[case("P", 'P')]
    fn test_shorthand_decoration(#[case] input: &str, #[case] expected_char: char) {
        let (rest, result) = shorthand_decoration(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, Decoration::Shorthand(expected_char));
    }

    #[rstest]
    #[case("!trill!", "trill")]
    #[case("!staccato!", "staccato")]
    #[case("!accent!", "accent")]
    #[case("+fermata+", "fermata")]
    fn test_explicit_decoration(#[case] input: &str, #[case] expected_name: &str) {
        let (rest, result) = explicit_decoration(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, Decoration::Explicit(Cow::Borrowed(expected_name)));
    }

    #[rstest]
    #[case("!p!", Dynamic::Level(Dynamics::P))]
    #[case("!pp!", Dynamic::Level(Dynamics::PP))]
    #[case("!f!", Dynamic::Level(Dynamics::F))]
    #[case("!ff!", Dynamic::Level(Dynamics::FF))]
    #[case("!mf!", Dynamic::Level(Dynamics::MF))]
    #[case("!mp!", Dynamic::Level(Dynamics::MP))]
    #[case("!sfz!", Dynamic::Sfz)]
    #[case("!crescendo(!", Dynamic::CrescendoStart)]
    #[case("!crescendo)!", Dynamic::CrescendoEnd)]
    #[case("!<(!", Dynamic::CrescendoStart)]
    #[case("!<)!", Dynamic::CrescendoEnd)]
    #[case("!diminuendo(!", Dynamic::DiminuendoStart)]
    #[case("!diminuendo)!", Dynamic::DiminuendoEnd)]
    #[case("!>(!", Dynamic::DiminuendoStart)]
    #[case("!>)!", Dynamic::DiminuendoEnd)]
    fn test_dynamic_decoration(#[case] input: &str, #[case] expected: Dynamic) {
        let (rest, result) = explicit_decoration(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, Decoration::Dynamic(expected));
    }

    #[test]
    fn test_decoration_with_trailing() {
        let result = decoration("~ABC").unwrap();
        assert_eq!(result, ("ABC", Decoration::Shorthand('~')));
    }
}
