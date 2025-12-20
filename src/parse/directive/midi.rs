//! MIDI directive parsing.
//!
//! This module parses `%%MIDI` directives from ABC notation.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag_no_case, take_while1},
    character::complete::{char, digit1, space1},
    combinator::{map_res, opt, value},
    multi::many0,
    sequence::delimited,
};

use crate::types::{Channel, Instrument, ast::MidiDirective, midi::Velocity};

/// Parse a MIDI directive (after `%%MIDI `).
///
/// This parses the content following `%%MIDI ` and returns the appropriate
/// `MidiDirective` variant.
pub(crate) fn midi_directive(input: &str) -> IResult<&str, MidiDirective<'_>> {
    // Split into groups to stay within nom's alt() tuple size limit (21)
    alt((
        alt((
            midi_channel,
            midi_program,
            midi_beat,
            midi_transpose,
            midi_gchord,
            midi_chordprog,
            midi_bassprog,
            midi_drum,
            midi_drumbars,
            midi_drumoff,
            midi_drumon,
            midi_gchordon,
            midi_gchordoff,
        )),
        alt((
            midi_bassvol,
            midi_chordvol,
            midi_grace,
            midi_gracedivider,
            midi_makechordchannels,
            midi_randomchordattack,
            midi_chordattack,
            midi_control,
            midi_portamento,
            midi_pitchbend,
            midi_fermataproportional,
            midi_fermata,
            midi_expand,
            midi_noexpand,
        )),
        alt((
            midi_rtranspose,
            midi_beatstring,
            midi_ratio,
            midi_chordname,
        )),
    ))
    .parse(input)
}

/// Parse `%%MIDI channel N`
fn midi_channel(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("channel").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, channel) = parse_channel(input)?;
    Ok((input, MidiDirective::Channel(channel)))
}

/// Parse `%%MIDI program [channel] N`
fn midi_program(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("program").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, first) = parse_u8(input)?;
    let (input, second) = opt((space1, parse_u8)).parse(input)?;

    let (channel, program) = match second {
        Some((_, prog)) => (Channel::new(first), Instrument::from(prog)),
        None => (None, Instrument::from(first)),
    };

    Ok((input, MidiDirective::Program { channel, program }))
}

/// Parse `%%MIDI beat f1 f2 f3 f4`
fn midi_beat(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("beat").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, first) = parse_velocity(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, strong) = parse_velocity(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, weak) = parse_velocity(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, very_weak) = parse_velocity(input)?;

    Ok((
        input,
        MidiDirective::Beat {
            first,
            strong,
            weak,
            very_weak,
        },
    ))
}

/// Parse `%%MIDI transpose N`
fn midi_transpose(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("transpose").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, semitones) = parse_i8(input)?;
    Ok((input, MidiDirective::Transpose(semitones)))
}

/// Parse `%%MIDI gchord STRING`
fn midi_gchord(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("gchord").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, pattern) = take_while1(|c: char| !c.is_whitespace()).parse(input)?;
    Ok((input, MidiDirective::GChord(pattern)))
}

/// Parse `%%MIDI chordprog N`
fn midi_chordprog(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("chordprog").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, instrument) = parse_instrument(input)?;
    Ok((input, MidiDirective::ChordProg(instrument)))
}

/// Parse `%%MIDI bassprog N`
fn midi_bassprog(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("bassprog").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, instrument) = parse_instrument(input)?;
    Ok((input, MidiDirective::BassProg(instrument)))
}

/// Parse `%%MIDI drum STRING`
fn midi_drum(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("drum").parse(input)?;
    let (input, _) = space1.parse(input)?;
    // Take the rest of the line as the pattern
    let (input, pattern) = take_while1(|c: char| c != '\n' && c != '\r').parse(input)?;
    Ok((input, MidiDirective::Drum { pattern }))
}

/// Parse `%%MIDI drumbars N`
fn midi_drumbars(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("drumbars").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, bars) = parse_u8(input)?;
    Ok((input, MidiDirective::DrumBars(bars)))
}

/// Parse `%%MIDI drumoff`
fn midi_drumoff(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::DrumOff, tag_no_case("drumoff")).parse(input)
}

/// Parse `%%MIDI drumon`
fn midi_drumon(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::DrumOn, tag_no_case("drumon")).parse(input)
}

/// Parse `%%MIDI gchordon`
fn midi_gchordon(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::GChordOn, tag_no_case("gchordon")).parse(input)
}

/// Parse `%%MIDI gchordoff`
fn midi_gchordoff(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::GChordOff, tag_no_case("gchordoff")).parse(input)
}

/// Parse `%%MIDI bassvol N`
fn midi_bassvol(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("bassvol").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, vol) = parse_velocity(input)?;
    Ok((input, MidiDirective::BassVol(vol)))
}

/// Parse `%%MIDI chordvol N`
fn midi_chordvol(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("chordvol").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, vol) = parse_velocity(input)?;
    Ok((input, MidiDirective::ChordVol(vol)))
}

/// Parse `%%MIDI grace NUM/DENOM`
fn midi_grace(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("grace").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, numerator) = parse_u8(input)?;
    let (input, _) = char('/').parse(input)?;
    let (input, denominator) = parse_u8(input)?;
    Ok((
        input,
        MidiDirective::Grace {
            numerator,
            denominator,
        },
    ))
}

/// Parse `%%MIDI gracedivider N`
fn midi_gracedivider(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("gracedivider").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, divider) = parse_u8(input)?;
    Ok((input, MidiDirective::GraceDivider(divider)))
}

/// Parse `%%MIDI makechordchannels N`
fn midi_makechordchannels(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("makechordchannels").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, channels) = parse_u8(input)?;
    Ok((input, MidiDirective::MakeChordChannels(channels)))
}

/// Parse `%%MIDI randomchordattack N`
fn midi_randomchordattack(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("randomchordattack").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, amount) = parse_u8(input)?;
    Ok((input, MidiDirective::RandomChordAttack(amount)))
}

/// Parse `%%MIDI chordattack N`
fn midi_chordattack(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("chordattack").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, delay) = parse_u8(input)?;
    Ok((input, MidiDirective::ChordAttack(delay)))
}

/// Parse `%%MIDI control CHANNEL CONTROLLER VALUE`
fn midi_control(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("control").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, channel) = parse_channel(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, controller) = parse_u8(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, value) = parse_u8(input)?;
    Ok((
        input,
        MidiDirective::Control {
            channel,
            controller,
            value,
        },
    ))
}

/// Parse `%%MIDI portamento N`
fn midi_portamento(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("portamento").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, time) = parse_u8(input)?;
    Ok((input, MidiDirective::Portamento(time)))
}

/// Parse `%%MIDI pitchbend [CHANNEL] VALUE`
fn midi_pitchbend(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("pitchbend").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, first) = parse_i16(input)?;
    let (input, second) = opt((space1, parse_i16)).parse(input)?;

    let (channel, value) = match second {
        Some((_, val)) => {
            let ch = if (1..=16).contains(&first) {
                Channel::new(first as u8)
            } else {
                None
            };
            (ch, val)
        }
        None => (None, first),
    };

    Ok((input, MidiDirective::PitchBend { channel, value }))
}

/// Parse `%%MIDI fermataproportional`
fn midi_fermataproportional(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(
        MidiDirective::FermataProportional,
        tag_no_case("fermataproportional"),
    )
    .parse(input)
}

/// Parse `%%MIDI fermata NUM/DENOM`
fn midi_fermata(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("fermata").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, numerator) = parse_u8(input)?;
    let (input, _) = char('/').parse(input)?;
    let (input, denominator) = parse_u8(input)?;
    Ok((
        input,
        MidiDirective::Fermata {
            numerator,
            denominator,
        },
    ))
}

/// Parse `%%MIDI expand`
fn midi_expand(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::Expand, tag_no_case("expand")).parse(input)
}

/// Parse `%%MIDI noexpand`
fn midi_noexpand(input: &str) -> IResult<&str, MidiDirective<'_>> {
    value(MidiDirective::NoExpand, tag_no_case("noexpand")).parse(input)
}

/// Parse a u8 value.
fn parse_u8(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>()).parse(input)
}

/// Parse a signed i8 value.
fn parse_i8(input: &str) -> IResult<&str, i8> {
    let (input, sign) = opt(alt((char('+'), char('-')))).parse(input)?;
    let (input, value) = map_res(digit1, |s: &str| s.parse::<i8>()).parse(input)?;
    let value = if sign == Some('-') { -value } else { value };
    Ok((input, value))
}

/// Parse a signed i16 value.
fn parse_i16(input: &str) -> IResult<&str, i16> {
    let (input, sign) = opt(alt((char('+'), char('-')))).parse(input)?;
    let (input, value) = map_res(digit1, |s: &str| s.parse::<i16>()).parse(input)?;
    let value = if sign == Some('-') { -value } else { value };
    Ok((input, value))
}

/// Parse a MIDI channel (1-16).
fn parse_channel(input: &str) -> IResult<&str, Channel> {
    let (input, n) = parse_u8(input)?;
    let channel = Channel::new(n).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;
    Ok((input, channel))
}

/// Parse an instrument number (0-255).
fn parse_instrument(input: &str) -> IResult<&str, Instrument> {
    let (input, n) = parse_u8(input)?;
    Ok((input, Instrument::from(n)))
}

/// Parse a MIDI velocity (0-127).
fn parse_velocity(input: &str) -> IResult<&str, Velocity> {
    let (input, n) = parse_u8(input)?;
    let velocity = Velocity::new(n).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;
    Ok((input, velocity))
}

/// Parse `%%MIDI rtranspose N`
fn midi_rtranspose(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("rtranspose").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, semitones) = parse_i8(input)?;
    Ok((input, MidiDirective::Rtranspose(semitones)))
}

/// Parse `%%MIDI beatstring "pattern"`
fn midi_beatstring(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("beatstring").parse(input)?;
    let (input, _) = space1.parse(input)?;
    // Parse quoted string
    let (input, pattern) = delimited(
        char('"'),
        take_while1(|c: char| c != '"'),
        char('"'),
    )
    .parse(input)?;
    Ok((input, MidiDirective::BeatString(pattern)))
}

/// Parse `%%MIDI ratio N M`
fn midi_ratio(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("ratio").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, num) = parse_u8(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, denom) = parse_u8(input)?;
    Ok((input, MidiDirective::Ratio(num, denom)))
}

/// Parse `%%MIDI chordname NAME notes...`
fn midi_chordname(input: &str) -> IResult<&str, MidiDirective<'_>> {
    let (input, _) = tag_no_case("chordname").parse(input)?;
    let (input, _) = space1.parse(input)?;
    // Parse chord name (non-whitespace string)
    let (input, name) = take_while1(|c: char| !c.is_whitespace()).parse(input)?;
    // Parse space-separated list of note numbers
    let (input, notes) = many0((space1, parse_i8).map(|(_, n)| n)).parse(input)?;
    Ok((input, MidiDirective::ChordName { name, notes }))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("channel 1", MidiDirective::Channel(Channel::new(1).unwrap()))]
    #[case("channel 10", MidiDirective::Channel(Channel::new(10).unwrap()))]
    #[case("channel 16", MidiDirective::Channel(Channel::new(16).unwrap()))]
    #[case("CHANNEL 5", MidiDirective::Channel(Channel::new(5).unwrap()))]
    fn test_midi_channel(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("program 0", MidiDirective::Program { channel: None, program: Instrument::AcousticGrandPiano })]
    #[case("program 127", MidiDirective::Program { channel: None, program: Instrument::Gunshot })]
    #[case("program 1 73", MidiDirective::Program { channel: Channel::new(1), program: Instrument::Flute })]
    fn test_midi_program(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_midi_beat() {
        let (remaining, result) = midi_directive("beat 127 100 80 60").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::Beat {
                first: Velocity::new(127).unwrap(),
                strong: Velocity::new(100).unwrap(),
                weak: Velocity::new(80).unwrap(),
                very_weak: Velocity::new(60).unwrap(),
            }
        );
    }

    #[rstest]
    #[case("transpose 0", MidiDirective::Transpose(0))]
    #[case("transpose 12", MidiDirective::Transpose(12))]
    #[case("transpose -12", MidiDirective::Transpose(-12))]
    #[case("transpose +5", MidiDirective::Transpose(5))]
    fn test_midi_transpose(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_midi_gchord() {
        let (remaining, result) = midi_directive("gchord czfczfcz").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, MidiDirective::GChord("czfczfcz"));
    }

    #[rstest]
    #[case("drumoff", MidiDirective::DrumOff)]
    #[case("drumon", MidiDirective::DrumOn)]
    #[case("gchordon", MidiDirective::GChordOn)]
    #[case("gchordoff", MidiDirective::GChordOff)]
    #[case("expand", MidiDirective::Expand)]
    #[case("noexpand", MidiDirective::NoExpand)]
    #[case("fermataproportional", MidiDirective::FermataProportional)]
    fn test_midi_simple_flags(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_midi_grace() {
        let (remaining, result) = midi_directive("grace 1/4").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::Grace {
                numerator: 1,
                denominator: 4
            }
        );
    }

    #[test]
    fn test_midi_control() {
        let (remaining, result) = midi_directive("control 1 7 100").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::Control {
                channel: Channel::new(1).unwrap(),
                controller: 7,
                value: 100,
            }
        );
    }

    #[test]
    fn test_midi_pitchbend() {
        // Without channel
        let (remaining, result) = midi_directive("pitchbend 4096").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::PitchBend {
                channel: None,
                value: 4096
            }
        );

        // With channel
        let (remaining, result) = midi_directive("pitchbend 1 -2048").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::PitchBend {
                channel: Channel::new(1),
                value: -2048
            }
        );
    }

    #[rstest]
    #[case("rtranspose 0", MidiDirective::Rtranspose(0))]
    #[case("rtranspose 5", MidiDirective::Rtranspose(5))]
    #[case("rtranspose -3", MidiDirective::Rtranspose(-3))]
    #[case("rtranspose +7", MidiDirective::Rtranspose(7))]
    #[case("RTRANSPOSE 2", MidiDirective::Rtranspose(2))]
    fn test_midi_rtranspose(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(r#"beatstring "fmfp""#, MidiDirective::BeatString("fmfp"))]
    #[case(r#"beatstring "fppp""#, MidiDirective::BeatString("fppp"))]
    #[case(r#"beatstring "ffmm""#, MidiDirective::BeatString("ffmm"))]
    #[case(r#"BEATSTRING "fzfz""#, MidiDirective::BeatString("fzfz"))]
    fn test_midi_beatstring(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("ratio 3 2", MidiDirective::Ratio(3, 2))]
    #[case("ratio 7 4", MidiDirective::Ratio(7, 4))]
    #[case("ratio 5 3", MidiDirective::Ratio(5, 3))]
    #[case("RATIO 2 1", MidiDirective::Ratio(2, 1))]
    fn test_midi_ratio(#[case] input: &str, #[case] expected: MidiDirective<'_>) {
        let (remaining, result) = midi_directive(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_midi_chordname() {
        // Simple chord with a few notes
        let (remaining, result) = midi_directive("chordname maj7 0 4 7 11").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::ChordName {
                name: "maj7",
                notes: vec![0, 4, 7, 11]
            }
        );

        // Chord with negative notes
        let (remaining, result) = midi_directive("chordname dim -3 0 3 6").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::ChordName {
                name: "dim",
                notes: vec![-3, 0, 3, 6]
            }
        );

        // Empty chord (no notes)
        let (remaining, result) = midi_directive("chordname empty").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::ChordName {
                name: "empty",
                notes: vec![]
            }
        );

        // Case insensitive
        let (remaining, result) = midi_directive("CHORDNAME test 0 1 2").unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            result,
            MidiDirective::ChordName {
                name: "test",
                notes: vec![0, 1, 2]
            }
        );
    }
}
