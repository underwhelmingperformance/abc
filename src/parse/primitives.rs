//! Primitive parsers for ABC notation.
//!
//! This module provides building block parsers for the fundamental elements
//! of ABC notation: pitch classes, accidentals, octave modifiers, and duration
//! modifiers. These primitives are composed to create higher-level parsers.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, one_of},
    combinator::{map, map_opt, not, opt, peek, value},
    error::context,
    multi::many0,
    sequence::{pair, preceded, terminated},
};

use crate::types::{Accidental, Duration, Octave, PitchClass};

/// Parse a pitch class (note letter).
///
/// Recognises the seven note letters: C, D, E, F, G, A, B in either
/// uppercase (higher octave) or lowercase (lower octave).
pub(crate) fn pitch_class(input: &str) -> IResult<&str, PitchClass> {
    context(
        "pitch class",
        map_opt(one_of("CDEFGABcdefgab"), |c| PitchClass::try_from(c).ok()),
    )
    .parse(input)
}

/// Parse a pitch class with case information.
///
/// Returns the pitch class and whether the original character was lowercase.
/// This is needed to determine the octave offset.
pub(crate) fn pitch_class_with_case(input: &str) -> IResult<&str, (PitchClass, bool)> {
    context(
        "pitch class",
        map_opt(one_of("CDEFGABcdefgab"), |c| {
            PitchClass::try_from(c)
                .ok()
                .map(|pc| (pc, c.is_ascii_lowercase()))
        }),
    )
    .parse(input)
}

/// Parse an accidental modifier.
///
/// Recognises:
/// - Standard: sharp (^), flat (_), natural (=), double sharp (^^), double flat (__)
/// - Microtonal: quarter-sharp (^/), quarter-flat (_/), three-quarter-sharp (^3/4),
///   three-quarter-flat (_3/4)
pub(crate) fn accidental(input: &str) -> IResult<&str, Accidental> {
    context(
        "accidental",
        alt((
            // Double accidentals first (longest matches)
            value(Accidental::DoubleSharp, tag("^^")),
            value(Accidental::DoubleFlat, tag("__")),
            // Microtonal accidentals (longer patterns before shorter)
            value(Accidental::ThreeQuarterSharp, tag("^3/4")),
            value(Accidental::ThreeQuarterFlat, tag("_3/4")),
            value(Accidental::QuarterSharp, tag("^/")),
            value(Accidental::QuarterFlat, tag("_/")),
            // Standard single accidentals
            value(Accidental::Sharp, char('^')),
            value(Accidental::Flat, char('_')),
            value(Accidental::Natural, char('=')),
        )),
    )
    .parse(input)
}

/// Parse a single octave modifier.
///
/// Recognises apostrophe (') for raising octave and comma (,) for
/// lowering octave. Returns +1 for apostrophe, -1 for comma.
pub(crate) fn octave_modifier(input: &str) -> IResult<&str, i8> {
    context(
        "octave modifier",
        alt((value(1, char('\'')), value(-1, char(',')))),
    )
    .parse(input)
}

/// Parse octave modifiers and compute total octave shift.
///
/// Multiple modifiers can be applied (e.g., `C''` or `C,,`). The
/// shift is the sum of all modifiers.
pub(crate) fn octave_shift(input: &str) -> IResult<&str, Octave> {
    context(
        "octave shift",
        map(many0(octave_modifier), |mods| Octave(mods.iter().sum())),
    )
    .parse(input)
}

/// Maximum allowed value for duration numerator/denominator.
///
/// This prevents parsing unreasonably large values that could cause
/// overflow or represent nonsensical musical durations. 1024 allows
/// for practical divisions (up to 1/1024 notes) whilst rejecting
/// clearly erroneous input like `C999999`.
const MAX_DURATION_VALUE: u32 = 1024;

/// Parse a bounded u32 from digit string.
///
/// Returns None if the parsed value exceeds MAX_DURATION_VALUE.
fn parse_bounded_u32(s: &str) -> Option<u32> {
    s.parse::<u32>().ok().filter(|&n| n <= MAX_DURATION_VALUE)
}

/// Parse a duration modifier.
///
/// Duration modifiers specify how long a note should be held relative
/// to the default unit note length. Formats include:
/// - `/` alone means "halve" (equivalent to `/2`)
/// - `//` means "quarter" (equivalent to `/4`)
/// - `/N` means "divide by N"
/// - `N` means "multiply by N"
/// - `N/M` means "multiply by N, divide by M"
///
/// Returns a Duration (numerator, denominator). Values are bounded to
/// a maximum of 1024 to prevent overflow and reject nonsensical input.
///
/// The parser is structured to minimize backtracking by grouping alternatives
/// by their first character: digit-first (N or N/M) vs slash-first (/N or //).
pub(crate) fn duration_modifier(input: &str) -> IResult<&str, Duration> {
    context(
        "duration modifier",
        alt((
            // Digit-first: N/M or just N
            // Parse the numerator once, then optionally look for /denominator
            map_opt(
                pair(digit1, opt(preceded(char('/'), digit1))),
                |(num_str, maybe_denom): (&str, Option<&str>)| {
                    let num = parse_bounded_u32(num_str)?;
                    match maybe_denom {
                        Some(denom_str) => {
                            let denom = parse_bounded_u32(denom_str).filter(|&d| d > 0)?;
                            Some(Duration::new(num, denom))
                        }
                        None => Some(Duration::new(num, 1)),
                    }
                },
            ),
            // Slash-first: /N or bare slashes (/, //, ///, etc.)
            // Consume the first slash, then dispatch on what follows
            preceded(
                char('/'),
                alt((
                    // /N - digit immediately after first slash (reject /0)
                    map_opt(digit1, |denom_str: &str| {
                        let denom = parse_bounded_u32(denom_str).filter(|&d| d > 0)?;
                        Some(Duration::new(1, denom))
                    }),
                    // Bare slashes: more slashes or end of duration
                    // / = 1/2, // = 1/4, /// = 1/8, etc.
                    map(
                        terminated(many0(char('/')), peek(not(one_of("0123456789")))),
                        |extra_slashes: Vec<char>| {
                            let count = (extra_slashes.len() + 1) as u32; // +1 for first slash
                            Duration::new(1, 1 << count)
                        },
                    ),
                )),
            ),
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
    #[case("C", ("", PitchClass::C))]
    #[case("c", ("", PitchClass::C))]
    #[case("D", ("", PitchClass::D))]
    #[case("d", ("", PitchClass::D))]
    #[case("E", ("", PitchClass::E))]
    #[case("F", ("", PitchClass::F))]
    #[case("G", ("", PitchClass::G))]
    #[case("A", ("", PitchClass::A))]
    #[case("B", ("", PitchClass::B))]
    fn test_pitch_class(#[case] input: &str, #[case] expected: (&str, PitchClass)) {
        let result = pitch_class(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    // Standard accidentals
    #[case("^", ("", Accidental::Sharp))]
    #[case("_", ("", Accidental::Flat))]
    #[case("=", ("", Accidental::Natural))]
    #[case("^^", ("", Accidental::DoubleSharp))]
    #[case("__", ("", Accidental::DoubleFlat))]
    // Microtonal accidentals
    #[case("^/", ("", Accidental::QuarterSharp))]
    #[case("_/", ("", Accidental::QuarterFlat))]
    #[case("^3/4", ("", Accidental::ThreeQuarterSharp))]
    #[case("_3/4", ("", Accidental::ThreeQuarterFlat))]
    fn test_accidental(#[case] input: &str, #[case] expected: (&str, Accidental)) {
        let result = accidental(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_microtonal_accidental_with_trailing() {
        // ^/C should parse as quarter-sharp followed by 'C'
        let result = accidental("^/C").unwrap();
        assert_eq!(result, ("C", Accidental::QuarterSharp));

        // ^3/4C should parse as three-quarter-sharp followed by 'C'
        let result = accidental("^3/4C").unwrap();
        assert_eq!(result, ("C", Accidental::ThreeQuarterSharp));
    }

    #[rstest]
    #[case("'", ("", 1))]
    #[case(",", ("", -1))]
    fn test_octave_modifier(#[case] input: &str, #[case] expected: (&str, i8)) {
        let result = octave_modifier(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("", ("", Octave(0)))]
    #[case("'", ("", Octave(1)))]
    #[case("''", ("", Octave(2)))]
    #[case(",", ("", Octave(-1)))]
    #[case(",,", ("", Octave(-2)))]
    #[case(",,,", ("", Octave(-3)))]
    fn test_octave_shift(#[case] input: &str, #[case] expected: (&str, Octave)) {
        let result = octave_shift(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("2", ("", Duration::new(2, 1)))]
    #[case("3", ("", Duration::new(3, 1)))]
    #[case("/", ("", Duration::new(1, 2)))]
    #[case("//", ("", Duration::new(1, 4)))]
    #[case("///", ("", Duration::new(1, 8)))]
    #[case("/2", ("", Duration::new(1, 2)))]
    #[case("/4", ("", Duration::new(1, 4)))]
    #[case("3/2", ("", Duration::new(3, 2)))]
    #[case("1/8", ("", Duration::new(1, 8)))]
    fn test_duration_modifier(#[case] input: &str, #[case] expected: (&str, Duration)) {
        let result = duration_modifier(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pitch_class_with_trailing() {
        let result = pitch_class("C2").unwrap();
        let expected = ("2", PitchClass::C);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_pitch_class() {
        let result = pitch_class("H");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_accidental() {
        let result = accidental("x");
        assert!(result.is_err());
    }

    #[rstest]
    #[case("9999")]
    #[case("/9999")]
    #[case("9999/1")]
    #[case("1/9999")]
    fn test_duration_bounds_exceeded(#[case] input: &str) {
        // Values exceeding MAX_DURATION_VALUE (1024) should fail
        assert!(duration_modifier(input).is_err());
    }

    #[rstest]
    #[case("1024", ("", Duration::new(1024, 1)))]
    #[case("/1024", ("", Duration::new(1, 1024)))]
    #[case("1024/1", ("", Duration::new(1024, 1)))]
    #[case("1/1024", ("", Duration::new(1, 1024)))]
    fn test_duration_at_bounds(#[case] input: &str, #[case] expected: (&str, Duration)) {
        // Values at the boundary should succeed
        let result = duration_modifier(input).unwrap();
        assert_eq!(result, expected);
    }

    mod proptest_tests {
        use super::*;
        use proptest::prelude::*;

        /// Strategy for generating valid pitch class characters.
        fn pitch_char_strategy() -> impl Strategy<Value = char> {
            prop_oneof![
                Just('C'),
                Just('D'),
                Just('E'),
                Just('F'),
                Just('G'),
                Just('A'),
                Just('B'),
                Just('c'),
                Just('d'),
                Just('e'),
                Just('f'),
                Just('g'),
                Just('a'),
                Just('b'),
            ]
        }

        /// Strategy for generating valid accidental strings.
        fn accidental_strategy() -> impl Strategy<Value = &'static str> {
            prop_oneof![
                // Standard accidentals
                Just("^"),
                Just("_"),
                Just("="),
                Just("^^"),
                Just("__"),
                // Microtonal accidentals
                Just("^/"),
                Just("_/"),
                Just("^3/4"),
                Just("_3/4"),
            ]
        }

        /// Strategy for generating valid duration modifier strings.
        fn duration_strategy() -> impl Strategy<Value = String> {
            prop_oneof![
                // N (multiplier only)
                (1u32..=1024).prop_map(|n| n.to_string()),
                // /N (divisor only)
                (1u32..=1024).prop_map(|n| format!("/{n}")),
                // N/M (fraction)
                (1u32..=1024, 1u32..=1024).prop_map(|(n, m)| format!("{n}/{m}")),
                // Bare slashes (1-4 slashes)
                (1usize..=4).prop_map(|n| "/".repeat(n)),
            ]
        }

        proptest! {
            /// All valid pitch characters should parse successfully.
            #[test]
            fn prop_valid_pitch_parses(c in pitch_char_strategy()) {
                let input = c.to_string();
                let result = pitch_class(&input);
                prop_assert!(result.is_ok(), "Failed to parse valid pitch: {}", c);
                let (rest, _) = result.unwrap();
                prop_assert_eq!(rest, "", "Unexpected trailing input after pitch");
            }

            /// Pitch class with case should return correct case information.
            #[test]
            fn prop_pitch_case_detection(c in pitch_char_strategy()) {
                let input = c.to_string();
                let result = pitch_class_with_case(&input);
                prop_assert!(result.is_ok());
                let (_, (_, is_lower)) = result.unwrap();
                prop_assert_eq!(is_lower, c.is_ascii_lowercase());
            }

            /// All valid accidentals should parse successfully.
            #[test]
            fn prop_valid_accidental_parses(s in accidental_strategy()) {
                let result = accidental(s);
                prop_assert!(result.is_ok(), "Failed to parse valid accidental: {}", s);
            }

            /// All generated duration modifiers should parse successfully.
            #[test]
            fn prop_valid_duration_parses(s in duration_strategy()) {
                let result = duration_modifier(&s);
                prop_assert!(result.is_ok(), "Failed to parse valid duration: {}", s);
            }

            /// Parsed durations should have positive numerator and denominator.
            #[test]
            fn prop_duration_positive_values(s in duration_strategy()) {
                if let Ok((_, duration)) = duration_modifier(&s) {
                    prop_assert!(duration.numerator() > 0, "Numerator should be positive");
                    prop_assert!(duration.denominator() > 0, "Denominator should be positive");
                }
            }

            /// Octave modifiers should sum correctly.
            #[test]
            fn prop_octave_shift_sums(
                ups in 0usize..=5,
                downs in 0usize..=5
            ) {
                let input = "'".repeat(ups) + &",".repeat(downs);
                let result = octave_shift(&input);
                prop_assert!(result.is_ok());
                let (_, octave) = result.unwrap();
                let expected = ups as i8 - downs as i8;
                prop_assert_eq!(octave.0, expected);
            }

            /// Parser should not panic on arbitrary ASCII input.
            #[test]
            fn prop_pitch_no_panic(s in "\\PC*") {
                // Just check it doesn't panic - may succeed or fail
                let _ = pitch_class(&s);
            }

            /// Parser should not panic on arbitrary ASCII input.
            #[test]
            fn prop_duration_no_panic(s in "\\PC*") {
                let _ = duration_modifier(&s);
            }

            /// Parser should not panic on arbitrary ASCII input.
            #[test]
            fn prop_accidental_no_panic(s in "\\PC*") {
                let _ = accidental(&s);
            }
        }
    }
}
