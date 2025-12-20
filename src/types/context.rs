//! Musical context types.
//!
//! This module defines the `Context` type, which represents the musical context
//! (key signature, metre, tempo, and unit note length) at a point in a tune.
//! Context is shared across measures using `Rc<T>` until it changes.

use std::rc::Rc;

use super::{Duration, Instrument, KeySignature, Meter, TempoMarking, midi::Velocity};

/// Musical context at a point in a tune.
///
/// Context tracks the key signature, metre, tempo, unit note length, and
/// playback parameters that apply to musical events. When context doesn't
/// change between measures, it is shared via `Rc<Context>` to avoid duplication.
///
/// # Examples
///
/// ```
/// use abc::types::*;
///
/// // Using the builder (recommended)
/// let context = Context::builder()
///     .key(KeySignature {
///         tonic: PitchClass::G,
///         accidental: None,
///         mode: Mode::Major,
///         explicit_accidentals: vec![],
///         clef: None,
///         transpose: None,
///         octave_shift: None,
///         middle: None,
///         stafflines: None,
///     })
///     .meter(Meter { numerator: 6, denominator: 8 })
///     .unit_length(Duration::new(1, 8))
///     .build();
///
/// assert_eq!(context.meter.numerator, 6);
/// assert_eq!(context.expand_repeats, true); // Default value
/// ```
#[derive(Debug, Clone, PartialEq, bon::Builder)]
#[builder(on(Rc<KeySignature>, into))]
#[builder(on(Rc<TempoMarking>, into))]
pub struct Context<'input> {
    /// The key signature in effect.
    pub key: Rc<KeySignature>,

    /// The metre (time signature) in effect.
    pub meter: Meter,

    /// The tempo marking, if specified.
    pub tempo: Option<Rc<TempoMarking>>,

    /// The unit note length (default duration for notes without explicit length).
    ///
    /// This is typically `1/8` or `1/16` depending on the metre.
    pub unit_length: Duration,

    // Playback parameters
    /// Beat pattern velocities (first, strong, weak, very_weak).
    ///
    /// Controls the relative dynamics of beats within a measure for playback.
    pub beat_pattern: Option<(Velocity, Velocity, Velocity, Velocity)>,

    /// Guitar chord accompaniment pattern.
    ///
    /// A string like "fzcz" specifying when to play bass (f) and chords (c).
    pub gchord_pattern: Option<&'input str>,

    /// Whether guitar chord accompaniment is enabled.
    #[builder(default)]
    pub gchord_enabled: bool,

    /// Instrument for chord accompaniment.
    pub chord_instrument: Option<Instrument>,

    /// Volume for chord accompaniment.
    pub chord_volume: Option<Velocity>,

    /// Instrument for bass accompaniment.
    pub bass_instrument: Option<Instrument>,

    /// Volume for bass accompaniment.
    pub bass_volume: Option<Velocity>,

    /// Grace note timing as a fraction of the following note (numerator, denominator).
    ///
    /// For example, `(1, 8)` means grace notes occupy 1/8 of the following note's duration.
    pub grace_timing: Option<(u8, u8)>,

    /// Drum pattern string.
    pub drum_pattern: Option<&'input str>,

    /// Whether drum accompaniment is enabled.
    #[builder(default)]
    pub drum_enabled: bool,

    /// Number of bars over which the drum pattern repeats.
    pub drum_bars: Option<u8>,

    /// Number of channels to distribute chord notes across.
    pub make_chord_channels: Option<u8>,

    /// Amount of randomization for chord note timing (0-100).
    pub random_chord_attack: Option<u8>,

    /// Delay between chord notes in MIDI ticks.
    pub chord_attack: Option<u8>,

    /// Portamento (pitch slide) time.
    pub portamento: Option<u8>,

    /// Whether fermata length is proportional to note duration.
    #[builder(default)]
    pub fermata_proportional: bool,

    /// Fermata duration multiplier (numerator, denominator).
    pub fermata: Option<(u8, u8)>,

    /// Whether to expand repeats and variants in output.
    #[builder(default = true)]
    pub expand_repeats: bool,

    /// Beat pattern string (alternative to beat_pattern velocities).
    ///
    /// A string like "fmfp" where characters represent beat emphasis levels.
    pub beat_string: Option<&'input str>,

    /// Custom broken rhythm ratio override (numerator, denominator).
    ///
    /// Overrides the default 3:2 and 7:4 ratios for broken rhythm operators.
    pub broken_rhythm_ratio: Option<(u8, u8)>,
}

impl<'input> Context<'input> {
    /// Wrap this context in an `Rc` for sharing across measures.
    ///
    /// # Examples
    ///
    /// ```
    /// use abc::types::{Context, Duration, KeySignature, Meter, Mode, PitchClass};
    /// use std::rc::Rc;
    ///
    /// let key = KeySignature {
    ///     tonic: PitchClass::C,
    ///     accidental: None,
    ///     mode: Mode::Major,
    ///     explicit_accidentals: Vec::new(),
    ///     clef: None,
    ///     transpose: None,
    ///     octave_shift: None,
    ///     middle: None,
    ///     stafflines: None,
    /// };
    ///
    /// let meter = Meter { numerator: 4, denominator: 4 };
    /// let context = Context {
    ///     key: Rc::new(key),
    ///     meter,
    ///     tempo: None,
    ///     unit_length: Duration::new(1, 8),
    ///     beat_pattern: None,
    ///     gchord_pattern: None,
    ///     gchord_enabled: false,
    ///     chord_instrument: None,
    ///     chord_volume: None,
    ///     bass_instrument: None,
    ///     bass_volume: None,
    ///     grace_timing: None,
    ///     drum_pattern: None,
    ///     drum_enabled: false,
    ///     drum_bars: None,
    ///     make_chord_channels: None,
    ///     random_chord_attack: None,
    ///     chord_attack: None,
    ///     portamento: None,
    ///     fermata_proportional: false,
    ///     fermata: None,
    ///     expand_repeats: true,
    ///     beat_string: None,
    ///     broken_rhythm_ratio: None,
    /// };
    /// let shared = context.into_rc();
    /// assert_eq!(Rc::strong_count(&shared), 1);
    /// ```
    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}
