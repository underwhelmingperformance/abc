//! Musical event types.
//!
//! This module defines the resolved musical events that make up a tune:
//! notes, rests, chords, guitar chords, and annotations. Events use abstract
//! pitch representation that preserves musical spelling (C# vs Db) rather than
//! MIDI-specific values.

use super::{Articulation, Duration, DynamicDirection, Dynamics, Pitch, Syllable};

/// Placement of an annotation relative to the note or chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationPlacement {
    /// Above the note or chord.
    Above,
    /// Below the note or chord.
    Below,
    /// To the left of the note or chord.
    Left,
    /// To the right of the note or chord.
    Right,
    /// Centered above the note or chord.
    CenterAbove,
}

/// A musical event with absolute timing.
///
/// Events are the resolved elements that make up the music in a measure,
/// separated into timed events (which have duration and advance musical time)
/// and markers (which mark a point in time without duration).
///
/// # Examples
///
/// ```
/// use abc::types::{DynamicDirection, Event, Note, Pitch, PitchClass, Octave, Duration, Dynamics};
///
/// let note = Note {
///     pitch: Pitch::new(PitchClass::C, Octave(4), None),  // Middle C
///     duration: Duration::new(1, 4),
///     dynamics: Dynamics::MF,
///     dynamic_direction: DynamicDirection::None,
///     absolute_time: Duration::new(0, 1),
///     ..Default::default()
/// };
///
/// // Using From for ergonomic conversion:
/// let event: Event = note.into();
/// ```
#[derive(Debug, PartialEq)]
pub enum Event<'input> {
    /// An event with duration that advances musical time.
    Timed(TimedEvent<'input>),

    /// A marker at a point in time with zero duration.
    Marker(MarkerEvent<'input>),
}

/// Events that have duration and advance musical time.
#[derive(Debug, PartialEq)]
pub enum TimedEvent<'input> {
    /// A single note.
    Note(Note<'input>),

    /// A rest (silence).
    Rest(Rest),

    /// Multiple notes sounded simultaneously.
    Chord(Chord<'input>),
}

/// Events that mark a point in time without consuming duration.
#[derive(Debug, PartialEq, Eq)]
pub enum MarkerEvent<'input> {
    /// A guitar chord symbol for accompaniment.
    ///
    /// Guitar chords (e.g., "Am7", "G/B") indicate harmony and can be used
    /// for both display and generating accompaniment patterns.
    GuitarChord(GuitarChordEvent<'input>),

    /// A text annotation positioned at a specific time.
    ///
    /// Annotations are displayed above, below, or beside notes/chords in
    /// the rendered score.
    Annotation(AnnotationEvent<'input>),

    /// A MIDI control change message.
    ///
    /// Control changes modify synthesizer parameters at a specific point in time.
    MidiControl(MidiControlEvent),

    /// A MIDI pitch bend message.
    ///
    /// Pitch bend modifies the pitch of all notes on a channel at a specific point in time.
    MidiPitchBend(MidiPitchBendEvent),
}

impl<'input> TimedEvent<'input> {
    /// Get the duration of this timed event.
    pub fn duration(&self) -> &Duration {
        match self {
            TimedEvent::Note(n) => &n.duration,
            TimedEvent::Rest(r) => &r.duration,
            TimedEvent::Chord(c) => &c.duration,
        }
    }

    /// Get the absolute time of this event from the start of the tune.
    pub fn absolute_time(&self) -> &Duration {
        match self {
            TimedEvent::Note(n) => &n.absolute_time,
            TimedEvent::Rest(r) => &r.absolute_time,
            TimedEvent::Chord(c) => &c.absolute_time,
        }
    }
}

impl<'input> MarkerEvent<'input> {
    /// Get the absolute time (position) of this marker from the start of the tune.
    pub fn absolute_time(&self) -> &Duration {
        match self {
            MarkerEvent::GuitarChord(g) => &g.absolute_time,
            MarkerEvent::Annotation(a) => &a.absolute_time,
            MarkerEvent::MidiControl(c) => &c.absolute_time,
            MarkerEvent::MidiPitchBend(p) => &p.absolute_time,
        }
    }
}

impl<'input> Event<'input> {
    /// Get the duration of this event, if it has one.
    ///
    /// Returns `Some` for timed events (notes, rests, chords) and `None` for markers.
    pub fn duration(&self) -> Option<&Duration> {
        match self {
            Event::Timed(t) => Some(t.duration()),
            Event::Marker(_) => None,
        }
    }

    /// Get the absolute time of this event from the start of the tune.
    ///
    /// All events have an absolute time position, even markers with zero duration.
    pub fn absolute_time(&self) -> &Duration {
        match self {
            Event::Timed(t) => t.absolute_time(),
            Event::Marker(m) => m.absolute_time(),
        }
    }
}

// Ergonomic From implementations
impl<'input> From<Note<'input>> for Event<'input> {
    fn from(note: Note<'input>) -> Self {
        Event::Timed(TimedEvent::Note(note))
    }
}

impl From<Rest> for Event<'_> {
    fn from(rest: Rest) -> Self {
        Event::Timed(TimedEvent::Rest(rest))
    }
}

impl<'input> From<Chord<'input>> for Event<'input> {
    fn from(chord: Chord<'input>) -> Self {
        Event::Timed(TimedEvent::Chord(chord))
    }
}

impl<'input> From<GuitarChordEvent<'input>> for Event<'input> {
    fn from(chord: GuitarChordEvent<'input>) -> Self {
        Event::Marker(MarkerEvent::GuitarChord(chord))
    }
}

impl<'input> From<AnnotationEvent<'input>> for Event<'input> {
    fn from(annotation: AnnotationEvent<'input>) -> Self {
        Event::Marker(MarkerEvent::Annotation(annotation))
    }
}

impl From<MidiControlEvent> for Event<'_> {
    fn from(control: MidiControlEvent) -> Self {
        Event::Marker(MarkerEvent::MidiControl(control))
    }
}

impl From<MidiPitchBendEvent> for Event<'_> {
    fn from(pitch_bend: MidiPitchBendEvent) -> Self {
        Event::Marker(MarkerEvent::MidiPitchBend(pitch_bend))
    }
}

/// A fully resolved note with pitch and timing.
///
/// Notes have been processed with musical context to produce resolved
/// pitches and computed durations.
///
/// # Examples
///
/// ```
/// use abc::types::{DynamicDirection, Note, Pitch, PitchClass, Octave, Duration, Dynamics};
///
/// // Middle C as a quarter note
/// let note = Note {
///     pitch: Pitch::new(PitchClass::C, Octave(4), None),
///     duration: Duration::new(1, 4),
///     dynamics: Dynamics::MF,
///     dynamic_direction: DynamicDirection::None,
///     absolute_time: Duration::new(0, 1),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, PartialEq, Eq, Default, bon::Builder)]
pub struct Note<'input> {
    /// The pitch with musical spelling preserved.
    ///
    /// Unlike MIDI note numbers, this preserves the enharmonic spelling
    /// (C# vs Db) which is important for notation display.
    pub pitch: Pitch,

    /// The computed duration of this note.
    ///
    /// This has been resolved from either an explicit duration modifier
    /// or the default unit note length from the context.
    pub duration: Duration,

    /// The dynamics level for this note.
    ///
    /// Derived from dynamics markings in the notation. For MIDI output,
    /// this is converted to velocity in the MIDI export module.
    pub dynamics: Dynamics,

    /// The direction of dynamic change (crescendo or diminuendo).
    ///
    /// Indicates whether this note is part of a hairpin marking
    /// for gradual dynamic change.
    pub dynamic_direction: DynamicDirection,

    /// The absolute time from the start of the tune.
    ///
    /// This is the cumulative duration of all previous events.
    pub absolute_time: Duration,

    /// Articulations and ornaments applied to this note.
    ///
    /// These include ornaments (trills, mordents), articulations (staccato,
    /// accent), and other performance instructions. Dynamics are stored
    /// separately in the `dynamics` field.
    pub articulations: Vec<Articulation>,

    /// The lyric syllable aligned with this note, if any.
    ///
    /// Lyrics are aligned from `w:` lines in the ABC notation. Notes may
    /// have no lyric (instrumental passages, held syllables, or skipped notes).
    pub lyric: Option<Syllable<'input>>,

    /// Number of slurs starting at this note.
    ///
    /// Slurs connect multiple notes (possibly of different pitches),
    /// indicating they should be played smoothly. Slurs can nest,
    /// so this counts how many slurs begin at this note.
    pub slur_starts: u8,

    /// Number of slurs ending at this note.
    ///
    /// Counts how many slurs conclude at this note. When rendering,
    /// the renderer should close this many slur arcs.
    pub slur_ends: u8,

    /// Number of dotted slurs starting at this note.
    ///
    /// Dotted slurs are rendered with a dotted line instead of a solid line.
    /// They are used to indicate a different type of articulation or phrasing.
    pub dotted_slur_starts: u8,

    /// Number of dotted slurs ending at this note.
    ///
    /// Counts how many dotted slurs conclude at this note.
    pub dotted_slur_ends: u8,
}

/// A rest (period of silence).
///
/// Rests have duration and timing but no pitch.
///
/// # Examples
///
/// ```
/// use abc::types::{Rest, Duration};
///
/// // A quarter rest
/// let rest = Rest {
///     duration: Duration::new(1, 4),
///     absolute_time: Duration::new(0, 1),
/// };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Rest {
    /// The duration of this rest.
    pub duration: Duration,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,
}

/// Multiple notes sounded simultaneously.
///
/// Chords contain multiple pitches played at the same time with the
/// same duration and dynamics.
///
/// # Examples
///
/// ```
/// use abc::types::{Chord, DynamicDirection, Pitch, PitchClass, Octave, Duration, Dynamics};
///
/// // C major chord (C, E, G)
/// let chord = Chord {
///     pitches: vec![
///         Pitch::new(PitchClass::C, Octave(4), None),
///         Pitch::new(PitchClass::E, Octave(4), None),
///         Pitch::new(PitchClass::G, Octave(4), None),
///     ],
///     duration: Duration::new(1, 2),
///     dynamics: Dynamics::MF,
///     dynamic_direction: DynamicDirection::None,
///     absolute_time: Duration::new(0, 1),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, PartialEq, Eq, Default, bon::Builder)]
pub struct Chord<'input> {
    /// The pitches in this chord with musical spelling preserved.
    ///
    /// Typically ordered from lowest to highest, but not required.
    pub pitches: Vec<Pitch>,

    /// The duration of this chord.
    ///
    /// All notes in the chord have the same duration.
    pub duration: Duration,

    /// The dynamics level for all notes in this chord.
    pub dynamics: Dynamics,

    /// The direction of dynamic change (crescendo or diminuendo).
    ///
    /// Indicates whether this chord is part of a hairpin marking
    /// for gradual dynamic change.
    pub dynamic_direction: DynamicDirection,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,

    /// Articulations and ornaments applied to this chord.
    ///
    /// These apply to the chord as a whole. They include ornaments (trills,
    /// arpeggios), articulations (staccato, accent), and other performance
    /// instructions. Dynamics are stored separately in the `dynamics` field.
    pub articulations: Vec<Articulation>,

    /// The lyric syllable aligned with this chord, if any.
    ///
    /// Lyrics are aligned from `w:` lines in the ABC notation.
    pub lyric: Option<Syllable<'input>>,

    /// Number of slurs starting at this chord.
    pub slur_starts: u8,

    /// Number of slurs ending at this chord.
    pub slur_ends: u8,

    /// Number of dotted slurs starting at this chord.
    ///
    /// Dotted slurs are rendered with a dotted line instead of a solid line.
    pub dotted_slur_starts: u8,

    /// Number of dotted slurs ending at this chord.
    ///
    /// Counts how many dotted slurs conclude at this chord.
    pub dotted_slur_ends: u8,
}

/// A guitar chord symbol event.
///
/// Guitar chord symbols (e.g., "Am7", "G/B") indicate harmonic content and
/// are positioned at specific points in time. They can be used for both
/// display (shown above the staff) and generating accompaniment patterns.
///
/// # Examples
///
/// ```
/// use abc::types::{GuitarChordEvent, Duration};
///
/// let chord = GuitarChordEvent {
///     symbol: "Am7",
///     bass_note: None,
///     absolute_time: Duration::new(0, 1),
/// };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct GuitarChordEvent<'input> {
    /// The chord symbol (e.g., "Am7", "Cmaj7", "D").
    pub symbol: &'input str,

    /// Optional bass note (e.g., "B" in "G/B").
    pub bass_note: Option<&'input str>,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,
}

/// A text annotation event.
///
/// Annotations are text positioned at specific points in time, displayed
/// relative to notes or chords. They are typically used for performance
/// directions, fingerings, or other textual information.
///
/// # Examples
///
/// ```
/// use abc::types::{AnnotationEvent, AnnotationPlacement, Duration};
///
/// let annotation = AnnotationEvent {
///     text: "legato",
///     placement: AnnotationPlacement::Above,
///     absolute_time: Duration::new(0, 1),
/// };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct AnnotationEvent<'input> {
    /// The text content of the annotation.
    pub text: &'input str,

    /// Where to display the annotation relative to the note/chord.
    pub placement: AnnotationPlacement,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,
}

/// A MIDI control change event.
///
/// Control change events modify synthesizer parameters at a specific point in time.
/// They are emitted by `%%MIDI control` directives in ABC notation.
///
/// # Examples
///
/// ```
/// use abc::types::{MidiControlEvent, Duration, Channel};
///
/// // Set volume (controller 7) to 100 on channel 1
/// let control = MidiControlEvent {
///     channel: Channel::new(1).unwrap(),
///     controller: 7,
///     value: 100,
///     absolute_time: Duration::new(0, 1),
/// };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct MidiControlEvent {
    /// The MIDI channel for this control change.
    pub channel: super::Channel,

    /// The controller number (0-127).
    pub controller: u8,

    /// The controller value (0-127).
    pub value: u8,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,
}

/// A MIDI pitch bend event.
///
/// Pitch bend events modify the pitch of all notes on a channel at a specific point in time.
/// They are emitted by `%%MIDI pitchbend` directives in ABC notation.
///
/// # Examples
///
/// ```
/// use abc::types::{MidiPitchBendEvent, Duration, Channel};
///
/// // Bend pitch up on channel 1
/// let pitch_bend = MidiPitchBendEvent {
///     channel: Channel::new(1).unwrap(),
///     value: 2000,
///     absolute_time: Duration::new(0, 1),
/// };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct MidiPitchBendEvent {
    /// The MIDI channel for this pitch bend.
    pub channel: super::Channel,

    /// The pitch bend value (-8192 to +8191).
    pub value: i16,

    /// The absolute time from the start of the tune.
    pub absolute_time: Duration,
}
