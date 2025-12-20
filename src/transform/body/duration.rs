//! Duration resolution and rhythmic transformations.

use super::BodyTransformer;
use crate::types::{
    Duration, Event, MarkerEvent, TimedEvent,
    ast::{self, BodyElement, NoteDuration},
};

// Broken rhythm coefficients for duration adjustment.
// These represent the multipliers applied to notes affected by broken rhythm operators.

/// Broken rhythm `>` (dot first): first note gets 3/2 of its duration
const BROKEN_RHYTHM_DOT_FIRST: (u32, u32) = (3, 2);
/// Broken rhythm `>` (dot first): second note gets 1/2 of its duration
const BROKEN_RHYTHM_DOT_SECOND: (u32, u32) = (1, 2);
/// Broken rhythm `>>` (double-dot first): first note gets 7/4 of its duration
const BROKEN_RHYTHM_DOUBLEDOT_FIRST: (u32, u32) = (7, 4);
/// Broken rhythm `>>` (double-dot first): second note gets 1/4 of its duration
const BROKEN_RHYTHM_DOUBLEDOT_SECOND: (u32, u32) = (1, 4);
/// Broken rhythm `<` (dot second): first note gets 1/2 of its duration
const BROKEN_RHYTHM_REVERSE_DOT_FIRST: (u32, u32) = (1, 2);
/// Broken rhythm `<` (dot second): second note gets 3/2 of its duration
const BROKEN_RHYTHM_REVERSE_DOT_SECOND: (u32, u32) = (3, 2);
/// Broken rhythm `<<` (double-dot second): first note gets 1/4 of its duration
const BROKEN_RHYTHM_REVERSE_DOUBLEDOT_FIRST: (u32, u32) = (1, 4);
/// Broken rhythm `<<` (double-dot second): second note gets 7/4 of its duration
const BROKEN_RHYTHM_REVERSE_DOUBLEDOT_SECOND: (u32, u32) = (7, 4);

impl<'a> BodyTransformer<'a> {
    /// Resolve a note duration from the AST.
    pub(super) fn resolve_duration(&self, duration: NoteDuration) -> Duration {
        match duration {
            NoteDuration::Default => *self.context_builder.unit_length(),
            NoteDuration::Explicit {
                numerator,
                denominator,
            } => {
                let unit_length = self.context_builder.unit_length();
                // duration = unit_length * (numerator / denominator)
                Duration::new(
                    unit_length.numerator() * numerator,
                    unit_length.denominator() * denominator,
                )
                .normalise()
            }
        }
    }

    /// Apply broken rhythm adjustment to the first note.
    ///
    /// The first note is lengthened by the broken rhythm operator:
    /// - `>`: 3/2× (dotted) or custom ratio if set
    /// - `>>`: 7/4× (double-dotted) or custom ratio if set
    pub(super) fn apply_broken_rhythm_to_first(
        &self,
        duration: Duration,
        br: ast::BrokenRhythm,
    ) -> Duration {
        let (num_mult, denom_mult) =
            if let Some((num, denom)) = self.context_builder.broken_rhythm_ratio() {
                // Use custom ratio if set
                match br {
                    ast::BrokenRhythm::DotFirst | ast::BrokenRhythm::DoubleDotFirst => {
                        (num as u32, denom as u32)
                    }
                    ast::BrokenRhythm::DotSecond | ast::BrokenRhythm::DoubleDotSecond => {
                        (denom as u32, num as u32)
                    }
                }
            } else {
                // Use default ratios
                match br {
                    ast::BrokenRhythm::DotFirst => BROKEN_RHYTHM_DOT_FIRST,
                    ast::BrokenRhythm::DoubleDotFirst => BROKEN_RHYTHM_DOUBLEDOT_FIRST,
                    ast::BrokenRhythm::DotSecond => BROKEN_RHYTHM_REVERSE_DOT_FIRST,
                    ast::BrokenRhythm::DoubleDotSecond => BROKEN_RHYTHM_REVERSE_DOUBLEDOT_FIRST,
                }
            };
        Duration::new(
            duration.numerator() * num_mult,
            duration.denominator() * denom_mult,
        )
    }

    /// Apply broken rhythm adjustment to the second note.
    ///
    /// The second note's duration is adjusted complementarily:
    /// - After `>`: 1/2× (halved) or custom ratio if set
    /// - After `>>`: 1/4× (quartered) or custom ratio if set
    pub(super) fn apply_broken_rhythm_to_second(
        &self,
        duration: Duration,
        br: ast::BrokenRhythm,
    ) -> Duration {
        let (num_mult, denom_mult) =
            if let Some((num, denom)) = self.context_builder.broken_rhythm_ratio() {
                // Use custom ratio if set
                // The second note gets the complementary ratio
                // If first note gets num/denom, second gets (2*denom - num)/denom
                // But for simplicity, we use the reciprocal approach
                match br {
                    ast::BrokenRhythm::DotFirst | ast::BrokenRhythm::DoubleDotFirst => {
                        // Second note gets smaller duration
                        (2 * denom as u32 - num as u32, denom as u32)
                    }
                    ast::BrokenRhythm::DotSecond | ast::BrokenRhythm::DoubleDotSecond => {
                        // Second note gets larger duration
                        (2 * num as u32 - denom as u32, num as u32)
                    }
                }
            } else {
                // Use default ratios
                match br {
                    ast::BrokenRhythm::DotFirst => BROKEN_RHYTHM_DOT_SECOND,
                    ast::BrokenRhythm::DoubleDotFirst => BROKEN_RHYTHM_DOUBLEDOT_SECOND,
                    ast::BrokenRhythm::DotSecond => BROKEN_RHYTHM_REVERSE_DOT_SECOND,
                    ast::BrokenRhythm::DoubleDotSecond => BROKEN_RHYTHM_REVERSE_DOUBLEDOT_SECOND,
                }
            };
        Duration::new(
            duration.numerator() * num_mult,
            duration.denominator() * denom_mult,
        )
    }

    /// Process a tuplet, scaling durations of contained notes.
    pub(super) fn process_tuplet(&mut self, spec: &ast::Tuplet, elements: &[BodyElement<'a>]) {
        // Infer q (notes replaced) if not specified
        // Common default: (3 means 3:2, (2 means 2:3
        let q = spec.q.unwrap_or_else(|| {
            if spec.p.is_multiple_of(3) {
                (spec.p * 2) / 3
            } else if spec.p.is_multiple_of(2) {
                (spec.p * 3) / 2
            } else {
                // For odd p not divisible by 3, use next lower power of 2
                let mut q = 1;
                while q * 2 < spec.p {
                    q *= 2;
                }
                q
            }
        });

        // Process each element in the tuplet
        // We need to scale durations by q/p
        // Store events before processing tuplet and the starting absolute time
        let events_before = self.current_measure_events.len();
        let start_time = self.absolute_time();

        for element in elements {
            self.process_element(element);
        }

        // Scale the durations of all events added by the tuplet
        for event in &mut self.current_measure_events[events_before..] {
            match event {
                Event::Timed(TimedEvent::Note(note)) => {
                    note.duration = Duration::new(
                        note.duration.numerator() * q as u32,
                        note.duration.denominator() * spec.p as u32,
                    )
                    .normalise();
                }
                Event::Timed(TimedEvent::Rest(rest)) => {
                    rest.duration = Duration::new(
                        rest.duration.numerator() * q as u32,
                        rest.duration.denominator() * spec.p as u32,
                    )
                    .normalise();
                }
                Event::Timed(TimedEvent::Chord(chord)) => {
                    chord.duration = Duration::new(
                        chord.duration.numerator() * q as u32,
                        chord.duration.denominator() * spec.p as u32,
                    )
                    .normalise();
                }
                Event::Marker(_) => {
                    // Markers have zero duration, no scaling needed
                }
            }
        }

        // Recalculate absolute times for all events after the tuplet start
        let mut time = if events_before > 0 {
            match &self.current_measure_events[events_before - 1] {
                Event::Timed(t) => t.absolute_time() + t.duration(),
                Event::Marker(m) => *m.absolute_time(),
            }
        } else {
            start_time
        };

        for event in &mut self.current_measure_events[events_before..] {
            match event {
                Event::Timed(TimedEvent::Note(note)) => {
                    note.absolute_time = time;
                    time = &time + &note.duration;
                }
                Event::Timed(TimedEvent::Rest(rest)) => {
                    rest.absolute_time = time;
                    time = &time + &rest.duration;
                }
                Event::Timed(TimedEvent::Chord(chord)) => {
                    chord.absolute_time = time;
                    time = &time + &chord.duration;
                }
                Event::Marker(MarkerEvent::GuitarChord(chord)) => {
                    chord.absolute_time = time;
                    // Zero duration, time doesn't advance
                }
                Event::Marker(MarkerEvent::Annotation(annotation)) => {
                    annotation.absolute_time = time;
                    // Zero duration, time doesn't advance
                }
                Event::Marker(MarkerEvent::MidiControl(control)) => {
                    control.absolute_time = time;
                    // Zero duration, time doesn't advance
                }
                Event::Marker(MarkerEvent::MidiPitchBend(pitch_bend)) => {
                    pitch_bend.absolute_time = time;
                    // Zero duration, time doesn't advance
                }
            }
        }

        self.set_absolute_time(time);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::super::{BodyTransformer, test_utils::create_test_context};
    use crate::types::{
        Duration, DynamicDirection, Dynamics, Measure, MeasureNumber, Note, Octave, Pitch,
        PitchClass, Voice, VoiceId,
        ast::{self, BodyElement, NoteDuration, NotePitch},
    };

    #[test]
    fn test_duration_resolution() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Default duration
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

        // Doubled duration
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::D,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Explicit {
                numerator: 2,
                denominator: 1,
            },
            decorations: Vec::new(),
            broken_rhythm: None,
        }));

        // Halved duration
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::E,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Explicit {
                numerator: 1,
                denominator: 2,
            },
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
                            pitch_class: PitchClass::D,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 4),
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
                    Note {
                        pitch: Pitch {
                            pitch_class: PitchClass::E,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 16),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(3, 8),
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
    fn test_broken_rhythm_dot_first() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // C with > (dot first) - C should be 3/2×, D should be 1/2×
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: Some(ast::BrokenRhythm::DotFirst),
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
                        duration: Duration::new(3, 16), // 1/8 * 3/2
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
                        duration: Duration::new(1, 16), // 1/8 * 1/2
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(3, 16),
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
    fn test_broken_rhythm_dot_second() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // C with < (dot second) - C should be 1/2×, D should be 3/2×
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: Vec::new(),
            broken_rhythm: Some(ast::BrokenRhythm::DotSecond),
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
                        duration: Duration::new(1, 16),
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
                        duration: Duration::new(3, 16),
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
    fn test_tuplet_triplet() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // (3ABC - triplet (3 notes in time of 2)
        let tuplet_spec = ast::Tuplet {
            p: 3,
            q: None, // Should infer to 2
            r: None,
        };

        let tuplet_notes = vec![
            BodyElement::Note(ast::Note {
                pitch: NotePitch {
                    base: PitchClass::C,
                    accidental: None,
                    octave: Octave(0),
                },
                duration: NoteDuration::Default,
                decorations: Vec::new(),
                broken_rhythm: None,
            }),
            BodyElement::Note(ast::Note {
                pitch: NotePitch {
                    base: PitchClass::D,
                    accidental: None,
                    octave: Octave(0),
                },
                duration: NoteDuration::Default,
                decorations: Vec::new(),
                broken_rhythm: None,
            }),
            BodyElement::Note(ast::Note {
                pitch: NotePitch {
                    base: PitchClass::E,
                    accidental: None,
                    octave: Octave(0),
                },
                duration: NoteDuration::Default,
                decorations: Vec::new(),
                broken_rhythm: None,
            }),
        ];

        transformer.process_element(&BodyElement::Tuplet {
            spec: tuplet_spec,
            elements: tuplet_notes,
        });

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
                        duration: Duration::new(1, 12),
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
                        duration: Duration::new(1, 12),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 12),
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
                            pitch_class: PitchClass::E,
                            accidental: None,
                            octave: Octave(4),
                        },
                        duration: Duration::new(1, 12),
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 6),
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
