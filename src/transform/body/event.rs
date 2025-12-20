//! Note, chord, and rest event processing.

use crate::types::{
    ast::{self, NoteDuration},
    Chord, Duration, Event, Note, Pitch, Rest,
};

use super::{BodyTransformer, LastNotePosition};

impl<'a> BodyTransformer<'a> {
    /// Process a note, resolving pitch and duration.
    pub(super) fn process_note(&mut self, note: &ast::Note<'a>) -> Option<Event<'a>> {
        // Emit any pending grace notes before this main note
        if let Some(grace_notes) = self.pending_grace_notes.take() {
            self.emit_grace_notes(&grace_notes);
        }

        // Combine pending decorations with note decorations
        let mut all_decorations = std::mem::take(&mut self.pending_decorations);
        all_decorations.extend(note.decorations.iter().cloned());

        // Extract dynamics from decorations (updates current_dynamics if found)
        self.apply_dynamics_from_decorations(&all_decorations);

        // Extract articulations from decorations
        let articulations = Self::extract_articulations(&all_decorations);

        let pitch = self.resolve_pitch(&note.pitch);
        let mut duration = self.resolve_duration(note.duration);

        // Apply pending broken rhythm from previous note
        if let Some(prev_broken_rhythm) = self.pending_broken_rhythm.take() {
            duration = self.apply_broken_rhythm_to_second(duration, prev_broken_rhythm);
        }

        // Apply this note's broken rhythm to itself and store for next note
        if let Some(broken_rhythm) = note.broken_rhythm {
            duration = self.apply_broken_rhythm_to_first(duration, broken_rhythm);
            self.pending_broken_rhythm = Some(broken_rhythm);
        }

        // Handle ties: if a tie is pending, extend the previous note/chord's duration
        if self.pending_tie {
            self.pending_tie = false;

            if self.apply_tie_to_last_event(&duration, Some(&pitch)) {
                // Still advance absolute time
                self.advance_time(&duration);
                // Don't create a new note event
                return None;
            }
            // If tie couldn't be applied (no previous note), fall through to create the note
        }

        let absolute_time = self.absolute_time();
        self.advance_time(&duration);

        // Track this note's position for potential ties
        self.last_note_position = Some(LastNotePosition::CurrentMeasure(
            self.current_measure_events.len(),
        ));

        // Consume pending slur starts
        let slur_starts = self.pending_slur_starts;
        self.pending_slur_starts = 0;

        // Consume pending dotted slur starts
        let dotted_slur_starts = self.pending_dotted_slur_starts;
        self.pending_dotted_slur_starts = 0;

        Some(
            Note {
                pitch,
                duration,
                dynamics: self.current_dynamics,
                dynamic_direction: self.current_dynamic_direction,
                absolute_time,
                articulations,
                lyric: None,
                slur_starts,
                slur_ends: 0,
                dotted_slur_starts,
                dotted_slur_ends: 0,
            }
            .into(),
        )
    }

    /// Emit grace notes as events.
    ///
    /// Grace notes are given very short durations (1/32 for regular grace notes,
    /// 1/64 for acciaccaturas) and are emitted as separate note events that
    /// advance the absolute time.
    pub(super) fn emit_grace_notes(&mut self, grace_notes: &ast::GraceNotes<'a>) {
        // Determine grace note duration based on whether it's an acciaccatura
        let grace_duration = if grace_notes.acciaccatura {
            Duration::new(1, 64) // Very quick for acciaccatura
        } else {
            Duration::new(1, 32) // Quick for regular grace notes
        };

        for grace_note in &grace_notes.notes {
            let pitch = self.resolve_pitch(&grace_note.pitch);
            let absolute_time = self.absolute_time();

            self.current_measure_events.push(
                Note {
                pitch,
                duration: grace_duration,
                dynamics: self.current_dynamics,
                dynamic_direction: self.current_dynamic_direction,
                absolute_time,
                articulations: Vec::new(), // Grace notes don't typically have articulations
                lyric: None,
                slur_starts: 0, // Grace notes don't start/end slurs
                slur_ends: 0,
                dotted_slur_starts: 0,
                dotted_slur_ends: 0,
            }
            .into(),
            );

            // Advance absolute time
            self.advance_time(&grace_duration);
        }
    }

    /// Process a rest, resolving duration.
    pub(super) fn process_rest(&mut self, rest: &ast::Rest) -> Option<Event<'a>> {
        // Clear any pending decorations (rests don't have articulations)
        self.pending_decorations.clear();

        // Extract the duration from the rest variant
        let note_duration = match rest {
            ast::Rest::Visible(dur) | ast::Rest::Invisible(dur) => *dur,
            ast::Rest::MultiMeasure { measures, .. } => {
                // Multi-measure rests span multiple measures; for now we treat them
                // as a rest with the specified number of whole measures
                NoteDuration::Explicit {
                    numerator: *measures,
                    denominator: 1,
                }
            }
        };

        let duration = self.resolve_duration(note_duration);
        let absolute_time = self.absolute_time();
        self.advance_time(&duration);

        Some(
            Rest {
                duration,
                absolute_time,
            }
            .into(),
        )
    }

    /// Process a chord, resolving pitches and duration.
    pub(super) fn process_chord(&mut self, chord: &ast::Chord<'a>) -> Option<Event<'a>> {
        // Combine pending decorations with chord note decorations
        let all_decorations = std::mem::take(&mut self.pending_decorations);

        // Extract dynamics and articulations from pending + note decorations
        let mut articulations = Vec::new();

        // First apply pending decorations
        self.apply_dynamics_from_decorations(&all_decorations);
        articulations.extend(Self::extract_articulations(&all_decorations));

        // Then apply each note's decorations
        for note in &chord.notes {
            self.apply_dynamics_from_decorations(&note.decorations);
            articulations.extend(Self::extract_articulations(&note.decorations));
        }

        let pitches: Vec<Pitch> = chord
            .notes
            .iter()
            .map(|note| self.resolve_pitch(&note.pitch))
            .collect();

        // Use the chord's overall duration if specified, otherwise use the first note's duration
        let note_duration = chord.duration.unwrap_or_else(|| {
            chord
                .notes
                .first()
                .map(|n| n.duration)
                .unwrap_or(NoteDuration::Default)
        });

        let duration = self.resolve_duration(note_duration);

        // Handle ties: if a tie is pending, extend the previous note/chord's duration
        if self.pending_tie {
            self.pending_tie = false;

            if self.apply_tie_to_last_event(&duration, None) {
                // Still advance absolute time
                self.advance_time(&duration);
                // Don't create a new chord event
                return None;
            }
            // If tie couldn't be applied, fall through to create the chord
        }

        let absolute_time = self.absolute_time();
        self.advance_time(&duration);

        // Track this chord's position for potential ties
        self.last_note_position = Some(LastNotePosition::CurrentMeasure(
            self.current_measure_events.len(),
        ));

        // Consume pending slur starts
        let slur_starts = self.pending_slur_starts;
        self.pending_slur_starts = 0;

        // Consume pending dotted slur starts
        let dotted_slur_starts = self.pending_dotted_slur_starts;
        self.pending_dotted_slur_starts = 0;

        Some(
            Chord {
                pitches,
                duration,
                dynamics: self.current_dynamics,
                dynamic_direction: self.current_dynamic_direction,
                absolute_time,
                articulations,
                lyric: None,
                slur_starts,
                slur_ends: 0,
                dotted_slur_starts,
                dotted_slur_ends: 0,
            }
            .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::super::{test_utils::create_test_context, BodyTransformer};
    use crate::types::{
        ast::{self, BodyElement, Decoration, NoteDuration, NotePitch},
        Articulation, DynamicDirection, Dynamics, Duration, Measure, MeasureNumber, Note, Octave,
        Pitch, PitchClass, Voice, VoiceId,
    };

    #[test]
    fn test_basic_note_transformation() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Note with staccato (shorthand '.') and trill (explicit '!trill!')
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: vec![
                Decoration::Shorthand('.'),
                Decoration::Explicit("trill".into()),
            ],
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
                    articulations: vec![Articulation::Staccato, Articulation::Trill],
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
    fn test_grace_notes_emitted_before_main_note() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Grace notes {AB}
        let grace_notes = ast::GraceNotes {
            notes: vec![
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::A,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::B,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: Vec::new(),
                    broken_rhythm: None,
                },
            ],
            acciaccatura: false,
        };

        transformer.process_element(&BodyElement::GraceNotes(grace_notes));

        // Main note C
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
                            pitch_class: PitchClass::A,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 32),
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
                            pitch_class: PitchClass::B,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 32),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 32),
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
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 16),
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
    fn test_acciaccatura_shorter_duration() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Acciaccatura {/A}
        let grace_notes = ast::GraceNotes {
            notes: vec![ast::Note {
                pitch: NotePitch {
                    base: PitchClass::A,
                    accidental: None,
                    octave: Octave(0),
                },
                duration: NoteDuration::Default,
                decorations: Vec::new(),
                broken_rhythm: None,
            }],
            acciaccatura: true,
        };

        transformer.process_element(&BodyElement::GraceNotes(grace_notes));

        // Main note C
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
                            pitch_class: PitchClass::A,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 64),
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
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 64),
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
}
