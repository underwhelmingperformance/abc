//! Pitch resolution for notes.

use crate::types::{Octave, Pitch, ast::NotePitch};

use super::BodyTransformer;

impl<'a> BodyTransformer<'a> {
    /// Resolve a NotePitch to a Pitch, considering context.
    ///
    /// This handles:
    /// - Accidentals from the note itself
    /// - Active accidentals within the measure
    /// - Key signature accidentals
    /// - Octave calculation (ABC octave 0 = MIDI octave 4)
    pub(super) fn resolve_pitch(&mut self, pitch: &NotePitch) -> Pitch {
        // If the note has an explicit accidental, record it for the measure
        if let Some(acc) = pitch.accidental {
            self.context_builder.apply_accidental(pitch.base, acc);
        }

        // Get the effective accidental (from note, measure, or key signature)
        let accidental = pitch
            .accidental
            .or_else(|| self.context_builder.get_accidental(pitch.base));

        // Get voice-specific attributes
        let voice_attrs = self
            .context_builder
            .get_voice_attributes(&self.current_voice);

        // ABC octave 0 corresponds to the "middle" octave (MIDI octave 4)
        // Apply voice-specific octave shift
        let octave = Octave(4 + pitch.octave.0 + voice_attrs.octave_shift);

        let resolved_pitch = Pitch {
            pitch_class: pitch.base,
            accidental,
            octave,
        };

        // Apply voice-specific transposition (in semitones)
        // This includes:
        // - voice transpose attribute (from V: field)
        // - MIDI transpose directive (%%MIDI transpose)
        // - MIDI rtranspose directive (%%MIDI rtranspose, cumulative)
        let total_transpose =
            voice_attrs.transpose + voice_attrs.midi_transpose + voice_attrs.midi_rtranspose;
        if total_transpose != 0 {
            resolved_pitch
                .transpose_semitones(total_transpose)
                .unwrap_or(resolved_pitch)
        } else {
            resolved_pitch
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use super::super::test_utils::create_test_context;
    use super::super::BodyTransformer;
    use crate::types::{
        Accidental, Duration, DynamicDirection, Dynamics, Measure, MeasureNumber, Note, Octave,
        Pitch, PitchClass, Voice, VoiceId,
        ast::{self, BodyElement, InlineField, NoteDuration, NotePitch},
    };

    #[test]
    fn test_accidental_persists_within_measure() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // First C with sharp
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: Some(Accidental::Sharp),
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        // Second C without accidental (should inherit sharp)
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
                events: vec![
                    Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::C,
                            accidental: Some(Accidental::Sharp),
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
                    Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::C,
                            accidental: Some(Accidental::Sharp),
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
                context: context_rc,
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_accidental_cleared_at_barline() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // C with sharp
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: Some(Accidental::Sharp),
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        // Barline
        transformer.process_element(&BodyElement::BarLine(ast::BarLine::Single));

        // C without accidental (should be natural now)
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
            measures: vec![
                Measure {
                    number: MeasureNumber(1),
                    events: vec![Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::C,
                            accidental: Some(Accidental::Sharp),
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
                    context: context_rc.clone(),
                    part: None,
                    variant: None,
                },
                Measure {
                    number: MeasureNumber(2),
                    events: vec![Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::C,
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
                    .into()],
                    context: context_rc,
                    part: None,
                    variant: None,
                },
            ],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_voice_attribute_octave_shift() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Voice declaration with octave=-1 (shift down one octave)
        transformer.process_element(&BodyElement::InlineField(InlineField::Voice(
            ast::VoiceDeclaration {
                id: VoiceId::new("bass"),
                attributes: Some(ast::VoiceAttributes {
                    name: None,
                    subname: None,
                    clef: None,
                    stem: None,
                    octave: Some(-1),
                    transpose: None,
                    instrument: None,
                    key: None,
                }),
            },
        )));

        // C in octave 0 should become C in octave 3 (4 + 0 - 1 = 3)
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
        let voice = voices.get(&VoiceId::new("bass")).unwrap();

        let expected = Voice {
            id: VoiceId::new("bass"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::C,
                        accidental: None,
                        octave: Octave(3), // Shifted down by 1 from octave 4
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
    fn test_voice_attribute_transpose() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Voice declaration with transpose=2 (shift up a whole step)
        transformer.process_element(&BodyElement::InlineField(InlineField::Voice(
            ast::VoiceDeclaration {
                id: VoiceId::new("clarinet"),
                attributes: Some(ast::VoiceAttributes {
                    name: None,
                    subname: None,
                    clef: None,
                    stem: None,
                    octave: None,
                    transpose: Some(2), // Up a whole step (2 semitones)
                    instrument: None,
                    key: None,
                }),
            },
        )));

        // C should become D when transposed up 2 semitones
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
        let voice = voices.get(&VoiceId::new("clarinet")).unwrap();

        let expected = Voice {
            id: VoiceId::new("clarinet"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Note {
                    pitch: Pitch {
                        pitch_class: PitchClass::D,
                        accidental: None,
                        octave: Octave(4), // Same octave, just transposed pitch class
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
}
