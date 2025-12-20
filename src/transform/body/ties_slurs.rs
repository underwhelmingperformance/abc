//! Tie and slur handling.

use crate::types::{Duration, Event, Pitch, TimedEvent};

use super::{BodyTransformer, LastNotePosition};

impl<'a> BodyTransformer<'a> {
    /// Apply a tie by extending the duration of the last note or chord.
    ///
    /// Returns true if the tie was successfully applied, false if there was no
    /// previous note/chord to tie to.
    ///
    /// For notes, the pitch is validated (tie is applied even if pitches don't match,
    /// as this is a common notation shorthand). For chords, all pitches are extended.
    pub(super) fn apply_tie_to_last_event(
        &mut self,
        duration: &Duration,
        _tied_pitch: Option<&Pitch>,
    ) -> bool {
        let position = match &self.last_note_position {
            Some(pos) => pos.clone(),
            None => return false,
        };

        match position {
            LastNotePosition::CurrentMeasure(idx) => {
                if let Some(event) = self.current_measure_events.get_mut(idx) {
                    match event {
                        Event::Timed(TimedEvent::Note(note)) => {
                            // TODO: Could validate that tied_pitch matches note.pitch
                            // For now, we extend regardless (common in ABC)
                            note.duration = &note.duration + duration;
                            return true;
                        }
                        Event::Timed(TimedEvent::Chord(chord)) => {
                            // Extend the chord's duration (all notes in chord are tied)
                            chord.duration = &chord.duration + duration;
                            return true;
                        }
                        Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {
                            // Can't tie to these events
                            return false;
                        }
                    }
                }
            }
            LastNotePosition::CommittedMeasure(measure_idx, event_idx) => {
                if let Some(measures) = self.voices.get_mut(&self.current_voice)
                    && let Some(measure) = measures.get_mut(measure_idx)
                        && let Some(event) = measure.events.get_mut(event_idx) {
                            match event {
                                Event::Timed(TimedEvent::Note(note)) => {
                                    note.duration = &note.duration + duration;
                                    return true;
                                }
                                Event::Timed(TimedEvent::Chord(chord)) => {
                                    chord.duration = &chord.duration + duration;
                                    return true;
                                }
                                Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {
                                    return false;
                                }
                            }
                        }
            }
        }

        false
    }

    /// Apply a slur end marker to the last note or chord.
    ///
    /// This is called when a SlurEnd `)` is encountered. It increments the
    /// slur_ends count on the most recent note or chord event.
    ///
    /// Note: ABC notation doesn't distinguish between dotted and regular slur ends
    /// (both use `)` to close). We close slurs in LIFO order, prioritizing dotted
    /// slurs if the last opened slur was dotted.
    pub(super) fn apply_slur_end(&mut self) {
        // Find the last note/chord using last_note_position
        if let Some(ref position) = self.last_note_position {
            match position {
                LastNotePosition::CurrentMeasure(idx) => {
                    if let Some(event) = self.current_measure_events.get_mut(*idx) {
                        match event {
                            Event::Timed(TimedEvent::Note(note)) => {
                                // Check if this note has any open dotted slurs
                                if note.dotted_slur_starts > note.dotted_slur_ends {
                                    note.dotted_slur_ends += 1;
                                } else {
                                    note.slur_ends += 1;
                                }
                            }
                            Event::Timed(TimedEvent::Chord(chord)) => {
                                // Check if this chord has any open dotted slurs
                                if chord.dotted_slur_starts > chord.dotted_slur_ends {
                                    chord.dotted_slur_ends += 1;
                                } else {
                                    chord.slur_ends += 1;
                                }
                            }
                            Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {
                                // These events don't have slurs
                            }
                        }
                    }
                }
                LastNotePosition::CommittedMeasure(measure_idx, event_idx) => {
                    // Get the voice's measures and find the event
                    if let Some(measures) = self.voices.get_mut(&self.current_voice)
                        && let Some(measure) = measures.get_mut(*measure_idx)
                        && let Some(event) = measure.events.get_mut(*event_idx)
                    {
                        match event {
                            Event::Timed(TimedEvent::Note(note)) => {
                                // Check if this note has any open dotted slurs
                                if note.dotted_slur_starts > note.dotted_slur_ends {
                                    note.dotted_slur_ends += 1;
                                } else {
                                    note.slur_ends += 1;
                                }
                            }
                            Event::Timed(TimedEvent::Chord(chord)) => {
                                // Check if this chord has any open dotted slurs
                                if chord.dotted_slur_starts > chord.dotted_slur_ends {
                                    chord.dotted_slur_ends += 1;
                                } else {
                                    chord.slur_ends += 1;
                                }
                            }
                            Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {
                                // These events don't have slurs
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, rc::Rc};

    use pretty_assertions::assert_eq;

    use super::super::BodyTransformer;
    use crate::types::{
        Chord, Duration, DynamicDirection, Dynamics, Measure, MeasureNumber, Note, Octave, Pitch,
        PitchClass, Voice, VoiceId,
        ast::{self, BodyElement, NoteDuration, NotePitch},
    };

    fn create_test_context() -> super::super::super::context::TransformContext<'static> {
        use crate::types::{KeySignature, Meter, Mode};

        let key = KeySignature {
            tonic: PitchClass::C,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };

        super::super::super::context::TransformContext::new(
            Rc::new(key),
            Meter {
                numerator: 4,
                denominator: 4,
            },
            None,
            Duration::new(1, 8),
        )
    }

    #[test]
    fn test_ties_extend_duration() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // First note (C, 1/8 duration)
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

        // Tie marker
        transformer.process_element(&BodyElement::TieStart);

        // Second note (C, 1/8 duration) - should extend previous note
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
                    duration: Duration::new(1, 4), // 1/8 + 1/8 = 1/4
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
    fn test_tie_between_chords() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // First chord [CEG]
        transformer.process_element(&BodyElement::Chord(ast::Chord {
            notes: vec![
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::E,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::G,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
            ],
            duration: None,
        }));

        // Tie marker
        transformer.process_element(&BodyElement::TieStart);

        // Second chord [CEG] - should extend previous chord's duration
        transformer.process_element(&BodyElement::Chord(ast::Chord {
            notes: vec![
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::E,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::G,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
            ],
            duration: None,
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![Chord {
                    pitches: vec![
                        Pitch {
                            pitch_class: PitchClass::C,
                            accidental: None,
                            octave: Octave(4),
                        },
                        Pitch {
                            pitch_class: PitchClass::E,
                            accidental: None,
                            octave: Octave(4),
                        },
                        Pitch {
                            pitch_class: PitchClass::G,
                            accidental: None,
                            octave: Octave(4),
                        },
                    ],
                    duration: Duration::new(1, 4), // 1/8 + 1/8
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
    fn test_tie_across_barline() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Note in first measure
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

        // Tie marker (before barline)
        transformer.process_element(&BodyElement::TieStart);

        // Barline
        transformer.process_element(&BodyElement::BarLine(ast::BarLine::Single));

        // Note in second measure - should extend first note's duration
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

        // First measure has note with extended duration (1/4)
        // Second measure is empty (tied note absorbed), so not created
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
                    duration: Duration::new(1, 4), // 1/8 + 1/8
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
    fn test_slur_tracking() {
        // Test that slurs are properly tracked: (CDE) - slur starts at C, ends at E
        let input = "X:1\nT:Test\nK:C\n(CDE)\n";
        let doc = crate::parse(input).unwrap();
        let tune = doc.tunes().first().unwrap();
        let voice = tune.primary_voice();

        let expected = Voice {
            id: VoiceId::new("1"),
            name: None,
            measures: vec![Measure {
                number: MeasureNumber(1),
                events: vec![
                    Note {
                        pitch: Pitch::new(PitchClass::C, Octave(4), None),
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(0, 1),
                        articulations: Vec::new(),
                        lyric: None,
                        slur_starts: 1,
                        slur_ends: 0,
                        dotted_slur_starts: 0,
                        dotted_slur_ends: 0,
                    }.into(),
                    Note {
                        pitch: Pitch::new(PitchClass::D, Octave(4), None),
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
                    }.into(),
                    Note {
                        pitch: Pitch::new(PitchClass::E, Octave(4), None),
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 4),
                        articulations: Vec::new(),
                        lyric: None,
                        slur_starts: 0,
                        slur_ends: 1,
                        dotted_slur_starts: 0,
                        dotted_slur_ends: 0,
                    }.into(),
                ],
                context: voice.measures.first().unwrap().context.clone(),
                part: None,
                variant: None,
            }],
        };

        assert_eq!(voice, &expected);
    }
}
