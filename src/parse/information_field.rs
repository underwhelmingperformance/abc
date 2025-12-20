//! Information field parsers.
//!
//! This module provides parsers for ABC information fields, which provide
//! metadata and musical parameters for tunes. Information fields are written
//! as a field identifier (one or two characters) followed by a colon and
//! the field content.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_while1},
    character::complete::{anychar, char, digit1, line_ending, space0, space1},
    combinator::{eof, map, map_res, opt, peek, value},
    error::context,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated},
};

#[cfg(test)]
use crate::types::PitchClass;
use crate::{
    parse::primitives::{accidental, pitch_class},
    types::{Duration, KeySignature, Meter, MeterSymbol, Mode, ReferenceNumber, TempoMarking},
};

/// Parse the X: (reference number) field.
///
/// The reference number uniquely identifies a tune and must be the first
/// field in a tune header. It consists of the letter 'X' followed by a
/// colon and a positive integer.
pub(crate) fn reference_number(input: &str) -> IResult<&str, ReferenceNumber> {
    context(
        "reference number",
        map_res(terminated(digit1, opt(line_ending)), |s: &str| {
            s.parse::<u32>().map(ReferenceNumber)
        }),
    )
    .parse(input)
}

/// Parse the M: (metre/time signature) field.
///
/// Metre can be specified as:
/// - `C` for common time (4/4)
/// - `C|` for cut time (2/2)
/// - An explicit fraction like `3/4`, `6/8`
pub(crate) fn meter_symbol(input: &str) -> IResult<&str, MeterSymbol> {
    context(
        "meter symbol",
        terminated(
            alt((
                value(MeterSymbol::CutTime, tag("C|")),
                value(MeterSymbol::CommonTime, char('C')),
                map_res(
                    separated_pair(digit1, char('/'), digit1),
                    |(num_str, denom_str): (&str, &str)| {
                        Ok::<_, std::num::ParseIntError>(MeterSymbol::Explicit(Meter {
                            numerator: num_str.parse()?,
                            denominator: denom_str.parse()?,
                        }))
                    },
                ),
            )),
            opt(line_ending),
        ),
    )
    .parse(input)
}

/// Parse the L: (unit note length) field.
///
/// Unit note length is specified as a fraction (e.g., `1/8`, `1/16`).
pub(crate) fn unit_note_length(input: &str) -> IResult<&str, Duration> {
    context(
        "unit note length",
        terminated(
            map_res(
                separated_pair(digit1, char('/'), digit1),
                |(num_str, denom_str): (&str, &str)| {
                    Ok::<_, std::num::ParseIntError>(Duration::new(
                        num_str.parse()?,
                        denom_str.parse()?,
                    ))
                },
            ),
            opt(line_ending),
        ),
    )
    .parse(input)
}

/// Parse the Q: (tempo) field.
///
/// Tempo is specified as a note value followed by equals and BPM.
/// For example, `1/4=120` means 120 quarter notes per minute.
pub(crate) fn tempo_marking(input: &str) -> IResult<&str, TempoMarking> {
    context(
        "tempo marking",
        terminated(
            map_res(
                separated_pair(separated_pair(digit1, char('/'), digit1), char('='), digit1),
                |((num_str, denom_str), bpm_str): ((&str, &str), &str)| {
                    Ok::<_, std::num::ParseIntError>(TempoMarking {
                        note_value: Duration::new(num_str.parse()?, denom_str.parse()?),
                        beats_per_minute: bpm_str.parse()?,
                    })
                },
            ),
            opt(line_ending),
        ),
    )
    .parse(input)
}

/// Parse a mode name.
///
/// Modes can be abbreviated (e.g., "min", "m", "dor") or written in full
/// (e.g., "minor", "dorian"). The single letter "m" specifically means minor.
fn mode(input: &str) -> IResult<&str, Mode> {
    // Macro to define word boundary check (can't clone parsers)
    macro_rules! word_boundary {
        () => {
            peek(alt((space1, tag("="), tag("\n"), tag("\r"), eof)))
        };
    }

    context(
        "mode",
        alt((
            // Longest patterns first to avoid partial matches
            terminated(value(Mode::Mixolydian, alt((tag("mixolydian"), tag("mix")))), word_boundary!()),
            terminated(value(Mode::Ionian, alt((tag("ionian"), tag("major"), tag("maj"), tag("ion")))), word_boundary!()),
            terminated(value(Mode::Minor, alt((tag("aeolian"), tag("minor"), tag("aeo")))), word_boundary!()),
            terminated(value(Mode::Dorian, alt((tag("dorian"), tag("dor")))), word_boundary!()),
            terminated(value(Mode::Phrygian, alt((tag("phrygian"), tag("phr")))), word_boundary!()),
            terminated(value(Mode::Lydian, alt((tag("lydian"), tag("lyd")))), word_boundary!()),
            terminated(value(Mode::Locrian, alt((tag("locrian"), tag("loc")))), word_boundary!()),
            // Special case: "min" must be followed by word boundary to not match "middle"
            terminated(value(Mode::Minor, tag("min")), word_boundary!()),
            // Single 'm' = minor (only if followed by space/newline/end)
            terminated(value(Mode::Minor, char('m')), word_boundary!()),
        )),
    )
    .parse(input)
}

/// Parse a U: (user-defined symbol) field.
///
/// User-defined symbols allow mapping a single character to a decoration.
/// Format: `U:symbol=decoration` or `U:symbol=!decoration!`
///
/// # Examples
///
/// ```text
/// U:T=!trill!
/// U:H=!fermata!
/// U:~=!roll!
/// ```
pub(crate) fn user_defined_symbol(input: &str) -> IResult<&str, (char, &str)> {
    context(
        "user-defined symbol",
        terminated(
            map(
                (
                    // The symbol being defined (any single character)
                    anychar,
                    // Optional whitespace before =
                    space0,
                    // Equals sign
                    char('='),
                    // Optional whitespace after =
                    space0,
                    // The decoration name (with or without ! delimiters)
                    alt((
                        // !decoration! format
                        delimited(char('!'), take_while1(|c| c != '!'), char('!')),
                        // Plain decoration name (until end of line)
                        take_till(|c| c == '\n' || c == '\r'),
                    )),
                ),
                |(symbol, _, _, _, decoration)| (symbol, decoration),
            ),
            opt(line_ending),
        ),
    )
    .parse(input)
}

/// Parse an explicit accidental override (e.g., `=c`, `^f`, `_b`).
///
/// These appear after the key signature and override the implied accidentals.
/// Format: accidental + pitch class (lowercase pitch class for pitch name)
fn explicit_accidental_override(
    input: &str,
) -> IResult<&str, (crate::types::PitchClass, crate::types::Accidental)> {
    context(
        "explicit accidental override",
        map((accidental, pitch_class), |(acc, pc)| (pc, acc)),
    )
    .parse(input)
}

/// Parse a middle=pitch parameter.
///
/// Specifies which pitch is on the middle line of the staff.
/// Format: `middle=` followed by a pitch class letter (e.g., `middle=d`, `middle=B`)
fn middle_parameter(input: &str) -> IResult<&str, crate::types::PitchClass> {
    context(
        "middle parameter",
        preceded(tag("middle="), pitch_class),
    )
    .parse(input)
}

/// Parse a stafflines=N parameter.
///
/// Specifies the number of staff lines (typically 1-6).
/// Format: `stafflines=` followed by a digit (e.g., `stafflines=5`, `stafflines=4`)
fn stafflines_parameter(input: &str) -> IResult<&str, u8> {
    context(
        "stafflines parameter",
        preceded(
            tag("stafflines="),
            map_res(digit1, |s: &str| -> Result<u8, &'static str> {
                let n = s.parse::<u8>().map_err(|_| "Invalid number")?;
                // Validate range: typically 1-6 staff lines
                if (1..=6).contains(&n) {
                    Ok(n)
                } else {
                    Err("stafflines must be between 1 and 6")
                }
            }),
        ),
    )
    .parse(input)
}

/// Represents a parameter that can appear in a key signature field.
#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyParameter {
    /// Explicit accidental override (e.g., `=c`, `^f`)
    ExplicitAccidental(crate::types::PitchClass, crate::types::Accidental),
    /// Middle line pitch (e.g., `middle=d`)
    Middle(crate::types::PitchClass),
    /// Number of staff lines (e.g., `stafflines=5`)
    Stafflines(u8),
}

/// Parse any key signature parameter.
fn key_parameter(input: &str) -> IResult<&str, KeyParameter> {
    context(
        "key parameter",
        alt((
            map(middle_parameter, KeyParameter::Middle),
            map(stafflines_parameter, KeyParameter::Stafflines),
            map(explicit_accidental_override, |(pc, acc)| {
                KeyParameter::ExplicitAccidental(pc, acc)
            }),
        )),
    )
    .parse(input)
}

/// Parse the K: (key signature) field.
///
/// Key signatures specify the tonic and mode. They can include additional
/// modifiers for clef, transposition, explicit accidentals, middle line pitch,
/// and staff line count.
///
/// # Special Keys
///
/// - `K:HP` - Highland Bagpipe (A mixolydian with F# and C#)
/// - `K:Hp` - Highland Bagpipe (alternative form)
/// - `K:none` - No key signature (all naturals)
///
/// # Parameters
///
/// After the key and optional mode, you can specify parameters separated by spaces:
/// - Explicit accidentals: `=c`, `^f`, `_b` (override key signature)
/// - Middle line pitch: `middle=d` (rendering info)
/// - Staff line count: `stafflines=5` (rendering info)
///
/// # Examples
///
/// - `K:D =c` - D major but with C natural
/// - `K:D middle=d` - D major with d on middle line
/// - `K:D stafflines=5` - D major with 5 staff lines
/// - `K:D middle=d stafflines=5` - D major with both rendering parameters
pub(crate) fn key_signature(input: &str) -> IResult<&str, KeySignature> {
    context(
        "key signature",
        terminated(
            alt((
                // K:HP - Highland Bagpipe (uppercase P)
                map(tag("HP"), |_| KeySignature {
                    tonic: crate::types::PitchClass::A,
                    accidental: None,
                    mode: Mode::HighlandBagpipe,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
                // K:Hp - Highland Bagpipe (lowercase p)
                map(tag("Hp"), |_| KeySignature {
                    tonic: crate::types::PitchClass::A,
                    accidental: None,
                    mode: Mode::HighlandBagpipeLower,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
                // K:none - No key signature
                map(tag("none"), |_| KeySignature {
                    tonic: crate::types::PitchClass::C,
                    accidental: None,
                    mode: Mode::None,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
                // Standard key signature with optional parameters
                map(
                    (
                        opt(accidental),
                        pitch_class,
                        opt(preceded(space0, mode)),
                        many0(preceded(space1, key_parameter)),
                    ),
                    |(acc, tonic, mode_opt, parameters)| {
                        let mode = mode_opt.unwrap_or(Mode::Major);

                        // Process parameters
                        let mut explicit_accidentals = Vec::new();
                        let mut middle = None;
                        let mut stafflines = None;

                        for param in parameters {
                            match param {
                                KeyParameter::ExplicitAccidental(pc, accidental) => {
                                    explicit_accidentals.push((pc, accidental));
                                }
                                KeyParameter::Middle(pc) => {
                                    middle = Some(pc);
                                }
                                KeyParameter::Stafflines(n) => {
                                    stafflines = Some(n);
                                }
                            }
                        }

                        KeySignature {
                            tonic,
                            accidental: acc,
                            mode,
                            explicit_accidentals,
                            clef: None,
                            transpose: None,
                            octave_shift: None,
                            middle,
                            stafflines,
                        }
                    },
                ),
            )),
            opt(line_ending),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("T=!trill!\n", ("", ('T', "trill")))]
    #[case("H=!fermata!\n", ("", ('H', "fermata")))]
    #[case("~=!roll!\n", ("", ('~', "roll")))]
    #[case("T = !trill!\n", ("", ('T', "trill")))]
    #[case("T=trill\n", ("", ('T', "trill")))]
    fn test_user_defined_symbol(#[case] input: &str, #[case] expected: (&str, (char, &str))) {
        let result = user_defined_symbol(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1\n", ("", ReferenceNumber(1)))]
    #[case("42\n", ("", ReferenceNumber(42)))]
    #[case("123\n", ("", ReferenceNumber(123)))]
    fn test_reference_number(#[case] input: &str, #[case] expected: (&str, ReferenceNumber)) {
        let result = reference_number(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("C\n", ("", MeterSymbol::CommonTime))]
    #[case("C|\n", ("", MeterSymbol::CutTime))]
    #[case("4/4\n", ("", MeterSymbol::Explicit(Meter { numerator: 4, denominator: 4 })))]
    #[case("3/4\n", ("", MeterSymbol::Explicit(Meter { numerator: 3, denominator: 4 })))]
    #[case("6/8\n", ("", MeterSymbol::Explicit(Meter { numerator: 6, denominator: 8 })))]
    fn test_meter_symbol(#[case] input: &str, #[case] expected: (&str, MeterSymbol)) {
        let result = meter_symbol(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1/8\n", ("", Duration::new(1, 8)))]
    #[case("1/16\n", ("", Duration::new(1, 16)))]
    #[case("1/4\n", ("", Duration::new(1, 4)))]
    fn test_unit_note_length(#[case] input: &str, #[case] expected: (&str, Duration)) {
        let result = unit_note_length(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("1/4=120\n", ("", TempoMarking { note_value: Duration::new(1, 4), beats_per_minute: 120 }))]
    #[case("1/8=180\n", ("", TempoMarking { note_value: Duration::new(1, 8), beats_per_minute: 180 }))]
    #[case("3/8=50\n", ("", TempoMarking { note_value: Duration::new(3, 8), beats_per_minute: 50 }))]
    fn test_tempo_marking(#[case] input: &str, #[case] expected: (&str, TempoMarking)) {
        let result = tempo_marking(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("C\n", PitchClass::C, Mode::Major)]
    #[case("Am\n", PitchClass::A, Mode::Minor)]
    #[case("G\n", PitchClass::G, Mode::Major)]
    fn test_key_signature(#[case] input: &str, #[case] tonic: PitchClass, #[case] mode: Mode) {
        let result = key_signature(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic,
                    accidental: None,
                    mode,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_hp() {
        // K:HP - Highland Bagpipe
        let result = key_signature("HP\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::A,
                    accidental: None,
                    mode: Mode::HighlandBagpipe,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_hp_lower() {
        // K:Hp - Highland Bagpipe (lowercase p)
        let result = key_signature("Hp\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::A,
                    accidental: None,
                    mode: Mode::HighlandBagpipeLower,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_none() {
        // K:none - No key signature
        let result = key_signature("none\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::C,
                    accidental: None,
                    mode: Mode::None,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_explicit_accidentals() {
        use crate::types::Accidental;

        // K:D =c - D major with C natural
        let result = key_signature("D =c\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: vec![(PitchClass::C, Accidental::Natural)],
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_multiple_explicit_accidentals() {
        use crate::types::Accidental;

        // K:Am ^f ^g - A minor with F# and G#
        let result = key_signature("Am ^f ^g\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::A,
                    accidental: None,
                    mode: Mode::Minor,
                    explicit_accidentals: vec![
                        (PitchClass::F, Accidental::Sharp),
                        (PitchClass::G, Accidental::Sharp),
                    ],
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_middle_parameter() {
        // K:D middle=d
        let result = key_signature("D middle=d\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: Some(PitchClass::D),
                    stafflines: None,
                }
            )
        );
    }

    #[test]
    fn test_key_signature_stafflines_parameter() {
        // K:G stafflines=5
        let result = key_signature("G stafflines=5\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::G,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: Some(5),
                }
            )
        );
    }

    #[test]
    fn test_key_signature_both_rendering_parameters() {
        // K:D middle=d stafflines=5
        let result = key_signature("D middle=d stafflines=5\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: Some(PitchClass::D),
                    stafflines: Some(5),
                }
            )
        );
    }

    #[test]
    fn test_key_signature_mode_and_parameters() {
        // K:D dor middle=d stafflines=4
        let result = key_signature("D dor middle=d stafflines=4\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Dorian,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: Some(PitchClass::D),
                    stafflines: Some(4),
                }
            )
        );
    }

    #[test]
    fn test_key_signature_all_parameters() {
        use crate::types::Accidental;

        // K:D =c middle=d stafflines=5
        let result = key_signature("D =c middle=d stafflines=5\n").unwrap();
        assert_eq!(
            result,
            (
                "",
                KeySignature {
                    tonic: PitchClass::D,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: vec![(PitchClass::C, Accidental::Natural)],
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: Some(PitchClass::D),
                    stafflines: Some(5),
                }
            )
        );
    }

    #[test]
    fn test_key_signature_stafflines_validation() {
        // stafflines=0 is out of range - parser ignores invalid parameter
        let (remaining, key) = key_signature("D stafflines=0\n").unwrap();
        assert_eq!(remaining, " stafflines=0\n"); // Parameter not consumed
        assert_eq!(key.stafflines, None); // Not set

        // stafflines=7 is out of range - parser ignores invalid parameter
        let (remaining, key) = key_signature("D stafflines=7\n").unwrap();
        assert_eq!(remaining, " stafflines=7\n"); // Parameter not consumed
        assert_eq!(key.stafflines, None); // Not set

        // stafflines=1 through 6 should succeed
        for n in 1..=6 {
            let input = format!("D stafflines={}\n", n);
            let result = key_signature(&input);
            assert!(result.is_ok(), "stafflines={} should be valid", n);
            let (remaining, key) = result.unwrap();
            assert_eq!(remaining, "");
            assert_eq!(key.stafflines, Some(n));
        }
    }

    #[test]
    fn test_key_signature_parameters_order() {
        use crate::types::Accidental;

        // Parameters can appear in any order
        let result1 = key_signature("D middle=d stafflines=5 =c\n").unwrap();
        let result2 = key_signature("D =c stafflines=5 middle=d\n").unwrap();
        let result3 = key_signature("D stafflines=5 =c middle=d\n").unwrap();

        // All should produce the same result
        let expected = KeySignature {
            tonic: PitchClass::D,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: vec![(PitchClass::C, Accidental::Natural)],
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: Some(PitchClass::D),
            stafflines: Some(5),
        };

        assert_eq!(result1, ("", expected.clone()));
        assert_eq!(result2, ("", expected.clone()));
        assert_eq!(result3, ("", expected));
    }
}
