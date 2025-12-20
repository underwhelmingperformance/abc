//! Instrument specifications for playback.
//!
//! This module defines the instrument types used to specify which sound/timbre
//! should be used when playing back musical events. The instruments correspond
//! to the General MIDI Level 1 sound set, which defines 128 standard instruments
//! organized into 16 families.

/// An instrument specification for playback.
///
/// Instruments are organized according to the General MIDI Level 1 standard,
/// which defines 128 instruments across 16 families. Each instrument is assigned
/// a number from 0-127.
///
/// The `Unknown` variant allows preservation of extended sound banks beyond the
/// General MIDI standard.
///
/// # Examples
///
/// ```
/// use abc::types::Instrument;
///
/// let flute = Instrument::Flute;
///
/// // Convert from u8
/// let from_number: Instrument = 73_u8.into();
/// assert_eq!(from_number, Instrument::Flute);
///
/// // Unknown values are preserved
/// let unknown: Instrument = 200_u8.into();
/// assert_eq!(unknown, Instrument::Unknown(200));
///
/// // Display shows human-readable name
/// assert_eq!(flute.to_string(), "Flute");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, num_enum::FromPrimitive, strum::Display)]
#[repr(u8)]
#[strum(serialize_all = "title_case")]
pub enum Instrument {
    // Piano (0-7)
    AcousticGrandPiano = 0,
    BrightAcousticPiano = 1,
    ElectricGrandPiano = 2,
    HonkyTonkPiano = 3,
    ElectricPiano1 = 4,
    ElectricPiano2 = 5,
    Harpsichord = 6,
    Clavinet = 7,

    // Chromatic Percussion (8-15)
    Celesta = 8,
    Glockenspiel = 9,
    MusicBox = 10,
    Vibraphone = 11,
    Marimba = 12,
    Xylophone = 13,
    TubularBells = 14,
    Dulcimer = 15,

    // Organ (16-23)
    DrawbarOrgan = 16,
    PercussiveOrgan = 17,
    RockOrgan = 18,
    ChurchOrgan = 19,
    ReedOrgan = 20,
    Accordion = 21,
    Harmonica = 22,
    TangoAccordion = 23,

    // Guitar (24-31)
    AcousticGuitarNylon = 24,
    AcousticGuitarSteel = 25,
    ElectricGuitarJazz = 26,
    ElectricGuitarClean = 27,
    ElectricGuitarMuted = 28,
    OverdrivenGuitar = 29,
    DistortionGuitar = 30,
    GuitarHarmonics = 31,

    // Bass (32-39)
    AcousticBass = 32,
    ElectricBassFinger = 33,
    ElectricBassPick = 34,
    FretlessBass = 35,
    SlapBass1 = 36,
    SlapBass2 = 37,
    SynthBass1 = 38,
    SynthBass2 = 39,

    // Strings (40-47)
    Violin = 40,
    Viola = 41,
    Cello = 42,
    Contrabass = 43,
    TremoloStrings = 44,
    PizzicatoStrings = 45,
    OrchestralHarp = 46,
    Timpani = 47,

    // Ensemble (48-55)
    StringEnsemble1 = 48,
    StringEnsemble2 = 49,
    SynthStrings1 = 50,
    SynthStrings2 = 51,
    ChoirAahs = 52,
    VoiceOohs = 53,
    SynthVoice = 54,
    OrchestraHit = 55,

    // Brass (56-63)
    Trumpet = 56,
    Trombone = 57,
    Tuba = 58,
    MutedTrumpet = 59,
    FrenchHorn = 60,
    BrassSection = 61,
    SynthBrass1 = 62,
    SynthBrass2 = 63,

    // Reed (64-71)
    SopranoSax = 64,
    AltoSax = 65,
    TenorSax = 66,
    BaritoneSax = 67,
    Oboe = 68,
    EnglishHorn = 69,
    Bassoon = 70,
    Clarinet = 71,

    // Pipe (72-79)
    Piccolo = 72,
    Flute = 73,
    Recorder = 74,
    PanFlute = 75,
    BlownBottle = 76,
    Shakuhachi = 77,
    Whistle = 78,
    Ocarina = 79,

    // Synth Lead (80-87)
    Lead1Square = 80,
    Lead2Sawtooth = 81,
    Lead3Calliope = 82,
    Lead4Chiff = 83,
    Lead5Charang = 84,
    Lead6Voice = 85,
    Lead7Fifths = 86,
    Lead8BassAndLead = 87,

    // Synth Pad (88-95)
    Pad1NewAge = 88,
    Pad2Warm = 89,
    Pad3Polysynth = 90,
    Pad4Choir = 91,
    Pad5Bowed = 92,
    Pad6Metallic = 93,
    Pad7Halo = 94,
    Pad8Sweep = 95,

    // Synth Effects (96-103)
    Fx1Rain = 96,
    Fx2Soundtrack = 97,
    Fx3Crystal = 98,
    Fx4Atmosphere = 99,
    Fx5Brightness = 100,
    Fx6Goblins = 101,
    Fx7Echoes = 102,
    #[strum(serialize = "FX 8 Sci-Fi")]
    Fx8SciFi = 103,

    // Ethnic (104-111)
    Sitar = 104,
    Banjo = 105,
    Shamisen = 106,
    Koto = 107,
    Kalimba = 108,
    Bagpipe = 109,
    Fiddle = 110,
    Shanai = 111,

    // Percussive (112-119)
    TinkleBell = 112,
    Agogo = 113,
    SteelDrums = 114,
    Woodblock = 115,
    TaikoDrum = 116,
    MelodicTom = 117,
    SynthDrum = 118,
    ReverseCymbal = 119,

    // Sound Effects (120-127)
    GuitarFretNoise = 120,
    BreathNoise = 121,
    Seashore = 122,
    BirdTweet = 123,
    TelephoneRing = 124,
    Helicopter = 125,
    Applause = 126,
    Gunshot = 127,

    /// An instrument not in the General MIDI standard.
    ///
    /// This variant preserves instrument numbers beyond the standard 128 GM instruments,
    /// allowing for extended sound banks in some synthesizers.
    #[num_enum(catch_all)]
    #[strum(serialize = "Unknown Instrument")]
    Unknown(u8),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, Instrument::AcousticGrandPiano)]
    #[case(73, Instrument::Flute)]
    #[case(127, Instrument::Gunshot)]
    #[case(200, Instrument::Unknown(200))]
    fn test_instrument_from_u8(#[case] value: u8, #[case] expected: Instrument) {
        assert_eq!(Instrument::from(value), expected);
    }

    #[rstest]
    #[case(Instrument::AcousticGrandPiano, "Acoustic Grand Piano")]
    #[case(Instrument::Flute, "Flute")]
    #[case(Instrument::Fx8SciFi, "FX 8 Sci-Fi")]
    #[case(Instrument::Unknown(200), "Unknown Instrument")]
    fn test_instrument_display(#[case] instrument: Instrument, #[case] expected: &str) {
        assert_eq!(instrument.to_string(), expected);
    }
}
