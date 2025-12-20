//! Test utilities and factory functions.
//!
//! This module provides factory functions to reduce test verbosity by creating
//! common test structures with sensible defaults.

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod factories {
    use std::{collections::HashMap, rc::Rc};

    use crate::types::{
        Accidental, Context, Duration, DynamicDirection, Dynamics, Event, KeySignature,
        Measure, MeasureNumber, Meter, Mode, Note, Octave, Pitch, PitchClass, ReferenceNumber,
        Tune, TuneMetadata, Voice, VoiceId,
    };

    /// Creates a test note with the given pitch class and sensible defaults.
    ///
    /// # Defaults
    /// - Octave: 4 (middle octave)
    /// - Accidental: None
    /// - Duration: 1/8
    /// - Dynamics: default (mezzo-forte)
    /// - Absolute time: 0/1
    /// - No articulations
    /// - No lyrics
    /// - No slurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let c = test_note(PitchClass::C);
    /// let d = test_note(PitchClass::D);
    /// ```
    pub fn test_note(pitch_class: PitchClass) -> Note<'static> {
        Note {
            pitch: Pitch::new(pitch_class, Octave(4), None),
            duration: Duration::new(1, 8),
            dynamics: Dynamics::default(),
            dynamic_direction: DynamicDirection::None,
            absolute_time: Duration::new(0, 1),
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }
    }

    /// Creates a test note with full customization.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let c_sharp = test_note_full(
    ///     PitchClass::C,
    ///     Octave(5),
    ///     Some(Accidental::Sharp),
    ///     Duration::new(1, 4),
    ///     Duration::new(1, 2),
    /// );
    /// ```
    pub fn test_note_full(
        pitch_class: PitchClass,
        octave: Octave,
        accidental: Option<Accidental>,
        duration: Duration,
        absolute_time: Duration,
    ) -> Note<'static> {
        Note {
            pitch: Pitch::new(pitch_class, octave, accidental),
            duration,
            dynamics: Dynamics::default(),
            dynamic_direction: DynamicDirection::None,
            absolute_time,
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }
    }

    /// Creates a test measure with the given number and events.
    ///
    /// # Defaults
    /// - Context: C major, 4/4 time, 1/8 unit length
    /// - Part: None
    /// - Variant: None
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let measure = test_measure(
    ///     1,
    ///     vec![
    ///         test_note(PitchClass::C).into(),
    ///         test_note(PitchClass::D).into(),
    ///     ],
    /// );
    /// ```
    pub fn test_measure(number: u32, events: Vec<Event<'static>>) -> Measure<'static> {
        Measure {
            number: MeasureNumber(number),
            events,
            context: test_context(),
            part: None,
            variant: None,
        }
    }

    /// Creates a test measure with a custom context.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let context = test_context_with_key(PitchClass::G, Mode::Major);
    /// let measure = test_measure_with_context(1, vec![], context);
    /// ```
    pub fn test_measure_with_context(
        number: u32,
        events: Vec<Event<'static>>,
        context: Rc<Context<'static>>,
    ) -> Measure<'static> {
        Measure {
            number: MeasureNumber(number),
            events,
            context,
            part: None,
            variant: None,
        }
    }

    /// Creates a test voice with the given ID and measures.
    ///
    /// # Defaults
    /// - Name: None
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let voice = test_voice("1", vec![
    ///     test_measure(1, vec![test_note(PitchClass::C).into()]),
    /// ]);
    /// ```
    pub fn test_voice(id: &'static str, measures: Vec<Measure<'static>>) -> Voice<'static> {
        Voice {
            id: VoiceId::new(id),
            name: None,
            measures,
        }
    }

    /// Creates a test tune with the given parameters.
    ///
    /// # Defaults
    /// - Composer: None
    /// - Origin: None
    /// - Rhythm: None
    /// - Metadata: default
    /// - Tempo: None
    /// - Unit length: 1/8
    /// - Default voice: "1"
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let tune = test_tune(
    ///     1,
    ///     "Test Tune",
    ///     test_key_signature(PitchClass::G, Mode::Major),
    ///     test_meter(4, 4),
    ///     vec![test_voice("1", vec![])],
    /// );
    /// ```
    pub fn test_tune(
        ref_num: u32,
        title: &'static str,
        key: KeySignature,
        meter: Meter,
        voices: Vec<Voice<'static>>,
    ) -> Tune<'static> {
        let mut voice_map = HashMap::new();
        for voice in voices {
            voice_map.insert(voice.id, voice);
        }

        Tune {
            reference_number: ReferenceNumber(ref_num),
            title,
            composer: None,
            origin: None,
            rhythm: None,
            metadata: TuneMetadata::default(),
            key,
            meter,
            tempo: None,
            unit_length: Duration::new(1, 8),
            voices: voice_map,
            default_voice: VoiceId::new("1"),
        }
    }

    /// Creates a test key signature with the given tonic and mode.
    ///
    /// # Defaults
    /// - Accidental: None
    /// - Explicit accidentals: empty
    /// - Clef: None
    /// - Transpose: None
    /// - Octave shift: None
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let g_major = test_key_signature(PitchClass::G, Mode::Major);
    /// let d_minor = test_key_signature(PitchClass::D, Mode::Minor);
    /// ```
    pub fn test_key_signature(tonic: PitchClass, mode: Mode) -> KeySignature {
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
    }

    /// Creates a test meter with the given numerator and denominator.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let four_four = test_meter(4, 4);
    /// let three_four = test_meter(3, 4);
    /// let six_eight = test_meter(6, 8);
    /// ```
    pub fn test_meter(numerator: u8, denominator: u8) -> Meter {
        Meter {
            numerator,
            denominator,
        }
    }

    /// Creates a default test context with C major, 4/4 time, and 1/8 unit length.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let context = test_context();
    /// ```
    pub fn test_context() -> Rc<Context<'static>> {
        test_context_with_key(PitchClass::C, Mode::Major)
    }

    /// Creates a test context with the given key signature.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let g_major_context = test_context_with_key(PitchClass::G, Mode::Major);
    /// ```
    pub fn test_context_with_key(tonic: PitchClass, mode: Mode) -> Rc<Context<'static>> {
        let key = test_key_signature(tonic, mode);
        Context::builder()
            .key(key)
            .meter(test_meter(4, 4))
            .unit_length(Duration::new(1, 8))
            .expand_repeats(false) // Override default of true for tests
            .build()
            .into_rc()
    }
}
