//! Lyric alignment to notes.

use crate::types::{
    Event, Syllable, TimedEvent,
    ast::lyrics::{LyricLine, LyricSyllable},
};

use super::BodyTransformer;

impl<'a> BodyTransformer<'a> {
    /// Apply lyrics from a lyric line to pending notes.
    ///
    /// ABC lyric alignment rules:
    /// - Each syllable aligns to one note in order
    /// - Hold (_) means the previous syllable continues (note gets no new lyric)
    /// - Skip (*) means the note has no lyric
    /// - Space between syllables is handled by the parser
    /// - BarLine (|) is visual only and doesn't affect alignment
    pub(super) fn apply_lyrics(&mut self, lyric_line: &LyricLine<'a>) {
        let mut note_iter = self.pending_lyric_notes.iter().copied();

        for syllable in &lyric_line.syllables {
            match syllable {
                LyricSyllable::Text { text, continues } => {
                    // Assign this syllable to the next note
                    if let Some(note_idx) = note_iter.next()
                        && let Some(event) = self.current_measure_events.get_mut(note_idx)
                    {
                        match event {
                            Event::Timed(TimedEvent::Note(note)) => {
                                note.lyric = Some(Syllable::new(text, *continues));
                            }
                            Event::Timed(TimedEvent::Chord(chord)) => {
                                chord.lyric = Some(Syllable::new(text, *continues));
                            }
                            Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {
                                // These don't get lyrics - skip
                            }
                        }
                    }
                }
                LyricSyllable::Hold => {
                    // Hold means extend previous syllable - mark this note as held
                    if let Some(note_idx) = note_iter.next()
                        && let Some(event) = self.current_measure_events.get_mut(note_idx)
                    {
                        match event {
                            Event::Timed(TimedEvent::Note(note)) => {
                                note.lyric = Some(Syllable::hold());
                            }
                            Event::Timed(TimedEvent::Chord(chord)) => {
                                chord.lyric = Some(Syllable::hold());
                            }
                            Event::Timed(TimedEvent::Rest(_)) | Event::Marker(_) => {}
                        }
                    }
                }
                LyricSyllable::Skip => {
                    // Skip means this note has no lyric
                    let _ = note_iter.next();
                }
                LyricSyllable::Space => {
                    // Space between syllables - already handled by text separation
                    // Don't consume a note
                }
                LyricSyllable::BarLine => {
                    // Visual marker only - doesn't affect alignment
                }
            }
        }

        // Clear pending lyric notes after processing
        self.pending_lyric_notes.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::{
        Duration, DynamicDirection, Dynamics, Measure, MeasureNumber, Note, Octave, Pitch,
        PitchClass, Syllable, Voice, VoiceId,
        ast::{self, BodyElement, NoteDuration, NotePitch},
        ast::lyrics::{LyricLine, LyricSyllable},
    };

    #[test]
    fn test_lyrics_alignment() {
        let context = super::super::test_utils::create_test_context();
        let context_rc = context.build_context();
        let mut transformer = BodyTransformer::new(context, HashMap::new());

        // Create three notes
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

        // Process a lyric line with syllables: "Hel-lo world"
        // "Hel-" continues to next syllable
        // "lo" continues to next word
        // "world" is complete
        transformer.process_element(&BodyElement::LyricLine(LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "Hel",
                    continues: true,
                },
                LyricSyllable::Text {
                    text: "lo",
                    continues: false,
                },
                LyricSyllable::Text {
                    text: "world",
                    continues: false,
                },
            ],
        }));

        let voices = transformer.into_voices();
        let voice = voices.get(&VoiceId::new("1")).unwrap();

        // Expected: notes should have lyrics aligned
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
                        lyric: Some(Syllable::Text {
                            text: "Hel",
                            continues: true,
                        }),
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
                        lyric: Some(Syllable::Text {
                            text: "lo",
                            continues: false,
                        }),
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
                        dynamics: Dynamics::default(),
                        dynamic_direction: DynamicDirection::None,
                        absolute_time: Duration::new(1, 4),
                        articulations: Vec::new(),
                        lyric: Some(Syllable::Text {
                            text: "world",
                            continues: false,
                        }),
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
