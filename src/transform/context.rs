//! Context building and tracking during transformation.
//!
//! This module provides utilities for building and tracking musical context
//! (key, metre, tempo, unit length) as we transform the AST into public types.

use std::{collections::HashMap, rc::Rc};

use crate::types::{
    Accidental, Clef, Context, Duration, Instrument, KeySignature, Meter, MeterSymbol, PitchClass,
    TempoMarking, VoiceId, midi::Velocity,
    ast::{StemDirection, VoiceAttributes},
};

/// Attributes that affect how a voice's notes are rendered and played.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResolvedVoiceAttributes<'input> {
    /// Display name for this voice.
    #[allow(dead_code)]
    pub name: Option<&'input str>,
    /// The clef to use for this voice.
    #[allow(dead_code)]
    pub clef: Option<Clef>,
    /// The direction of note stems.
    #[allow(dead_code)]
    pub stem: Option<StemDirection>,
    /// Octave shift for this voice (in octaves).
    ///
    /// Positive values shift up, negative values shift down.
    pub octave_shift: i8,
    /// Transposition for this voice (in semitones).
    ///
    /// Positive values transpose up, negative values transpose down.
    pub transpose: i8,
    /// The instrument to use for this voice.
    pub instrument: Option<Instrument>,
    /// The MIDI channel assigned to this voice.
    pub midi_channel: Option<crate::types::Channel>,
    /// Additional MIDI transpose for this voice (in semitones).
    ///
    /// This is separate from the voice transpose attribute and is set by
    /// `%%MIDI transpose` directives. It adds to the voice transpose.
    pub midi_transpose: i8,
    /// Relative MIDI transpose for this voice (in semitones).
    ///
    /// This is set by `%%MIDI rtranspose` directives and accumulates
    /// with each directive, unlike midi_transpose which sets an absolute value.
    pub midi_rtranspose: i8,
}

/// Tracks musical context state during transformation.
///
/// TransformContext maintains the current musical context (key, metre,
/// tempo, unit length) and tracks accidentals that are active within a measure.
/// Use `Context::builder()` to construct Context instances.
pub(crate) struct TransformContext<'input> {
    /// Current key signature.
    key: Rc<KeySignature>,

    /// Current metre.
    meter: Meter,

    /// Current tempo, if specified.
    tempo: Option<Rc<TempoMarking>>,

    /// Current unit note length (default duration for notes).
    unit_length: Duration,

    /// Active accidentals within the current measure.
    ///
    /// These are cleared when a bar line is encountered.
    active_accidentals: HashMap<PitchClass, Accidental>,

    /// Voice attributes per voice.
    ///
    /// These affect pitch resolution (transpose, octave) and rendering (clef, stem).
    pub(super) voice_attributes: HashMap<VoiceId<'input>, ResolvedVoiceAttributes<'input>>,

    // Playback parameters
    beat_pattern: Option<(Velocity, Velocity, Velocity, Velocity)>,
    gchord_pattern: Option<&'input str>,
    gchord_enabled: bool,
    chord_instrument: Option<Instrument>,
    chord_volume: Option<Velocity>,
    bass_instrument: Option<Instrument>,
    bass_volume: Option<Velocity>,
    grace_timing: Option<(u8, u8)>,
    drum_pattern: Option<&'input str>,
    drum_enabled: bool,
    drum_bars: Option<u8>,
    make_chord_channels: Option<u8>,
    random_chord_attack: Option<u8>,
    chord_attack: Option<u8>,
    portamento: Option<u8>,
    fermata_proportional: bool,
    fermata: Option<(u8, u8)>,
    expand_repeats: bool,
    beat_string: Option<&'input str>,
    broken_rhythm_ratio: Option<(u8, u8)>,
}

/// Macro to eliminate boilerplate in update methods.
///
/// This macro generates an update method that compares a new value with the current
/// field value and returns true if they differ (indicating a context change).
///
/// Patterns:
/// - `update_field!(self.field, value)` - Direct comparison and assignment
/// - `update_field!(self.field, Some(value))` - Wrap in Some before comparing/assigning
/// - `update_field!(self.field as deref, value)` - Use as_deref for comparison (for String/Rc/Option)
macro_rules! update_field {
    // Pattern for fields with Option<String> that need as_deref comparison
    ($self:ident.$field:ident as deref, $value:expr) => {
        if $self.$field.as_deref() != Some(&$value) {
            $self.$field = Some($value);
            true
        } else {
            false
        }
    };
    // Pattern for direct value assignment without Option wrapping
    ($self:ident.$field:ident, $value:expr) => {
        if $self.$field != $value {
            $self.$field = $value;
            true
        } else {
            false
        }
    };
    // Pattern for wrapping value in Some
    ($self:ident.$field:ident, Some($value:expr)) => {
        if $self.$field != Some($value) {
            $self.$field = Some($value);
            true
        } else {
            false
        }
    };
}

impl<'input> TransformContext<'input> {
    /// Create a new transform context with the given initial values.
    pub(crate) fn new(
        key: Rc<KeySignature>,
        meter: Meter,
        tempo: Option<Rc<TempoMarking>>,
        unit_length: Duration,
    ) -> Self {
        Self {
            key,
            meter,
            tempo,
            unit_length,
            active_accidentals: HashMap::new(),
            voice_attributes: HashMap::new(),
            beat_pattern: None,
            gchord_pattern: None,
            gchord_enabled: false,
            chord_instrument: None,
            chord_volume: None,
            bass_instrument: None,
            bass_volume: None,
            grace_timing: None,
            drum_pattern: None,
            drum_enabled: false,
            drum_bars: None,
            make_chord_channels: None,
            random_chord_attack: None,
            chord_attack: None,
            portamento: None,
            fermata_proportional: false,
            fermata: None,
            expand_repeats: true,
            beat_string: None,
            broken_rhythm_ratio: None,
        }
    }

    /// Build a context from the current state.
    ///
    /// This creates an `Rc<Context>` that can be shared across measures
    /// until the context changes.
    pub(crate) fn build_context(&self) -> Rc<Context<'input>> {
        use crate::types::Context;

        Context::builder()
            .key(self.key.clone())
            .meter(self.meter)
            .maybe_tempo(self.tempo.clone())
            .unit_length(self.unit_length)
            .maybe_beat_pattern(self.beat_pattern)
            .maybe_gchord_pattern(self.gchord_pattern)
            .gchord_enabled(self.gchord_enabled)
            .maybe_chord_instrument(self.chord_instrument)
            .maybe_chord_volume(self.chord_volume)
            .maybe_bass_instrument(self.bass_instrument)
            .maybe_bass_volume(self.bass_volume)
            .maybe_grace_timing(self.grace_timing)
            .maybe_drum_pattern(self.drum_pattern)
            .drum_enabled(self.drum_enabled)
            .maybe_drum_bars(self.drum_bars)
            .maybe_make_chord_channels(self.make_chord_channels)
            .maybe_random_chord_attack(self.random_chord_attack)
            .maybe_chord_attack(self.chord_attack)
            .maybe_portamento(self.portamento)
            .fermata_proportional(self.fermata_proportional)
            .maybe_fermata(self.fermata)
            .expand_repeats(self.expand_repeats)
            .maybe_beat_string(self.beat_string)
            .maybe_broken_rhythm_ratio(self.broken_rhythm_ratio)
            .build()
            .into_rc()
    }

    /// Update the key signature.
    ///
    /// Returns `true` if the context changed (requiring a new `Rc<Context>`).
    pub(crate) fn update_key(&mut self, key: Rc<KeySignature>) -> bool {
        if self.key != key {
            self.key = key;
            true
        } else {
            false
        }
    }

    /// Update the metre.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_meter(&mut self, meter: MeterSymbol) -> bool {
        let new_meter = match meter {
            MeterSymbol::CommonTime => Meter {
                numerator: 4,
                denominator: 4,
            },
            MeterSymbol::CutTime => Meter {
                numerator: 2,
                denominator: 2,
            },
            MeterSymbol::Explicit(m) => m,
        };

        if self.meter != new_meter {
            self.meter = new_meter;
            true
        } else {
            false
        }
    }

    /// Update the tempo.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_tempo(&mut self, tempo: Rc<TempoMarking>) -> bool {
        if self.tempo.as_deref() != Some(&*tempo) {
            self.tempo = Some(tempo);
            true
        } else {
            false
        }
    }

    /// Update the unit note length.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_unit_length(&mut self, unit_length: Duration) -> bool {
        if self.unit_length != unit_length {
            self.unit_length = unit_length;
            true
        } else {
            false
        }
    }

    /// Get the current unit note length.
    pub(crate) fn unit_length(&self) -> &Duration {
        &self.unit_length
    }

    /// Apply an accidental to a pitch class.
    ///
    /// The accidental remains active until the next bar line.
    pub(crate) fn apply_accidental(&mut self, pitch: PitchClass, accidental: Accidental) {
        self.active_accidentals.insert(pitch, accidental);
    }

    /// Get the effective accidental for a pitch class.
    ///
    /// This checks:
    /// 1. Active accidentals within the current measure
    /// 2. The key signature (both explicit and implied accidentals)
    ///
    /// Returns `None` if the pitch is natural.
    pub(crate) fn get_accidental(&self, pitch: PitchClass) -> Option<Accidental> {
        // Check measure-local accidentals first
        if let Some(acc) = self.active_accidentals.get(&pitch) {
            // A natural accidental explicitly cancels any key signature accidental
            if *acc == Accidental::Natural {
                return None;
            }
            return Some(*acc);
        }

        // Then check key signature (explicit and implied accidentals)
        self.key.get_accidental(pitch)
    }

    /// Clear active accidentals (called at bar lines).
    pub(crate) fn clear_accidentals(&mut self) {
        self.active_accidentals.clear();
    }

    /// Update voice attributes from a voice declaration.
    ///
    /// This stores the attributes for later use when processing notes in this voice.
    pub(crate) fn update_voice_attributes(
        &mut self,
        voice_id: VoiceId<'input>,
        attrs: &VoiceAttributes<'input>,
    ) {
        let resolved = ResolvedVoiceAttributes {
            name: attrs.name,
            clef: attrs.clef,
            stem: attrs.stem,
            octave_shift: attrs.octave.unwrap_or(0),
            transpose: attrs.transpose.unwrap_or(0),
            instrument: attrs.instrument,
            midi_channel: None,
            midi_transpose: 0,
            midi_rtranspose: 0,
        };
        self.voice_attributes.insert(voice_id, resolved);
    }

    /// Get resolved attributes for a voice.
    ///
    /// Returns default attributes if the voice hasn't been declared.
    pub(crate) fn get_voice_attributes(&self, voice_id: &VoiceId<'input>) -> ResolvedVoiceAttributes<'input> {
        self.voice_attributes
            .get(voice_id)
            .cloned()
            .unwrap_or_default()
    }

    // Playback parameter update methods

    /// Update the beat pattern.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_beat_pattern(
        &mut self,
        first: Velocity,
        strong: Velocity,
        weak: Velocity,
        very_weak: Velocity,
    ) -> bool {
        update_field!(self.beat_pattern, Some((first, strong, weak, very_weak)))
    }

    /// Update the guitar chord pattern.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_gchord_pattern(&mut self, pattern: &'input str) -> bool {
        update_field!(self.gchord_pattern, Some(pattern))
    }

    /// Enable or disable guitar chord accompaniment.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn set_gchord_enabled(&mut self, enabled: bool) -> bool {
        update_field!(self.gchord_enabled, enabled)
    }

    /// Update the chord accompaniment instrument.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_chord_instrument(&mut self, instrument: Instrument) -> bool {
        update_field!(self.chord_instrument, Some(instrument))
    }

    /// Update the chord accompaniment volume.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_chord_volume(&mut self, volume: Velocity) -> bool {
        update_field!(self.chord_volume, Some(volume))
    }

    /// Update the bass accompaniment instrument.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_bass_instrument(&mut self, instrument: Instrument) -> bool {
        update_field!(self.bass_instrument, Some(instrument))
    }

    /// Update the bass accompaniment volume.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_bass_volume(&mut self, volume: Velocity) -> bool {
        update_field!(self.bass_volume, Some(volume))
    }

    /// Update the grace note timing.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_grace_timing(&mut self, numerator: u8, denominator: u8) -> bool {
        update_field!(self.grace_timing, Some((numerator, denominator)))
    }

    /// Update the drum pattern.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_drum_pattern(&mut self, pattern: &'input str) -> bool {
        update_field!(self.drum_pattern, Some(pattern))
    }

    /// Enable or disable drum accompaniment.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn set_drum_enabled(&mut self, enabled: bool) -> bool {
        update_field!(self.drum_enabled, enabled)
    }

    /// Update the number of bars over which the drum pattern repeats.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_drum_bars(&mut self, bars: u8) -> bool {
        update_field!(self.drum_bars, Some(bars))
    }

    /// Update grace note divider (alternative to grace timing).
    ///
    /// Converts divider to timing as (1, divider).
    /// Returns `true` if the context changed.
    pub(crate) fn update_grace_divider(&mut self, divider: u8) -> bool {
        self.update_grace_timing(1, divider)
    }

    /// Update the number of channels to distribute chord notes across.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_make_chord_channels(&mut self, channels: u8) -> bool {
        update_field!(self.make_chord_channels, Some(channels))
    }

    /// Update the randomization amount for chord note timing.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_random_chord_attack(&mut self, amount: u8) -> bool {
        update_field!(self.random_chord_attack, Some(amount))
    }

    /// Update the delay between chord notes.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_chord_attack(&mut self, delay: u8) -> bool {
        update_field!(self.chord_attack, Some(delay))
    }

    /// Update the portamento time.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_portamento(&mut self, time: u8) -> bool {
        update_field!(self.portamento, Some(time))
    }

    /// Enable or disable proportional fermata length.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn set_fermata_proportional(&mut self, enabled: bool) -> bool {
        update_field!(self.fermata_proportional, enabled)
    }

    /// Update the fermata duration multiplier.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_fermata(&mut self, numerator: u8, denominator: u8) -> bool {
        update_field!(self.fermata, Some((numerator, denominator)))
    }

    /// Enable or disable repeat expansion.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn set_expand_repeats(&mut self, expand: bool) -> bool {
        update_field!(self.expand_repeats, expand)
    }

    /// Update the beat pattern string.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_beat_string(&mut self, pattern: &'input str) -> bool {
        update_field!(self.beat_string, Some(pattern))
    }

    /// Update the broken rhythm ratio.
    ///
    /// Returns `true` if the context changed.
    pub(crate) fn update_broken_rhythm_ratio(&mut self, num: u8, denom: u8) -> bool {
        update_field!(self.broken_rhythm_ratio, Some((num, denom)))
    }

    /// Get the current broken rhythm ratio.
    ///
    /// Returns the custom ratio if set, otherwise None.
    pub(crate) fn broken_rhythm_ratio(&self) -> Option<(u8, u8)> {
        self.broken_rhythm_ratio
    }
}

/// Infer the default unit note length from a metre.
///
/// According to ABC 2.1 standard:
/// - Metres < 0.75 (e.g., 2/4, 3/4) default to 1/16
/// - Metres >= 0.75 (e.g., 4/4, 6/8) default to 1/8
pub(crate) fn infer_unit_length(meter: &Meter) -> Duration {
    let ratio = meter.numerator as f64 / meter.denominator as f64;
    if ratio < 0.75 {
        Duration::new(1, 16)
    } else {
        Duration::new(1, 8)
    }
}
