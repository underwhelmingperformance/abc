//! Core body transformer structure and main dispatch logic.

use std::{collections::HashMap, rc::Rc};

use crate::{
    transform::context::TransformContext,
    types::{
        ast::{self, BodyElement, Decoration, MacroDefinition},
        AnnotationEvent, AnnotationPlacement, Context, Duration, DynamicDirection, Dynamics,
        Event, GuitarChordEvent, Measure, PartLabel, Voice, VoiceId,
    },
};

/// Tracks the position of the last note or chord for tie handling.
#[derive(Debug, Clone)]
pub(crate) enum LastNotePosition {
    /// Event is in the current (uncommitted) measure's events.
    CurrentMeasure(usize),
    /// Event is in a committed measure (measure_index, event_index).
    CommittedMeasure(usize, usize),
}

/// Transforms tune body elements into measures with resolved events.
pub(crate) struct BodyTransformer<'a> {
    /// The current musical context.
    pub(super) context_builder: TransformContext<'a>,
    /// The current context as an `Rc` (shared until it changes).
    pub(super) current_context: Rc<Context<'a>>,
    /// Voices being built.
    pub(super) voices: HashMap<VoiceId<'a>, Vec<Measure<'a>>>,
    /// The current voice.
    pub(super) current_voice: VoiceId<'a>,
    /// Events accumulated for the current measure.
    pub(super) current_measure_events: Vec<Event<'a>>,
    /// Measure counter per voice.
    pub(super) measure_numbers: HashMap<VoiceId<'a>, u32>,
    /// Absolute time from the start of the tune per voice.
    pub(super) absolute_times: HashMap<VoiceId<'a>, Duration>,
    /// Pending broken rhythm adjustment for the next note.
    ///
    /// When a note has a broken rhythm operator, this stores the adjustment
    /// to be applied to the next note.
    pub(super) pending_broken_rhythm: Option<ast::BrokenRhythm>,
    /// Whether a tie is pending for the next note.
    ///
    /// When a TieStart is encountered, this is set to true. The next note
    /// will extend the previous note's duration rather than creating a new note.
    pub(super) pending_tie: bool,
    /// Index of the last note/chord event in the current or previous measure.
    ///
    /// Used to extend durations for ties, especially across barlines.
    /// The tuple is (measure_index, event_index) where measure_index is relative
    /// to the current voice's measures, or None if the event is in current_measure_events.
    pub(super) last_note_position: Option<LastNotePosition>,
    /// Pending grace notes to be played before the next main note.
    ///
    /// Grace notes are stored here when encountered, then emitted as events
    /// before the next main note. We clone the AST grace notes since we may
    /// receive them from macro expansions with shorter lifetimes.
    pub(super) pending_grace_notes: Option<ast::GraceNotes<'a>>,
    /// Pending decorations to be attached to the next note or chord.
    ///
    /// Standalone decorations (those not attached to a note in the input)
    /// are accumulated here and attached to the next note or chord.
    pub(super) pending_decorations: Vec<Decoration<'a>>,
    /// The current dynamics level.
    ///
    /// Dynamics persist until a new dynamics marking is encountered.
    /// Defaults to mezzo-forte (MF).
    pub(super) current_dynamics: Dynamics,
    /// The current dynamic direction (crescendo or diminuendo).
    ///
    /// Set by CrescendoStart/DiminuendoStart decorations, cleared by
    /// CrescendoEnd/DiminuendoEnd decorations. Applied to notes/chords
    /// between hairpin markers.
    pub(super) current_dynamic_direction: DynamicDirection,
    /// Indices of notes/chords in current_measure_events that need lyrics.
    ///
    /// When a lyric line is encountered, syllables are aligned to these events.
    /// Reset after each lyric line or at barlines.
    pub(super) pending_lyric_notes: Vec<usize>,
    /// Number of slurs waiting to start on the next note.
    ///
    /// When a SlurStart `(` is encountered, this is incremented.
    /// When a note/chord is created, this value is transferred to slur_starts.
    pub(super) pending_slur_starts: u8,
    /// Number of dotted slurs waiting to start on the next note.
    ///
    /// When a dotted SlurStart `.(` is encountered, this is incremented.
    /// When a note/chord is created, this value is transferred to dotted_slur_starts.
    pub(super) pending_dotted_slur_starts: u8,
    /// Macro definitions from the tune header.
    ///
    /// Maps macro symbol characters to their definitions for expansion
    /// when MacroInvocation elements are encountered.
    pub(super) macros: HashMap<char, Rc<MacroDefinition<'a>>>,
    /// The current part marker (A, B, etc.) if any.
    pub(super) current_part: Option<PartLabel>,
    /// The current variant ending(s) being processed.
    ///
    /// When a variant ending (|1, |2, etc.) is encountered, this is set to
    /// the variant numbers. Measures created while this is set will be marked
    /// as belonging to that variant.
    pub(super) current_variant: Option<Vec<u8>>,
    /// Voice names from voice declarations.
    ///
    /// Stores the name for each voice, if specified in V: fields.
    pub(super) voice_names: HashMap<VoiceId<'a>, Option<&'a str>>,
}

impl<'a> BodyTransformer<'a> {
    /// Create a new body transformer with initial context and macro definitions.
    pub(crate) fn new(
        context_builder: TransformContext<'a>,
        macros: HashMap<char, Rc<MacroDefinition<'a>>>,
    ) -> Self {
        let current_context = context_builder.build_context();
        let mut voices = HashMap::new();
        voices.insert(VoiceId::new("1"), Vec::new());

        let mut measure_numbers = HashMap::new();
        measure_numbers.insert(VoiceId::new("1"), 1);

        let mut absolute_times = HashMap::new();
        absolute_times.insert(VoiceId::new("1"), Duration::new(0, 1));

        Self {
            context_builder,
            current_context,
            voices,
            current_voice: VoiceId::new("1"),
            current_measure_events: Vec::new(),
            measure_numbers,
            absolute_times,
            pending_broken_rhythm: None,
            pending_tie: false,
            last_note_position: None,
            pending_grace_notes: None,
            pending_decorations: Vec::new(),
            current_dynamics: Dynamics::default(),
            current_dynamic_direction: DynamicDirection::None,
            pending_lyric_notes: Vec::new(),
            pending_slur_starts: 0,
            pending_dotted_slur_starts: 0,
            macros,
            current_part: None,
            current_variant: None,
            voice_names: HashMap::new(),
        }
    }

    /// Get the current voice's measure number.
    pub(super) fn measure_number(&self) -> u32 {
        *self.measure_numbers.get(&self.current_voice).unwrap_or(&1)
    }

    /// Increment the current voice's measure number.
    pub(super) fn increment_measure_number(&mut self) {
        let current = self.measure_number();
        self.measure_numbers.insert(self.current_voice, current + 1);
    }

    /// Get the current voice's absolute time.
    pub(super) fn absolute_time(&self) -> Duration {
        self.absolute_times
            .get(&self.current_voice)
            .cloned()
            .unwrap_or_else(|| Duration::new(0, 1))
    }

    /// Set the current voice's absolute time.
    pub(super) fn set_absolute_time(&mut self, time: Duration) {
        self.absolute_times.insert(self.current_voice, time);
    }

    /// Advance the current voice's absolute time by a duration.
    pub(super) fn advance_time(&mut self, duration: &Duration) {
        let current = self.absolute_time();
        let new_time = &current + duration;
        self.set_absolute_time(new_time);
    }

    /// Process a body element.
    pub(crate) fn process_element(&mut self, element: &BodyElement<'a>) {
        match element {
            BodyElement::Note(note) => {
                let idx = self.current_measure_events.len();
                if let Some(event) = self.process_note(note) {
                    self.current_measure_events.push(event);
                    // Track this note for lyric alignment
                    self.pending_lyric_notes.push(idx);
                }
            }
            BodyElement::Rest(rest) => {
                if let Some(event) = self.process_rest(rest) {
                    self.current_measure_events.push(event);
                    // Rests don't get lyrics, so we don't track them
                }
            }
            BodyElement::Chord(chord) => {
                let idx = self.current_measure_events.len();
                if let Some(event) = self.process_chord(chord) {
                    self.current_measure_events.push(event);
                    // Track this chord for lyric alignment
                    self.pending_lyric_notes.push(idx);
                }
            }
            BodyElement::BarLine(bar) => {
                self.close_measure();
                // Clear variant on repeat end, double bar, or final bar
                // (variant endings only apply within a repeat section)
                match bar {
                    ast::BarLine::RepeatEnd | ast::BarLine::RepeatBoth |
                    ast::BarLine::Double | ast::BarLine::FinalDouble => {
                        self.current_variant = None;
                    }
                    _ => {}
                }
            }
            BodyElement::VariantEnding(variant) => {
                // Close the current measure first (measures before the variant ending)
                self.close_measure();
                // Then set the variant for subsequent measures
                self.current_variant = Some(variant.variants.clone());
            }
            BodyElement::InlineField(field) => {
                self.process_inline_field(field);
            }
            BodyElement::Tuplet { spec, elements } => {
                self.process_tuplet(spec, elements);
            }
            // Handle standalone decorations - accumulate for next note/chord
            BodyElement::Decoration(decoration) => {
                // Apply dynamics immediately (they persist)
                self.apply_dynamics_from_decorations(std::slice::from_ref(decoration));
                // Store decoration to attach to next note/chord
                self.pending_decorations.push(decoration.clone());
            }
            BodyElement::LyricLine(lyric_line) => {
                self.apply_lyrics(lyric_line);
            }
            BodyElement::SlurStart { dotted } => {
                if *dotted {
                    self.pending_dotted_slur_starts += 1;
                } else {
                    self.pending_slur_starts += 1;
                }
            }
            BodyElement::SlurEnd => {
                // Apply slur end to the last note/chord
                self.apply_slur_end();
            }
            BodyElement::GuitarChord(guitar_chord) => {
                // Emit guitar chord event at current time
                let absolute_time = self.absolute_time();
                self.current_measure_events.push(
                    GuitarChordEvent {
                        symbol: guitar_chord.symbol,
                        bass_note: guitar_chord.bass_note,
                        absolute_time,
                    }
                    .into(),
                );
            }
            BodyElement::Annotation(annotation) => {
                // Emit annotation event at current time
                let absolute_time = self.absolute_time();
                let placement = match annotation.placement {
                    ast::AnnotationPlacement::Above => AnnotationPlacement::Above,
                    ast::AnnotationPlacement::Below => AnnotationPlacement::Below,
                    ast::AnnotationPlacement::Left => AnnotationPlacement::Left,
                    ast::AnnotationPlacement::Right => AnnotationPlacement::Right,
                    ast::AnnotationPlacement::CenterAbove => AnnotationPlacement::CenterAbove,
                };
                self.current_measure_events.push(
                    AnnotationEvent {
                        text: annotation.text,
                        placement,
                        absolute_time,
                    }
                    .into(),
                );
            }
            // Process directives (MIDI settings, etc.) - they update context but don't generate events
            BodyElement::Directive(directive) => {
                self.process_directive(directive);
            }
            BodyElement::PartMarker(part) => {
                self.current_part = Some(*part);
            }
            BodyElement::VoiceSwitch(decl) => {
                self.switch_voice(decl);
            }
            BodyElement::VoiceOverlay => {
                // Voice overlay (&) is complex and not yet implemented
                // For now, we ignore it but could implement later as a separate overlay track
            }
            BodyElement::MacroInvocation(symbol) => {
                // Look up the macro definition and expand if found
                if let Some(definition) = self.macros.get(symbol) {
                    // Clone the Rc to get owned access to the content
                    let definition = Rc::clone(definition);
                    for inner_element in definition.content.iter() {
                        self.process_element(inner_element);
                    }
                }
                // If macro not found, silently ignore (undefined macro)
            }
            BodyElement::GraceNotes(grace_notes) => {
                self.pending_grace_notes = Some(grace_notes.clone());
            }
            BodyElement::TieStart => {
                self.pending_tie = true;
            }
            BodyElement::TieEnd => {}
        }
    }

    /// Finalise and return the voices.
    pub(crate) fn into_voices(mut self) -> HashMap<VoiceId<'a>, Voice<'a>> {
        // Close the final measure if needed
        if !self.current_measure_events.is_empty() {
            self.close_measure();
        }

        self.voices
            .into_iter()
            .map(|(id, measures)| {
                let voice = Voice {
                    id,
                    name: self.voice_names.get(&id).copied().flatten(),
                    measures,
                };
                (id, voice)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        transform::body::test_utils::create_test_context,
        types::{
            ast::{BodyElement, MacroDefinition, NoteDuration, NotePitch},
            DynamicDirection, Measure, MeasureNumber, Note, Octave, Pitch, PitchClass, Voice,
            VoiceId,
        },
    };

    #[test]
    fn test_absolute_time_tracking() {
        let context = create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        for _ in 0..3 {
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
        }

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
    fn test_macro_expansion_tilde_format() {
        let context = create_test_context();
        let context_rc = context.build_context();

        // Define a macro ~T that expands to two eighth notes: C D
        let macro_content = vec![
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
        ];

        let macro_def = MacroDefinition {
            symbol: 'T',
            content: macro_content,
        };

        let mut macros = HashMap::new();
        macros.insert('T', Rc::new(macro_def));

        let mut transformer = BodyTransformer::new(context, macros);

        // Invoke the macro
        transformer.process_element(&BodyElement::MacroInvocation('T'));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        // Should have expanded to two notes: C and D
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
    fn test_undefined_macro_ignored() {
        let context = create_test_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Try to invoke an undefined macro - should be silently ignored
        transformer.process_element(&BodyElement::MacroInvocation('X'));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        // Should have no events (just an empty measure)
        assert!(voice.measures.is_empty() || voice.measures[0].events.is_empty());
    }
}
