//! Tune parser.
//!
//! This module provides a parser for complete ABC tunes, including both the
//! header (metadata and musical parameters) and body (musical notation).

use std::rc::Rc;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, line_ending, multispace0, space0},
    combinator::{cut, map, not, opt, peek},
    error::context,
    multi::{count, many0, many_till},
    sequence::{pair, preceded, terminated},
};

use crate::{
    parse::{
        annotation::annotation,
        barline::barline_or_variant,
        chord::{guitar_chord, note_chord},
        decoration::decoration,
        directive::directive,
        grace::grace_notes,
        information_field::{
            key_signature, meter_symbol, reference_number, tempo_marking, unit_note_length,
            user_defined_symbol,
        },
        inline_field::inline_field,
        lyrics::lyric_line,
        note::{note, rest},
        rhythm::tuplet,
        tie_slur::tie_or_slur,
        utils::till_line_ending,
        voice::{voice_declaration, voice_overlay},
    },
    types::ast::{BodyElement, InformationField, MacroDefinition, Tune, TuneBody, TuneHeader},
};

/// Parse required and musical parameter fields (X:, T:, K:, M:, L:, Q:).
fn required_fields(input: &str) -> IResult<&str, InformationField<'_>> {
    alt((
        map(directive, InformationField::Directive),
        map(preceded(tag("X:"), cut(reference_number)), InformationField::ReferenceNumber),
        map(preceded(tag("T:"), cut(till_line_ending)), InformationField::Title),
        map(preceded(tag("K:"), cut(key_signature)), InformationField::Key),
        map(preceded(tag("M:"), cut(meter_symbol)), InformationField::Meter),
        map(preceded(tag("L:"), cut(unit_note_length)), InformationField::UnitNoteLength),
        map(preceded(tag("Q:"), cut(tempo_marking)), InformationField::Tempo),
    ))
    .parse(input)
}

/// Parse tune metadata fields (C:, R:, O:, A:, B:, N:, S:, Z:, F:, D:, G:, H:).
fn metadata_fields(input: &str) -> IResult<&str, InformationField<'_>> {
    alt((
        map(preceded(tag("C:"), cut(till_line_ending)), InformationField::Composer),
        map(preceded(tag("R:"), cut(till_line_ending)), InformationField::Rhythm),
        map(preceded(tag("O:"), cut(till_line_ending)), InformationField::Origin),
        map(preceded(tag("A:"), cut(till_line_ending)), InformationField::Area),
        map(preceded(tag("B:"), cut(till_line_ending)), InformationField::Book),
        map(preceded(tag("N:"), cut(till_line_ending)), InformationField::Notes),
        map(preceded(tag("S:"), cut(till_line_ending)), InformationField::Source),
        map(preceded(tag("Z:"), cut(till_line_ending)), InformationField::Transcription),
        map(preceded(tag("F:"), cut(till_line_ending)), InformationField::FileUrl),
        map(preceded(tag("D:"), cut(till_line_ending)), InformationField::Discography),
        map(preceded(tag("G:"), cut(till_line_ending)), InformationField::Group),
        map(preceded(tag("H:"), cut(till_line_ending)), InformationField::History),
    ))
    .parse(input)
}

/// Parse structure and special fields (V:, P:, W:, I:, U:, m:, r:, s:).
fn structure_fields(input: &str) -> IResult<&str, InformationField<'_>> {
    alt((
        map(preceded(tag("V:"), cut(voice_declaration)), InformationField::Voice),
        map(preceded(tag("P:"), cut(till_line_ending)), InformationField::Parts),
        map(preceded(tag("W:"), cut(till_line_ending)), InformationField::Words),
        map(preceded(tag("I:"), cut(till_line_ending)), InformationField::Instruction),
        map(
            preceded(tag("U:"), cut(user_defined_symbol)),
            |(symbol, decoration)| InformationField::UserDefined { symbol, decoration },
        ),
        map(preceded(tag("m:"), cut(macro_definition)), InformationField::Macro),
        map(preceded(tag("r:"), cut(till_line_ending)), InformationField::Remark),
        map(preceded(tag("s:"), cut(till_line_ending)), InformationField::SymbolLine),
    ))
    .parse(input)
}

/// Parse a single information field line.
///
/// Information fields have the format `X:content` where X is a single
/// character field identifier. Directives (`%%...`) are also supported.
fn information_field_line(input: &str) -> IResult<&str, InformationField<'_>> {
    let (input, _) = space0(input)?;

    context(
        "information field",
        alt((required_fields, metadata_fields, structure_fields)),
    )
    .parse(input)
}

/// Parse a K: (key signature) field line only.
fn key_field_line(input: &str) -> IResult<&str, InformationField<'_>> {
    let (input, _) = space0(input)?;
    context(
        "key field",
        map(
            preceded(tag("K:"), cut(key_signature)),
            InformationField::Key,
        ),
    )
    .parse(input)
}

/// Parse a tune header.
///
/// The header consists of information fields starting with X: (reference number)
/// and ending with K: (key signature). The K: field must be last.
pub(crate) fn tune_header(input: &str) -> IResult<&str, TuneHeader<'_>> {
    context(
        "tune header",
        map(
            many_till(
                terminated(information_field_line, many0(line_ending)),
                terminated(key_field_line, many0(line_ending)),
            ),
            |(mut fields, key_field)| {
                fields.push(key_field);
                TuneHeader { fields }
            },
        ),
    )
    .parse(input)
}

/// Parse a single body element for macro content (single line only).
///
/// Unlike body_element which can span multiple lines, this parser stops
/// at newlines since macro definitions are single-line.
fn macro_body_element(input: &str) -> IResult<&str, BodyElement<'_>> {
    // Only consume spaces (not newlines) before the element
    preceded(
        space0,
        alt((
            // Don't include macro_invocation here to avoid consuming
            // letters that could be part of subsequent fields
            tuplet_with_notes,
            map(grace_notes, BodyElement::GraceNotes),
            map(decoration, BodyElement::Decoration),
            tie_or_slur,
            map(annotation, BodyElement::Annotation),
            map(note, BodyElement::Note),
            map(rest, BodyElement::Rest),
            map(note_chord, BodyElement::Chord),
            map(guitar_chord, BodyElement::GuitarChord),
        )),
    )
    .parse(input)
}

fn macro_body_elements(input: &str) -> IResult<&str, Vec<BodyElement<'_>>> {
    // Parse body elements until we hit a newline or end of input
    let (input, _) = space0(input)?;
    let (remaining, elements) = many0(macro_body_element).parse(input)?;

    // Consume the trailing newline if present
    let (remaining, _) = opt(line_ending).parse(remaining)?;

    Ok((remaining, elements))
}

/// Parse a macro invocation in the tune body.
///
/// ABC supports two macro invocation formats:
///
/// 1. `~n` where n is a digit 1-9 (e.g., `~1`, `~2`)
///    These are defined with `m:~1=content` in the header.
///
/// 2. Single letters h-w or H-W (e.g., `n`, `H`)
///    These are defined with `m:n=content` in the header.
///    Letters h-w are specifically reserved for macros in ABC since they
///    are not valid note names (notes are A-G only).
///
/// Example: If `m:~1={gf}` is defined, then `~1C` expands to `{gf}C`.
fn macro_invocation(input: &str) -> IResult<&str, BodyElement<'_>> {
    context(
        "macro invocation",
        alt((
            // ~n format (digit 1-9)
            map(
                preceded(char('~'), nom::character::complete::one_of("123456789")),
                BodyElement::MacroInvocation,
            ),
            // Plain letter format (h-w, H-W - not valid note names)
            // BUT: don't match if followed by:
            // - a note letter (A-G, a-g): indicates decoration prefix + note, not macro
            // - a colon: indicates a field identifier (like V: for voice), not macro
            map(
                terminated(
                    nom::character::complete::one_of("hijklmnopqrstuvwHIJKLMNOPQRSTUVW"),
                    not(peek(nom::character::complete::one_of("ABCDEFGabcdefg:"))),
                ),
                BodyElement::MacroInvocation,
            ),
        )),
    )
    .parse(input)
}

/// Parse a macro symbol (the name part of a macro definition).
///
/// ABC supports two macro symbol formats:
/// - `~n` where n is a digit 1-9 (e.g., `~1`, `~2`)
/// - Any single printable character (e.g., `n`, `h`)
///
/// For `~n` macros, only the digit is stored as the symbol (the `~` is
/// implicit). For plain character macros, the character itself is stored.
fn macro_symbol(input: &str) -> IResult<&str, char> {
    alt((
        // ~n format: store just the digit
        preceded(char('~'), nom::character::complete::one_of("123456789")),
        // Plain character format
        anychar,
    ))
    .parse(input)
}

/// Parse a macro definition (m: field).
///
/// Format: `m:symbol=body_elements`
///
/// Examples:
/// - `m:~1={gf}` defines macro '1' (invoked with `~1`)
/// - `m:n=cdef` defines macro 'n' (plain character macro)
fn macro_definition(input: &str) -> IResult<&str, Rc<MacroDefinition<'_>>> {
    context(
        "macro definition",
        map(
            (
                // The symbol being defined
                macro_symbol,
                // Optional whitespace before =
                space0,
                // Equals sign
                char('='),
                // Optional whitespace after =
                space0,
                // The body elements
                macro_body_elements,
            ),
            |(symbol, _, _, _, content)| {
                Rc::new(MacroDefinition { symbol, content })
            },
        ),
    )
    .parse(input)
}

/// Parse a tuplet with its contained notes.
///
/// Tuplets are written as `(p:q:r` followed by exactly `p` notes or elements.
/// For example, `(3ABC` is a triplet of three notes.
fn tuplet_with_notes(input: &str) -> IResult<&str, BodyElement<'_>> {
    context("tuplet with notes", |input| {
        let (input, spec) = tuplet(input)?;
        let (input, elements) = count(body_element, spec.p as usize).parse(input)?;
        Ok((input, BodyElement::Tuplet { spec, elements }))
    })
    .parse(input)
}

/// Parse a single body element.
fn body_element(input: &str) -> IResult<&str, BodyElement<'_>> {
    context(
        "body element",
        preceded(
            multispace0,
            alt((
                tuplet_with_notes,
                map(grace_notes, BodyElement::GraceNotes),
                // V: Voice switch - must come before macro_invocation since 'V' is
                // in the H-W range reserved for macros, but "V:" is a field code.
                // In the body, V: switches to a different voice for subsequent notes.
                map(
                    preceded(tag("V:"), cut(voice_declaration)),
                    BodyElement::VoiceSwitch,
                ),
                // w: Lyric line - must come before macro_invocation since 'w' is
                // in the h-w range reserved for macros, but "w:" is a field code
                map(preceded(tag("w:"), lyric_line), BodyElement::LyricLine),
                // Macro invocation must come before decoration to correctly parse
                // ~1 as macro vs ~C as decoration+note
                macro_invocation,
                map(decoration, BodyElement::Decoration),
                tie_or_slur,
                voice_overlay,
                map(annotation, BodyElement::Annotation),
                // %% Directive (must come before notes since %% is a specific prefix)
                map(directive, BodyElement::Directive),
                map(note, BodyElement::Note),
                map(rest, BodyElement::Rest),
                // Note chords `[CEG]` and inline fields `[K:D]` both use square brackets.
                // They're distinguished by content: note_chord expects note letters (A-G),
                // while inline_field expects a field identifier followed by colon (e.g., "K:").
                // Since valid notes won't match the "X:" pattern, note_chord naturally fails
                // on inline fields, allowing the alternation to fall through.
                map(note_chord, BodyElement::Chord),
                map(guitar_chord, BodyElement::GuitarChord),
                map(barline_or_variant, |(bar, variant)| {
                    variant
                        .map(BodyElement::VariantEnding)
                        .unwrap_or(BodyElement::BarLine(bar))
                }),
                map(inline_field, BodyElement::InlineField),
            )),
        ),
    )
    .parse(input)
}

/// Parse a tune body.
///
/// The body consists of musical notation elements: notes, rests, chords,
/// bar lines, and inline fields.
pub(crate) fn tune_body(input: &str) -> IResult<&str, TuneBody<'_>> {
    context(
        "tune body",
        map(terminated(many0(body_element), multispace0), |elements| {
            TuneBody { elements }
        }),
    )
    .parse(input)
}

/// Parse a complete tune.
///
/// A tune consists of a header (ending with K:) followed by a body
/// containing musical notation.
pub(crate) fn tune(input: &str) -> IResult<&str, Tune<'_>> {
    context(
        "tune",
        map(pair(tune_header, tune_body), |(header, body)| Tune {
            header,
            body,
        }),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::{KeySignature, Mode, PitchClass, ReferenceNumber};

    #[test]
    fn test_simple_tune_header() {
        let input = "X:1\nT:Test Tune\nK:G\n";
        let (remaining, header) = tune_header(input).unwrap();

        assert_eq!(header.fields.len(), 3);
        assert!(matches!(
            header.fields[0],
            InformationField::ReferenceNumber(_)
        ));
        assert!(matches!(header.fields[1], InformationField::Title(_)));
        assert!(matches!(header.fields[2], InformationField::Key(_)));
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_tune_header_with_metadata() {
        let input = "X:1\nT:Test\nC:Trad\nM:4/4\nL:1/8\nK:D\n";
        let (remaining, header) = tune_header(input).unwrap();

        assert_eq!(header.fields.len(), 6);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_simple_tune_body() {
        let input = "GABc|dedB|\n";
        let (remaining, body) = tune_body(input).unwrap();

        // Should have: G, A, B, c, |, d, e, d, B, |
        assert!(body.elements.len() >= 8);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_complete_tune() {
        let input = "X:1\nT:Simple\nK:C\nCDEF|GABc|\n";
        let (remaining, parsed_tune) = tune(input).unwrap();

        assert_eq!(parsed_tune.header.fields.len(), 3);
        assert!(!parsed_tune.body.elements.is_empty());
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_tune_with_inline_field() {
        let input = "X:1\nT:Test\nK:C\nCDEF|[K:G]GABc|\n";
        let (remaining, parsed_tune) = tune(input).unwrap();

        // Should parse the inline field [K:G]
        let has_inline = parsed_tune
            .body
            .elements
            .iter()
            .any(|e| matches!(e, BodyElement::InlineField(_)));
        assert!(has_inline);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_tune_with_parts_field() {
        let input = "X:1\nT:Test with Parts\nP:AABB\nK:C\nCDEF|\n";
        let (remaining, header) = tune_header(input).unwrap();

        let expected = TuneHeader {
            fields: vec![
                InformationField::ReferenceNumber(ReferenceNumber(1)),
                InformationField::Title("Test with Parts"),
                InformationField::Parts("AABB"),
                InformationField::Key(KeySignature {
                    tonic: PitchClass::C,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
            ],
        };

        assert_eq!((remaining, header), ("CDEF|\n", expected));
    }

    #[test]
    fn test_tune_with_lyrics() {
        use crate::types::ast::lyrics::{LyricLine, LyricSyllable};
        use crate::types::ast::note::{NoteDuration, NotePitch};
        use crate::types::Octave;

        let input = "X:1\nT:Test\nK:C\nC|\nw:Do\n";
        let (remaining, parsed_tune) = tune(input).unwrap();

        let expected = Tune {
            header: TuneHeader {
                fields: vec![
                    InformationField::ReferenceNumber(ReferenceNumber(1)),
                    InformationField::Title("Test"),
                    InformationField::Key(KeySignature {
                        tonic: PitchClass::C,
                        accidental: None,
                        mode: Mode::Major,
                        explicit_accidentals: Vec::new(),
                        clef: None,
                        transpose: None,
                        octave_shift: None,
                    middle: None,
                    stafflines: None,
                    }),
                ],
            },
            body: TuneBody {
                elements: vec![
                    BodyElement::Note(crate::types::ast::Note {
                        pitch: NotePitch {
                            base: PitchClass::C,
                            accidental: None,
                            octave: Octave(0),
                        },
                        duration: NoteDuration::Default,
                        decorations: Vec::new(),
                        broken_rhythm: None,
                    }),
                    BodyElement::BarLine(crate::types::ast::BarLine::Single),
                    BodyElement::LyricLine(LyricLine {
                        syllables: vec![LyricSyllable::Text {
                            text: "Do",
                            continues: false,
                        }],
                    }),
                ],
            },
        };

        assert_eq!((remaining, parsed_tune), ("", expected));
    }

    #[test]
    fn test_tune_with_user_defined_symbol() {
        let input = "X:1\nT:Test\nU:T=!trill!\nK:C\n";
        let (remaining, header) = tune_header(input).unwrap();

        let expected = TuneHeader {
            fields: vec![
                InformationField::ReferenceNumber(ReferenceNumber(1)),
                InformationField::Title("Test"),
                InformationField::UserDefined {
                    symbol: 'T',
                    decoration: "trill",
                },
                InformationField::Key(KeySignature {
                    tonic: PitchClass::C,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
            ],
        };

        assert_eq!((remaining, header), ("", expected));
    }

    #[test]
    fn test_tune_with_macro_definition() {
        use crate::types::ast::note::{NoteDuration, NotePitch};
        use crate::types::Octave;

        let input = "X:1\nT:Test\nm:n=CD\nK:C\n";
        let (remaining, header) = tune_header(input).unwrap();

        let expected = TuneHeader {
            fields: vec![
                InformationField::ReferenceNumber(ReferenceNumber(1)),
                InformationField::Title("Test"),
                InformationField::Macro(Rc::new(MacroDefinition {
                    symbol: 'n',
                    content: vec![
                        BodyElement::Note(crate::types::ast::Note {
                            pitch: NotePitch {
                                base: PitchClass::C,
                                accidental: None,
                                octave: Octave(0),
                            },
                            duration: NoteDuration::Default,
                            decorations: Vec::new(),
                            broken_rhythm: None,
                        }),
                        BodyElement::Note(crate::types::ast::Note {
                            pitch: NotePitch {
                                base: PitchClass::D,
                                accidental: None,
                                octave: Octave(0),
                            },
                            duration: NoteDuration::Default,
                            decorations: Vec::new(),
                            broken_rhythm: None,
                        }),
                    ],
                })),
                InformationField::Key(KeySignature {
                    tonic: PitchClass::C,
                    accidental: None,
                    mode: Mode::Major,
                    explicit_accidentals: Vec::new(),
                    clef: None,
                    transpose: None,
                    octave_shift: None,
                    middle: None,
                    stafflines: None,
                }),
            ],
        };

        assert_eq!((remaining, header), ("", expected));
    }
}
