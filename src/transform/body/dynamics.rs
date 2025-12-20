//! Dynamics and articulation handling.

use crate::types::{
    Articulation, DynamicDirection,
    ast::{Decoration, Dynamic},
};

use super::BodyTransformer;

impl<'a> BodyTransformer<'a> {
    /// Extract and apply dynamics from decorations.
    ///
    /// Updates `current_dynamics` if a volume level is found, and
    /// `current_dynamic_direction` if hairpin markers are present.
    pub(super) fn apply_dynamics_from_decorations(&mut self, decorations: &[Decoration]) {
        for decoration in decorations {
            if let Decoration::Dynamic(dynamic) = decoration {
                match dynamic {
                    // Volume levels update the sustained dynamics
                    Dynamic::Level(level) => {
                        self.current_dynamics = *level;
                    }
                    // Hairpin markers update dynamic direction
                    Dynamic::CrescendoStart => {
                        self.current_dynamic_direction = DynamicDirection::Crescendo;
                    }
                    Dynamic::CrescendoEnd => {
                        self.current_dynamic_direction = DynamicDirection::None;
                    }
                    Dynamic::DiminuendoStart => {
                        self.current_dynamic_direction = DynamicDirection::Diminuendo;
                    }
                    Dynamic::DiminuendoEnd => {
                        self.current_dynamic_direction = DynamicDirection::None;
                    }
                    // Sfz is an articulation rather than a sustained dynamic
                    Dynamic::Sfz => {}
                }
            }
        }
    }

    /// Extract articulations from AST decorations.
    ///
    /// Converts shorthand and explicit decorations to semantic Articulation types.
    /// Dynamic decorations are excluded (they're handled separately).
    pub(super) fn extract_articulations(decorations: &[Decoration]) -> Vec<Articulation> {
        decorations
            .iter()
            .filter_map(|dec| match dec {
                Decoration::Shorthand(c) => Articulation::from_shorthand(*c),
                Decoration::Explicit(name) => Articulation::from_explicit(name),
                Decoration::Dynamic(_) => None, // Dynamics handled separately
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::create_test_context;
    use super::super::BodyTransformer;
    use crate::types::{
        ast::{self, BodyElement, Decoration, Dynamic, NoteDuration, NotePitch},
        Articulation, Chord, DynamicDirection, Dynamics, Duration, Measure, MeasureNumber, Note,
        Octave, Pitch, PitchClass, Voice, VoiceId,
    };
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_dynamics_from_decoration() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Note with !p! decoration
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: vec![Decoration::Dynamic(Dynamic::Level(Dynamics::P))],
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
                    dynamics: Dynamics::P,
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
    fn test_dynamics_persist_across_notes() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // First note with !ff! decoration
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::C,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: vec![Decoration::Dynamic(Dynamic::Level(Dynamics::FF))],
            broken_rhythm: None,
        }));

        // Second note without decoration (should inherit ff)
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

        // Third note with !p! decoration (changes dynamics)
        transformer.process_element(&BodyElement::Note(ast::Note {
            pitch: NotePitch {
                base: PitchClass::E,
                accidental: None,
                octave: Octave(0),
            },
            duration: NoteDuration::Default,
            decorations: vec![Decoration::Dynamic(Dynamic::Level(Dynamics::P))],
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
                        dynamics: Dynamics::FF,
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
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::FF, // Inherited from first note
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
                        duration: Duration::new(1, 8),
                        dynamics: Dynamics::P, // Changed
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
            }],
        };

        assert_eq!(voice, &expected);
    }

    #[test]
    fn test_chord_dynamics_and_articulations() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Chord with !f! dynamic and !accent! articulation on notes
        // Dynamics should go to dynamics field, articulations to articulations field
        transformer.process_element(&BodyElement::Chord(ast::Chord {
            notes: vec![
                ast::Note {
                    pitch: NotePitch {
                        base: PitchClass::C,
                        accidental: None,
                        octave: Octave(0),
                    },
                    duration: NoteDuration::Default,
                    decorations: vec![
                        Decoration::Dynamic(Dynamic::Level(Dynamics::F)),
                        Decoration::Explicit("accent".into()),
                    ],
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
                    decorations: vec![Decoration::Shorthand('.')], // staccato
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
                    duration: Duration::new(1, 8),
                    dynamics: Dynamics::F,
                    dynamic_direction: DynamicDirection::None,
                    absolute_time: Duration::new(0, 1),
                    // Articulations from all chord notes combined
                    articulations: vec![Articulation::Accent, Articulation::Staccato],
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
    fn test_standalone_decoration_sets_dynamics() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Standalone dynamics decoration
        transformer.process_element(&BodyElement::Decoration(Decoration::Dynamic(Dynamic::Level(Dynamics::PP))));

        // Note without decoration (should use PP from standalone decoration)
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
                    dynamics: Dynamics::PP,
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
