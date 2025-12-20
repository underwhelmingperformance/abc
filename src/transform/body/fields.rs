//! Inline field and directive processing.

use std::rc::Rc;

use crate::types::ast::{self, InlineField};

use super::BodyTransformer;

impl<'a> BodyTransformer<'a> {
    /// Process an inline field, updating context.
    pub(super) fn process_inline_field(&mut self, field: &InlineField<'a>) {
        let context_changed = match field {
            InlineField::Key(key) => self.context_builder.update_key(Rc::new(key.clone())),
            InlineField::Meter(meter) => self.context_builder.update_meter(*meter),
            InlineField::Tempo(tempo) => self.context_builder.update_tempo(Rc::new(tempo.clone())),
            InlineField::UnitNoteLength(length) => {
                self.context_builder.update_unit_length(*length)
            }
            InlineField::Voice(voice_decl) => {
                // Register voice attributes if present
                if let Some(ref attrs) = voice_decl.attributes {
                    self.context_builder
                        .update_voice_attributes(voice_decl.id, attrs);
                }
                // Switch to the voice specified in the inline field
                self.switch_voice(voice_decl);
                false
            }
        };

        if context_changed {
            self.current_context = self.context_builder.build_context();
        }
    }

    /// Process a directive, updating context or voice attributes.
    pub(super) fn process_directive(&mut self, directive: &ast::Directive<'a>) {
        match directive {
            ast::Directive::Midi(midi_directive) => {
                self.process_midi_directive(midi_directive);
            }
            // Stylesheet, font, and text directives are for rendering only
            // They don't affect the musical transformation
            ast::Directive::Stylesheet(_)
            | ast::Directive::Font(_)
            | ast::Directive::Text(_)
            | ast::Directive::Unknown { .. } => {}
        }
    }

    /// Process a MIDI directive, updating context fields.
    pub(super) fn process_midi_directive(&mut self, directive: &ast::MidiDirective<'a>) {
        use ast::MidiDirective;

        let mut context_changed = false;

        match directive {
            MidiDirective::Channel(channel) => {
                // Update current voice's MIDI channel
                let mut attrs = self.context_builder.get_voice_attributes(&self.current_voice);
                attrs.midi_channel = Some(*channel);
                self.context_builder
                    .voice_attributes
                    .insert(self.current_voice, attrs);
            }
            MidiDirective::Program { channel: _, program } => {
                // Update current voice's instrument
                let mut attrs = self.context_builder.get_voice_attributes(&self.current_voice);
                attrs.instrument = Some(*program);
                self.context_builder
                    .voice_attributes
                    .insert(self.current_voice, attrs);
            }
            MidiDirective::Beat {
                first,
                strong,
                weak,
                very_weak,
            } => {
                context_changed |=
                    self.context_builder
                        .update_beat_pattern(*first, *strong, *weak, *very_weak);
            }
            MidiDirective::Transpose(semitones) => {
                // MIDI transpose adds to the voice transpose attribute
                // It's applied in addition to voice-level transpose during pitch resolution
                let mut attrs = self.context_builder.get_voice_attributes(&self.current_voice);
                attrs.midi_transpose = *semitones;
                self.context_builder
                    .voice_attributes
                    .insert(self.current_voice, attrs);
            }
            MidiDirective::GChord(pattern) => {
                context_changed |= self.context_builder.update_gchord_pattern(pattern);
            }
            MidiDirective::ChordProg(instrument) => {
                context_changed |= self.context_builder.update_chord_instrument(*instrument);
            }
            MidiDirective::BassProg(instrument) => {
                context_changed |= self.context_builder.update_bass_instrument(*instrument);
            }
            MidiDirective::BassVol(velocity) => {
                context_changed |= self.context_builder.update_bass_volume(*velocity);
            }
            MidiDirective::ChordVol(velocity) => {
                context_changed |= self.context_builder.update_chord_volume(*velocity);
            }
            MidiDirective::Grace {
                numerator,
                denominator,
            } => {
                context_changed |= self.context_builder.update_grace_timing(*numerator, *denominator);
            }
            MidiDirective::Drum { pattern } => {
                context_changed |= self.context_builder.update_drum_pattern(pattern);
            }
            MidiDirective::DrumOn => {
                context_changed |= self.context_builder.set_drum_enabled(true);
            }
            MidiDirective::DrumOff => {
                context_changed |= self.context_builder.set_drum_enabled(false);
            }
            MidiDirective::GChordOn => {
                context_changed |= self.context_builder.set_gchord_enabled(true);
            }
            MidiDirective::GChordOff => {
                context_changed |= self.context_builder.set_gchord_enabled(false);
            }
            MidiDirective::DrumBars(bars) => {
                context_changed |= self.context_builder.update_drum_bars(*bars);
            }
            MidiDirective::GraceDivider(divider) => {
                context_changed |= self.context_builder.update_grace_divider(*divider);
            }
            MidiDirective::MakeChordChannels(channels) => {
                context_changed |= self.context_builder.update_make_chord_channels(*channels);
            }
            MidiDirective::RandomChordAttack(amount) => {
                context_changed |= self.context_builder.update_random_chord_attack(*amount);
            }
            MidiDirective::ChordAttack(delay) => {
                context_changed |= self.context_builder.update_chord_attack(*delay);
            }
            MidiDirective::Portamento(time) => {
                context_changed |= self.context_builder.update_portamento(*time);
            }
            MidiDirective::FermataProportional => {
                context_changed |= self.context_builder.set_fermata_proportional(true);
            }
            MidiDirective::Fermata {
                numerator,
                denominator,
            } => {
                context_changed |= self.context_builder.update_fermata(*numerator, *denominator);
            }
            MidiDirective::Expand => {
                context_changed |= self.context_builder.set_expand_repeats(true);
            }
            MidiDirective::NoExpand => {
                context_changed |= self.context_builder.set_expand_repeats(false);
            }
            MidiDirective::Rtranspose(semitones) => {
                // Rtranspose is cumulative - it adds to the existing midi_rtranspose
                let mut attrs = self.context_builder.get_voice_attributes(&self.current_voice);
                attrs.midi_rtranspose = attrs.midi_rtranspose.saturating_add(*semitones);
                self.context_builder
                    .voice_attributes
                    .insert(self.current_voice, attrs);
            }
            MidiDirective::BeatString(pattern) => {
                context_changed |= self.context_builder.update_beat_string(pattern);
            }
            MidiDirective::Ratio(num, denom) => {
                context_changed |= self.context_builder.update_broken_rhythm_ratio(*num, *denom);
            }
            MidiDirective::ChordName { name: _, notes: _ } => {
                // ChordName defines custom chord shapes that would be stored in
                // a document-level chord definition table. For now, we parse it
                // but don't process it further. Full implementation would require
                // adding a chord definition table to the Document or Context.
                // This allows the directive to be parsed without error.
            }
            // Control and PitchBend are event-level, not context-level
            // They should be emitted as marker events at the current time
            MidiDirective::Control {
                channel,
                controller,
                value,
            } => {
                let absolute_time = self.absolute_time();
                self.current_measure_events.push(
                    crate::types::MidiControlEvent {
                        channel: *channel,
                        controller: *controller,
                        value: *value,
                        absolute_time,
                    }
                    .into(),
                );
            }
            MidiDirective::PitchBend { channel, value } => {
                // Use current voice's channel if not specified
                let ch = channel.unwrap_or_else(|| {
                    self.context_builder
                        .get_voice_attributes(&self.current_voice)
                        .midi_channel
                        .unwrap_or_else(|| crate::types::Channel::new(1).unwrap())
                });
                let absolute_time = self.absolute_time();
                self.current_measure_events.push(
                    crate::types::MidiPitchBendEvent {
                        channel: ch,
                        value: *value,
                        absolute_time,
                    }
                    .into(),
                );
            }
        }

        if context_changed {
            self.current_context = self.context_builder.build_context();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::create_test_context;
    use super::super::BodyTransformer;
    use crate::types::{
        ast::{self, BodyElement, Directive, MidiDirective, NoteDuration, NotePitch},
        midi::Velocity,
        DynamicDirection, Dynamics, Duration, Instrument, Measure, MeasureNumber, Note, Octave,
        Pitch, PitchClass, Voice, VoiceId,
    };
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_midi_gchord_pattern() {
        let mut context = create_test_context();
        context.update_gchord_pattern("fzczfzcz");
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::GChord("fzczfzcz"),
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_beat_pattern() {
        let mut context = create_test_context();
        context.update_beat_pattern(
            Velocity::new(127).unwrap(),
            Velocity::new(100).unwrap(),
            Velocity::new(80).unwrap(),
            Velocity::new(60).unwrap(),
        );
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(MidiDirective::Beat {
            first: Velocity::new(127).unwrap(),
            strong: Velocity::new(100).unwrap(),
            weak: Velocity::new(80).unwrap(),
            very_weak: Velocity::new(60).unwrap(),
        })));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_chord_bass_instruments() {
        let mut context = create_test_context();
        context.update_chord_instrument(Instrument::AcousticGuitarSteel);
        context.update_bass_instrument(Instrument::AcousticBass);
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::ChordProg(Instrument::AcousticGuitarSteel),
        )));
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::BassProg(Instrument::AcousticBass),
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_drum_enabled() {
        let mut context = create_test_context();
        context.set_drum_enabled(true);
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::DrumOn,
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_expand_repeats() {
        let mut context = create_test_context();
        context.set_expand_repeats(true);
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Expand,
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_channel_assignment() {
        use crate::types::Channel;

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Assign MIDI channel 5 to current voice
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Channel(Channel::new(5).unwrap()),
        )));

        // Verify channel is stored in voice attributes
        let attrs = transformer
            .context_builder
            .get_voice_attributes(&transformer.current_voice);
        assert_eq!(attrs.midi_channel, Some(Channel::new(5).unwrap()));
    }

    #[test]
    fn test_midi_transpose_directive() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Transpose up 2 semitones via MIDI directive
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Transpose(2),
        )));

        // C should become D (transposed up 2 semitones)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::D,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_transpose_adds_to_voice_transpose() {
        use crate::types::ast::InlineField;

        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Voice with transpose=1
        transformer.process_element(&BodyElement::InlineField(InlineField::Voice(
            ast::VoiceDeclaration {
                id: VoiceId::new("transposed"),
                attributes: Some(ast::VoiceAttributes {
                    name: None,
                    subname: None,
                    clef: None,
                    stem: None,
                    octave: None,
                    transpose: Some(1), // Voice transpose of +1
                    instrument: None,
                    key: None,
                }),
            },
        )));

        // Add MIDI transpose of +2
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Transpose(2),
        )));

        // C should become D# (transposed up 1+2=3 semitones)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("transposed")).unwrap();

        let expected = Voice {
            id: VoiceId::new("transposed"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::D,
                        accidental: Some(crate::types::Accidental::Sharp),
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_control_event() {
        use crate::types::{Channel, Event, MarkerEvent};

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Emit control change (controller 7, volume = 100) on channel 1
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Control {
                channel: Channel::new(1).unwrap(),
                controller: 7,
                value: 100,
            },
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        // Check that the control event was emitted at time 0
        use crate::types::MidiControlEvent;

        let expected_control = Event::Marker(MarkerEvent::MidiControl(MidiControlEvent {
            channel: Channel::new(1).unwrap(),
            controller: 7,
            value: 100,
            absolute_time: Duration::new(0, 1),
        }));

        let expected_note = Note {
            pitch: Pitch {
                pitch_class: PitchClass::C,
                accidental: None,
                octave: Octave(4),
            },
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
        }.into();

        assert_eq!(voice.measures[0].events, vec![expected_control, expected_note]);
    }

    #[test]
    fn test_midi_pitchbend_event() {
        use crate::types::{Channel, Event, MarkerEvent};

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Emit pitch bend (value = 1000) on channel 2
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::PitchBend {
                channel: Some(Channel::new(2).unwrap()),
                value: 1000,
            },
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        // Check that the pitch bend event was emitted at time 0
        use crate::types::MidiPitchBendEvent;

        let expected_pitch_bend = Event::Marker(MarkerEvent::MidiPitchBend(MidiPitchBendEvent {
            channel: Channel::new(2).unwrap(),
            value: 1000,
            absolute_time: Duration::new(0, 1),
        }));

        let expected_note = Note {
            pitch: Pitch {
                pitch_class: PitchClass::C,
                accidental: None,
                octave: Octave(4),
            },
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
        }.into();

        assert_eq!(voice.measures[0].events, vec![expected_pitch_bend, expected_note]);
    }

    #[test]
    fn test_midi_pitchbend_defaults_to_voice_channel() {
        use crate::types::{Channel, Event, MarkerEvent};

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Set voice channel to 5
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Channel(Channel::new(5).unwrap()),
        )));

        // Emit pitch bend without specifying channel (should use voice channel)
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::PitchBend {
                channel: None,
                value: 500,
            },
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        use crate::types::MidiPitchBendEvent;

        let expected_pitch_bend = Event::Marker(MarkerEvent::MidiPitchBend(MidiPitchBendEvent {
            channel: Channel::new(5).unwrap(),
            value: 500,
            absolute_time: Duration::new(0, 1),
        }));

        let expected_note = Note {
            pitch: Pitch {
                pitch_class: PitchClass::C,
                accidental: None,
                octave: Octave(4),
            },
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
        }.into();

        assert_eq!(voice.measures[0].events, vec![expected_pitch_bend, expected_note]);
    }

    #[test]
    fn test_midi_rtranspose_cumulative() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Apply first rtranspose of +2
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Rtranspose(2),
        )));

        // Apply second rtranspose of +3 (should cumulate to +5)
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Rtranspose(3),
        )));

        // C should become F (transposed up 5 semitones)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::F,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_rtranspose_with_transpose() {
        use crate::types::ast::InlineField;

        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Voice with transpose=1
        transformer.process_element(&BodyElement::InlineField(InlineField::Voice(
            ast::VoiceDeclaration {
                id: VoiceId::new("transposed"),
                attributes: Some(ast::VoiceAttributes {
                    name: None,
                    subname: None,
                    clef: None,
                    stem: None,
                    octave: None,
                    transpose: Some(1),
                    instrument: None,
                    key: None,
                }),
            },
        )));

        // Add MIDI transpose of +2
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Transpose(2),
        )));

        // Add rtranspose of +1 (should cumulate to voice:1 + midi:2 + rtranspose:1 = 4)
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Rtranspose(1),
        )));

        // C should become E (transposed up 4 semitones)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("transposed")).unwrap();

        let expected = Voice {
            id: VoiceId::new("transposed"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::E,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_beatstring() {
        let mut context = create_test_context();
        context.update_beat_string("fmfp");
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::BeatString("fmfp"),
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(4),
                    },
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
                .into()],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_ratio_override() {
        use crate::types::ast::BrokenRhythm;

        let mut context = create_test_context();
        context.update_broken_rhythm_ratio(5, 3);
        let context_rc = context.build_context();

        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Set custom broken rhythm ratio to 5:3
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::Ratio(5, 3),
        )));

        // C with > (dot first) - C should get 5/3×, D should get 1/3×
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: Some(BrokenRhythm::DotFirst),
        }));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::D,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![
                    Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::C,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(5, 24), // 1/8 * 5/3
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
                    .into(),
                    Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::D,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 24), // 1/8 * 1/3
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(5, 24),
                        articulations: Vec::new(),
                        lyric: None,
                        slur_starts: 0,
                        slur_ends: 0,
                        dotted_slur_starts: 0,
                        dotted_slur_ends: 0,
                    }
                    .into(),
                ],
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_midi_chordname_parses_without_error() {
        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // ChordName should parse and not cause errors, even though
        // it's not fully implemented (would need chord definition table)
        transformer.process_element(&BodyElement::Directive(Directive::Midi(
            MidiDirective::ChordName {
                name: "maj7",
                notes: vec![0, 4, 7, 11],
            },
        )));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        // Should complete without error
        let voices = transformer.into_voices();
        assert!(voices.contains_key(&VoiceId::new("1")));
    }
}
