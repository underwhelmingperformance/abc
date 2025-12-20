//! Integration tests for ABC parser.
//!
//! These tests parse complete ABC files and verify the full parse → transform pipeline.

use abc::{parse, types::*};

#[test]
fn test_parse_the_kesh_jig() {
    let input = include_str!("fixtures/simple_jig.abc");
    let doc = parse(input).unwrap();

    assert_eq!(doc.tunes.len(), 1);

    let tune = &doc.tunes[0];
    assert_eq!(tune.reference_number, ReferenceNumber(1));
    assert_eq!(tune.title, "The Kesh Jig");
    assert_eq!(tune.rhythm, Some("Jig"));
    assert_eq!(tune.meter.numerator, 6);
    assert_eq!(tune.meter.denominator, 8);
    assert_eq!(tune.key.tonic, PitchClass::G);
    assert_eq!(tune.key.mode, Mode::Major);

    // Should have one voice
    assert_eq!(tune.voices.len(), 1);

    let voice = tune.primary_voice();
    assert_eq!(voice.id, VoiceId::new("1"));

    // Should have 16 measures (8 bars per repeat section × 2 sections)
    assert_eq!(voice.measures.len(), 16);
}

#[test]
fn test_parse_multi_voice() {
    let input = include_str!("fixtures/multi_voice.abc");
    let doc = parse(input).unwrap();

    assert_eq!(doc.tunes.len(), 1);

    let tune = &doc.tunes[0];
    assert_eq!(tune.title, "Two Voice Example");

    // Should have two voices
    assert_eq!(tune.voices.len(), 2);

    let melody = tune.voices.get(&VoiceId::new("1")).unwrap();
    assert_eq!(melody.name, Some("Melody"));
    assert_eq!(melody.measures.len(), 2);

    let bass = tune.voices.get(&VoiceId::new("2")).unwrap();
    assert_eq!(bass.name, Some("Bass"));
    assert_eq!(bass.measures.len(), 2);
}

#[test]
fn test_parse_variant_endings() {
    let input = include_str!("fixtures/variant_endings.abc");
    let doc = parse(input).unwrap();

    assert_eq!(doc.tunes.len(), 1);

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();

    // Should have 3 measures: common, variant 1, variant 2
    assert_eq!(voice.measures.len(), 3);

    // First measure has no variant (played both times)
    assert_eq!(voice.measures[0].variant, None);

    // Second measure is variant 1 (first time only)
    assert_eq!(voice.measures[1].variant, Some(vec![1]));

    // Third measure is variant 2 (second time only)
    assert_eq!(voice.measures[2].variant, Some(vec![2]));
}

#[test]
fn test_parse_with_dynamics() {
    let input = "X:1\nT:Dynamics Test\nK:C\n!p!C !f!D !mf!E|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();
    let measure = &voice.measures[0];

    // Three notes with different dynamics
    let expected_events = vec![
        Note {
            pitch: Pitch::new(PitchClass::C, Octave(4), None),
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
        }.into(),
        Note {
            pitch: Pitch::new(PitchClass::D, Octave(4), None),
            duration: Duration::new(1, 8),
            dynamics: Dynamics::F,
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
            dynamics: Dynamics::MF,
            dynamic_direction: DynamicDirection::None,
            absolute_time: Duration::new(1, 4),
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }.into(),
    ];

    assert_eq!(measure.events, expected_events);
}

#[test]
fn test_parse_with_slurs() {
    let input = "X:1\nT:Slur Test\nK:C\n(CDE)|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();
    let measure = &voice.measures[0];

    let expected_events = vec![
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
    ];

    assert_eq!(measure.events, expected_events);
}

#[test]
fn test_parse_with_ties() {
    let input = "X:1\nT:Tie Test\nK:C\nC2-|C2 D2|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();

    assert_eq!(voice.measures.len(), 2);

    // First measure has one tied note
    let measure1 = &voice.measures[0];
    let expected_events_1 = vec![
        Note {
            pitch: Pitch::new(PitchClass::C, Octave(4), None),
            duration: Duration::new(1, 2),
            dynamics: Dynamics::MF,
            dynamic_direction: DynamicDirection::None,
            absolute_time: Duration::new(0, 1),
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }.into(),
    ];
    assert_eq!(measure1.events, expected_events_1);

    // Second measure has another note (the tie was resolved in measure 1)
    let measure2 = &voice.measures[1];
    let expected_events_2 = vec![
        Note {
            pitch: Pitch::new(PitchClass::D, Octave(4), None),
            duration: Duration::new(1, 4),
            dynamics: Dynamics::MF,
            dynamic_direction: DynamicDirection::None,
            absolute_time: Duration::new(1, 2),
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }.into(),
    ];
    assert_eq!(measure2.events, expected_events_2);
}

#[test]
fn test_parse_with_chords() {
    let input = "X:1\nT:Chord Test\nK:C\n[CEG]2|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();
    let measure = &voice.measures[0];

    let expected_events = vec![
        Chord {
            pitches: vec![
                Pitch::new(PitchClass::C, Octave(4), None),
                Pitch::new(PitchClass::E, Octave(4), None),
                Pitch::new(PitchClass::G, Octave(4), None),
            ],
            duration: Duration::new(1, 4),
            dynamics: Dynamics::default(),
            dynamic_direction: DynamicDirection::None,
            absolute_time: Duration::new(0, 1),
            articulations: Vec::new(),
            lyric: None,
            slur_starts: 0,
            slur_ends: 0,
            dotted_slur_starts: 0,
            dotted_slur_ends: 0,
        }.into(),
    ];

    assert_eq!(measure.events, expected_events);
}

#[test]
fn test_parse_with_tuplets() {
    let input = "X:1\nT:Tuplet Test\nK:C\n(3CDE|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();
    let measure = &voice.measures[0];

    // Three notes in a triplet - each note in triplet has 2/3 duration (3 notes in time of 2)
    let expected_events = vec![
        Note {
            pitch: Pitch::new(PitchClass::C, Octave(4), None),
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
        }.into(),
        Note {
            pitch: Pitch::new(PitchClass::D, Octave(4), None),
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
        }.into(),
        Note {
            pitch: Pitch::new(PitchClass::E, Octave(4), None),
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
        }.into(),
    ];

    assert_eq!(measure.events, expected_events);
}

#[test]
fn test_parse_with_key_changes() {
    let input = "X:1\nT:Key Change Test\nK:C\nCDE|[K:G]GAB|\n";
    let doc = parse(input).unwrap();

    let tune = &doc.tunes[0];
    let voice = tune.primary_voice();

    assert_eq!(voice.measures.len(), 2);

    // First measure in C major
    assert_eq!(voice.measures[0].context.key.tonic, PitchClass::C);

    // Second measure in G major
    assert_eq!(voice.measures[1].context.key.tonic, PitchClass::G);

    // F should be sharp in G major
    assert_eq!(
        voice.measures[1].context.key.get_accidental(PitchClass::F),
        Some(Accidental::Sharp)
    );
}

#[test]
fn test_parse_multiple_tunes_in_document() {
    let input = "X:1\nT:First Tune\nK:C\nC|\n\nX:2\nT:Second Tune\nK:D\nD|\n";
    let doc = parse(input).unwrap();

    assert_eq!(doc.tunes.len(), 2);

    assert_eq!(doc.tunes[0].reference_number, ReferenceNumber(1));
    assert_eq!(doc.tunes[0].title, "First Tune");

    assert_eq!(doc.tunes[1].reference_number, ReferenceNumber(2));
    assert_eq!(doc.tunes[1].title, "Second Tune");
}
