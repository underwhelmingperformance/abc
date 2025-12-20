//! MIDI directives in the AST.
//!
//! This module defines types for representing MIDI directives as they appear
//! in ABC notation. MIDI directives control how ABC tunes are converted to
//! MIDI for playback, specifying channels, instruments, dynamics, and other
//! performance parameters.

use crate::types::{Channel, Instrument, midi::Velocity};

/// A MIDI directive controlling playback parameters.
///
/// MIDI directives in ABC notation are written as `%%MIDI` followed by the
/// directive name and parameters. They control various aspects of MIDI
/// generation including channel assignment, instrument selection, dynamics,
/// and ornament interpretation.
///
/// # Examples
///
/// ```text
/// %%MIDI channel 1
/// %%MIDI program 1 0          % Channel 1, program 0 (piano)
/// %%MIDI transpose -2         % Transpose down 2 semitones
/// %%MIDI beat 127 100 80 60   % Strong, medium, weak, very weak beats
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiDirective<'input> {
    /// `%%MIDI channel N` - Set the MIDI channel for this voice.
    ///
    /// Channels 1-16 are available (channel 10 is typically percussion).
    Channel(Channel),

    /// `%%MIDI program [channel] program` - Set the MIDI program (instrument).
    ///
    /// If channel is specified, sets the program for that channel only.
    /// Otherwise, sets the program for the current voice's channel.
    Program {
        /// Optional channel number to set the program for.
        channel: Option<Channel>,
        /// The instrument to use.
        program: Instrument,
    },

    /// `%%MIDI beat f1 f2 f3 f4` - Set beat pattern dynamics.
    ///
    /// Specifies velocity values for the first beat (strong), subsequent
    /// strong beats, weak beats, and very weak beats in a measure.
    Beat {
        /// Velocity for the first beat of each measure.
        first: Velocity,
        /// Velocity for strong beats.
        strong: Velocity,
        /// Velocity for weak beats.
        weak: Velocity,
        /// Velocity for very weak beats.
        very_weak: Velocity,
    },

    /// `%%MIDI transpose N` - Transpose this voice by N semitones.
    ///
    /// Positive values transpose up, negative values transpose down.
    Transpose(i8),

    /// `%%MIDI gchord STRING` - Set guitar chord accompaniment style.
    ///
    /// Controls how guitar chord symbols are converted to MIDI.
    GChord(&'input str),

    /// `%%MIDI chordprog N` - Set the instrument for chord accompaniment.
    ChordProg(Instrument),

    /// `%%MIDI bassprog N` - Set the instrument for bass accompaniment.
    BassProg(Instrument),

    /// `%%MIDI drum STRING d1 d2 ... dn v1 v2 ... vn` - Define drum pattern.
    ///
    /// Defines a rhythmic drum pattern with note-on positions and velocities.
    Drum {
        /// The rhythm pattern string (e.g., "d2zdd 70 60").
        pattern: &'input str,
    },

    /// `%%MIDI drumbars N` - Number of bars over which drum pattern repeats.
    DrumBars(u8),

    /// `%%MIDI drumoff` - Turn off drum accompaniment.
    DrumOff,

    /// `%%MIDI drumon` - Turn on drum accompaniment.
    DrumOn,

    /// `%%MIDI gchordon` - Turn on guitar chord accompaniment.
    GChordOn,

    /// `%%MIDI gchordoff` - Turn off guitar chord accompaniment.
    GChordOff,

    /// `%%MIDI bassvol N` - Set bass volume (0-127).
    BassVol(Velocity),

    /// `%%MIDI chordvol N` - Set chord volume (0-127).
    ChordVol(Velocity),

    /// `%%MIDI grace NUM/DENOM` - Set grace note duration.
    ///
    /// Specifies the fraction of the following note's duration that
    /// grace notes should occupy.
    Grace {
        /// Numerator of the grace note duration fraction.
        numerator: u8,
        /// Denominator of the grace note duration fraction.
        denominator: u8,
    },

    /// `%%MIDI gracedivider N` - Alternative grace note duration specification.
    GraceDivider(u8),

    /// `%%MIDI makechordchannels N` - Distribute chord notes across N channels.
    MakeChordChannels(u8),

    /// `%%MIDI randomchordattack N` - Randomize chord note timing (0-100).
    RandomChordAttack(u8),

    /// `%%MIDI chordattack N` - Delay between chord notes in MIDI ticks.
    ChordAttack(u8),

    /// `%%MIDI control CHANNEL CONTROLLER VALUE` - Send MIDI control change.
    Control {
        /// The MIDI channel.
        channel: Channel,
        /// The controller number (0-127).
        controller: u8,
        /// The controller value (0-127).
        value: u8,
    },

    /// `%%MIDI portamento N` - Set portamento (pitch slide) time.
    Portamento(u8),

    /// `%%MIDI pitchbend [CHANNEL] VALUE` - Send pitch bend message.
    PitchBend {
        /// Optional channel (uses current voice's channel if not specified).
        channel: Option<Channel>,
        /// Pitch bend value (-8192 to 8191, where 0 is no bend).
        value: i16,
    },

    /// `%%MIDI fermataproportional` - Make fermata length proportional.
    FermataProportional,

    /// `%%MIDI fermata NUMERATOR/DENOMINATOR` - Set fermata duration.
    Fermata {
        /// Numerator of the fermata duration multiplier.
        numerator: u8,
        /// Denominator of the fermata duration multiplier.
        denominator: u8,
    },

    /// `%%MIDI expand` - Expand repeats and variants in MIDI output.
    Expand,

    /// `%%MIDI noexpand` - Do not expand repeats and variants.
    NoExpand,

    /// `%%MIDI rtranspose N` - Relative transposition (cumulative with transpose).
    ///
    /// This adds to any existing transpose value. Multiple rtranspose directives
    /// accumulate, unlike transpose which sets an absolute value.
    Rtranspose(i8),

    /// `%%MIDI beatstring "pattern"` - Alternative beat pattern using a string.
    ///
    /// Defines beat emphasis using characters like 'f' (forte) and 'p' (piano).
    /// Example: "fmfp" for strong, medium, strong, weak.
    BeatString(&'input str),

    /// `%%MIDI ratio N M` - Override broken rhythm ratios.
    ///
    /// Sets custom ratios for broken rhythm operators instead of the default
    /// 3:2 (for single dot) and 7:4 (for double dot).
    Ratio(u8, u8),

    /// `%%MIDI chordname NAME notes...` - Custom chord shape definition.
    ///
    /// Defines a custom chord shape with a name and list of note offsets
    /// (in semitones) from the root. Stored in document-level chord table.
    ChordName {
        /// The name of the chord (e.g., "maj7", "dim").
        name: &'input str,
        /// Note offsets in semitones from the root.
        notes: Vec<i8>,
    },
}
