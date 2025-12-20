//! Voice switching and measure management.

use super::{BodyTransformer, LastNotePosition};
use crate::types::{Duration, Measure, MeasureNumber, ast::VoiceDeclaration};

impl<'a> BodyTransformer<'a> {
    /// Switch to a different voice.
    ///
    /// This closes the current measure and switches to the specified voice.
    /// If the voice doesn't exist yet, it is created. If the declaration
    /// includes attributes (like name), they are stored.
    pub(super) fn switch_voice(&mut self, decl: &VoiceDeclaration<'a>) {
        let voice_id = decl.id;

        // Close the current measure if it has events
        if !self.current_measure_events.is_empty() {
            self.close_measure();
        }

        // Ensure the voice exists
        self.voices.entry(voice_id).or_default();

        // Initialize per-voice state if this is a new voice
        self.measure_numbers.entry(voice_id).or_insert(1);
        self.absolute_times
            .entry(voice_id)
            .or_insert_with(|| Duration::new(0, 1));

        // Store voice name if provided in attributes
        if let Some(ref attrs) = decl.attributes
            && attrs.name.is_some()
        {
            self.voice_names.insert(voice_id, attrs.name);
        }

        // Switch to the new voice
        self.current_voice = voice_id;
    }

    /// Close the current measure and start a new one.
    pub(super) fn close_measure(&mut self) {
        if !self.current_measure_events.is_empty() {
            // Update last_note_position to point to the committed measure
            // This allows ties to work across barlines
            if let Some(LastNotePosition::CurrentMeasure(event_idx)) = self.last_note_position {
                let measure_idx = self
                    .voices
                    .get(&self.current_voice)
                    .map(|v| v.len())
                    .unwrap_or(0);
                self.last_note_position =
                    Some(LastNotePosition::CommittedMeasure(measure_idx, event_idx));
            }

            let measure = Measure {
                number: MeasureNumber(self.measure_number()),
                events: std::mem::take(&mut self.current_measure_events),
                context: self.current_context.clone(),
                part: self.current_part,
                variant: self.current_variant.clone(),
            };

            self.voices
                .get_mut(&self.current_voice)
                .unwrap()
                .push(measure);

            self.increment_measure_number();
        }

        // Clear accidentals at barline
        self.context_builder.clear_accidentals();
        // Clear pending lyric notes - lyrics apply within a music line
        self.pending_lyric_notes.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::{super::test_utils::create_test_context, BodyTransformer};
    use crate::types::{
        Duration, DynamicDirection, Dynamics, Measure, MeasureNumber, Note, Octave, Pitch,
        PitchClass, Voice, VoiceId,
        ast::{self, BodyElement, InlineField, NoteDuration, NotePitch},
    };

    #[test]
    fn test_measure_numbers() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

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

        transformer.process_element(&BodyElement::BarLine(ast::BarLine::Single));

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

        transformer.process_element(&BodyElement::BarLine(ast::BarLine::Single));

        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::E,
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
            measures: vec![
                Measure {
                    number: MeasureNumber(1),
                    events: vec![
                        Note {
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
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: None,
                },
                Measure {
                    number: MeasureNumber(2),
                    events: vec![
                        Note {
                            pitch: Pitch {
                                pitch_class: PitchClass::D,
                                accidental: None,
                                octave: Octave(4),
                            },
                            duration: Duration::new(1, 8),
                            dynamics: Dynamics::default(),
                            dynamic_direction: DynamicDirection::None,
                            absolute_time: Duration::new(1, 8),
                            articulations: Vec::new(),
                            lyric: None,
                            slur_starts: 0,
                            slur_ends: 0,
                            dotted_slur_starts: 0,
                            dotted_slur_ends: 0,
                        }
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: None,
                },
                Measure {
                    number: MeasureNumber(3),
                    events: vec![
                        Note {
                            pitch: Pitch {
                                pitch_class: PitchClass::E,
                                accidental: None,
                                octave: Octave(4),
                            },
                            duration: Duration::new(1, 8),
                            dynamics: Dynamics::default(),
                            dynamic_direction: DynamicDirection::None,
                            absolute_time: Duration::new(1, 4),
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
                },
            ],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_voice_switching() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Note in voice 1
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

        // Switch to voice 2
        transformer.process_element(&BodyElement::VoiceSwitch(ast::VoiceDeclaration {
            id: VoiceId::new("2"),
            attributes: None,
        }));

        // Note in voice 2
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

        let mut expected = HashMap::new();
        expected.insert(
            VoiceId::new("1"),
            Voice {
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
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: None,
                }],
            },
        );
        expected.insert(
            VoiceId::new("2"),
            Voice {
                id: VoiceId::new("2"),
                name: None,
                measures: vec![Measure {
                    number: MeasureNumber(1),
                    events: vec![
                        Note {
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
                        .into(),
                    ],
                    context: context_rc,
                    part: None,
                    variant: None,
                }],
            },
        );

        assert_eq!(voices, expected);
    }

    #[test]
    fn test_inline_voice_switching() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Note in voice 1
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

        // Inline voice declaration [V:bass]
        transformer.process_element(&BodyElement::InlineField(InlineField::Voice(
            ast::VoiceDeclaration {
                id: VoiceId::new("bass"),
                attributes: None,
            },
        )));

        // Note in bass voice
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::E,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        let voices = transformer.into_voices();

        let mut expected = HashMap::new();
        expected.insert(
            VoiceId::new("1"),
            Voice {
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
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: None,
                }],
            },
        );
        expected.insert(
            VoiceId::new("bass"),
            Voice {
                id: VoiceId::new("bass"),
                name: None,
                measures: vec![Measure {
                    number: MeasureNumber(1),
                    events: vec![
                        Note {
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
                        .into(),
                    ],
                    context: context_rc,
                    part: None,
                    variant: None,
                }],
            },
        );

        assert_eq!(voices, expected);
    }

    #[test]
    fn test_variant_endings() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Simulate: C |1 D :|2 E |
        // - C is played both times (no variant)
        // - D is only played first time (variant 1)
        // - E is only played second time (variant 2)

        // C (no variant)
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

        // |1 - First variant ending
        transformer.process_element(&BodyElement::VariantEnding(ast::VariantEnding {
            bar: ast::BarLine::Single,
            variants: vec![1],
        }));

        // D (first time only)
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

        // :|2 - Repeat end + second variant ending
        transformer.process_element(&BodyElement::BarLine(ast::BarLine::RepeatEnd));
        transformer.process_element(&BodyElement::VariantEnding(ast::VariantEnding {
            bar: ast::BarLine::Single,
            variants: vec![2],
        }));

        // E (second time only)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::E,
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
            measures: vec![
                Measure {
                    number: MeasureNumber(1),
                    events: vec![
                        Note {
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
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: None, // Played both times
                },
                Measure {
                    number: MeasureNumber(2),
                    events: vec![
                        Note {
                            pitch: Pitch {
                                pitch_class: PitchClass::D,
                                accidental: None,
                                octave: Octave(4),
                            },
                            duration: Duration::new(1, 8),
                            dynamics: Dynamics::default(),
                            dynamic_direction: DynamicDirection::None,
                            absolute_time: Duration::new(1, 8),
                            articulations: Vec::new(),
                            lyric: None,
                            slur_starts: 0,
                            slur_ends: 0,
                            dotted_slur_starts: 0,
                            dotted_slur_ends: 0,
                        }
                        .into(),
                    ],
                    context: context_rc.clone(),
                    part: None,
                    variant: Some(vec![1]), // First time only
                },
                Measure {
                    number: MeasureNumber(3),
                    events: vec![
                        Note {
                            pitch: Pitch {
                                pitch_class: PitchClass::E,
                                accidental: None,
                                octave: Octave(4),
                            },
                            duration: Duration::new(1, 8),
                            dynamics: Dynamics::default(),
                            dynamic_direction: DynamicDirection::None,
                            absolute_time: Duration::new(1, 4),
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
                    variant: Some(vec![2]), // Second time only
                },
            ],
        };

        assert_eq!(voice, &expected);
    }
}
