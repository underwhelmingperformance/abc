//! Tune structure types.
//!
//! This module defines the core structure of an ABC tune: `Tune`, `Voice`,
//! and `Measure`, along with identifier types like `ReferenceNumber`, `VoiceId`,
//! `MeasureNumber`, and `PartLabel`.

use std::{collections::HashMap, rc::Rc};

use derive_more::Display;

use super::{
    Context, Duration, KeySignature, Meter, TempoMarking,
    event::{Event, Note, TimedEvent},
};

/// A tune reference number from the `X:` field.
///
/// Every tune in an ABC file must have a unique reference number, specified
/// with the `X:` field at the start of the tune header.
///
/// # Examples
///
/// ```
/// # use abc::types::ReferenceNumber;
/// let ref_num = ReferenceNumber(1);
/// ```
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ReferenceNumber(pub u32);

/// A voice identifier.
///
/// Voices represent separate melodic lines or parts within a tune. Each voice
/// is identified by a string specified in the `V:` field.
///
/// # Examples
///
/// ```
/// # use abc::types::VoiceId;
/// let soprano = VoiceId::new("soprano");
/// let bass = VoiceId::new("bass");
/// ```
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VoiceId<'a>(&'a str);

impl<'a> VoiceId<'a> {
    /// Creates a new voice identifier from a string slice.
    #[must_use]
    pub fn new(id: &'a str) -> Self {
        Self(id)
    }

    /// Returns the voice identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0
    }
}

/// A measure (bar) number.
///
/// Measures are numbered sequentially from the start of a tune, beginning at 1.
///
/// # Examples
///
/// ```
/// # use abc::types::MeasureNumber;
/// let first_measure = MeasureNumber(1);
/// ```
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeasureNumber(pub u32);

/// A part label.
///
/// Parts are sections of a tune that can be repeated in various orders,
/// typically labelled with uppercase letters (A, B, C, etc.).
///
/// # Examples
///
/// ```
/// # use abc::types::PartLabel;
/// let part_a = PartLabel('A');
/// let part_b = PartLabel('B');
/// ```
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartLabel(pub char);

/// A complete, validated ABC tune.
///
/// All context-dependent values have been resolved:
/// - Note pitches are absolute MIDI numbers
/// - Durations are computed from unit length and modifiers
/// - Accidentals have been applied
/// - Context changes are tracked
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use abc::{parse, types::*};
///
/// let input = "\
/// X:42
/// T:The Kesh Jig
/// C:Trad.
/// R:Jig
/// O:Ireland
/// K:G
/// ";
/// let doc = parse(input).unwrap();
/// let tune = doc.tunes().first().unwrap();
///
/// let mut expected_voices = HashMap::new();
/// expected_voices.insert(
///     VoiceId::new("1"),
///     Voice {
///         id: VoiceId::new("1"),
///         name: None,
///         measures: Vec::new(),
///     },
/// );
///
/// let expected = Tune {
///     reference_number: ReferenceNumber(42),
///     title: "The Kesh Jig",
///     composer: Some("Trad."),
///     origin: Some("Ireland"),
///     rhythm: Some("Jig"),
///     metadata: TuneMetadata::default(),
///     key: KeySignature {
///         tonic: PitchClass::G,
///         ..Default::default()
///     },
///     meter: Meter { numerator: 4, denominator: 4 },
///     tempo: None,
///     unit_length: Duration::new(1, 8),
///     voices: expected_voices,
///     default_voice: VoiceId::new("1"),
/// };
///
/// assert_eq!(tune, &expected);
/// ```
#[derive(Debug, PartialEq, Default, bon::Builder)]
pub struct Tune<'input> {
    /// The reference number (X: field) uniquely identifying this tune.
    #[builder(default)]
    pub reference_number: ReferenceNumber,

    /// The title of the tune (T: field).
    #[builder(default)]
    pub title: &'input str,

    /// The composer, if specified (C: field).
    pub composer: Option<&'input str>,

    /// The origin or geographical source (O: field).
    pub origin: Option<&'input str>,

    /// The rhythm type (R: field), e.g., "reel", "jig", "waltz".
    pub rhythm: Option<&'input str>,

    /// Additional metadata fields.
    #[builder(default)]
    pub metadata: TuneMetadata<'input>,

    /// The key signature.
    #[builder(default)]
    pub key: KeySignature,

    /// The metre (time signature).
    #[builder(default)]
    pub meter: Meter,

    /// The tempo marking, if specified.
    pub tempo: Option<TempoMarking>,

    /// The unit note length (default duration for notes without explicit length).
    #[builder(default)]
    pub unit_length: Duration,

    /// The voices in this tune.
    ///
    /// Single-voice tunes have one entry. Multi-voice tunes have multiple
    /// entries keyed by voice ID.
    #[builder(default)]
    pub voices: HashMap<VoiceId<'input>, Voice<'input>>,

    /// The default/primary voice ID.
    ///
    /// For single-voice tunes, this is the only voice. For multi-voice tunes,
    /// this is typically the first voice or the melody voice.
    #[builder(default)]
    pub default_voice: VoiceId<'input>,
}

impl<'input> Tune<'input> {
    /// Iterator over all voices in this tune.
    pub fn voices(&self) -> impl Iterator<Item = &Voice<'input>> + '_ {
        self.voices.values()
    }

    /// Get the primary/default voice.
    ///
    /// This is a convenience method for accessing the main voice in the tune.
    ///
    /// # Examples
    ///
    /// ```
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Test\nR:Reel\nK:D\n";
    /// let doc = parse(input).unwrap();
    /// let tune = doc.tunes().first().unwrap();
    /// let voice = tune.primary_voice();
    ///
    /// let expected = Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: Vec::new(),
    /// };
    ///
    /// assert_eq!(voice, &expected);
    /// ```
    pub fn primary_voice(&self) -> &Voice<'input> {
        &self.voices[&self.default_voice]
    }

    /// Iterator over all notes across all voices in chronological order.
    ///
    /// This flattens all voices and measures to provide a simple iterator
    /// over just the notes.
    ///
    /// # Examples
    ///
    /// ```
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Test Tune\nC:Composer\nK:G\n";
    /// let doc = parse(input).unwrap();
    /// let tune = doc.tunes().first().unwrap();
    /// let notes: Vec<&Note> = tune.notes().collect();
    ///
    /// // No notes yet as body transformation not implemented
    /// assert_eq!(notes, Vec::<&Note>::new());
    /// ```
    pub fn notes(&self) -> impl Iterator<Item = &Note<'_>> + '_ {
        self.voices()
            .flat_map(|v| v.measures())
            .flat_map(|m| m.events())
            .filter_map(|e| match e {
                Event::Timed(TimedEvent::Note(n)) => Some(n),
                _ => None,
            })
    }

    /// Iterator over all events across all voices.
    ///
    /// Events include notes, rests, and chords.
    ///
    /// # Examples
    ///
    /// ```
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Test\nR:Jig\nK:D\n";
    /// let doc = parse(input).unwrap();
    /// let tune = doc.tunes().first().unwrap();
    /// let events: Vec<&Event> = tune.events().collect();
    ///
    /// // No events yet as body transformation not implemented
    /// assert_eq!(events, Vec::<&Event>::new());
    /// ```
    pub fn events(&self) -> impl Iterator<Item = &Event<'_>> + '_ {
        self.voices()
            .flat_map(|v| v.measures())
            .flat_map(|m| m.events())
    }
}

/// Additional metadata for a tune.
///
/// Contains optional metadata fields that provide information about the tune.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TuneMetadata<'input> {
    /// Area or region (A: field).
    pub area: Option<&'input str>,

    /// Book or collection reference (B: field).
    pub book: Option<&'input str>,

    /// Discography reference (D: field).
    pub discography: Option<&'input str>,

    /// File URL or path (F: field).
    pub file_url: Option<&'input str>,

    /// Group, band, or ensemble (G: field).
    pub group: Option<&'input str>,

    /// History or background (H: field).
    pub history: Option<&'input str>,

    /// Notes or comments (N: field).
    pub notes: Option<&'input str>,

    /// Source of transcription (S: field).
    pub source: Option<&'input str>,

    /// Transcription information (Z: field).
    pub transcription: Option<&'input str>,
}

/// A voice within a tune.
///
/// Voices allow multiple melodic lines to be represented in a single tune.
/// Single-voice tunes have one voice, while polyphonic music has multiple.
///
/// # Examples
///
/// ```
/// use abc::{parse, types::*};
///
/// let input = "X:1\nT:Test\nC:Trad.\nK:G\n";
/// let doc = parse(input).unwrap();
/// let tune = doc.tunes().first().unwrap();
/// let voice = tune.voices().next().unwrap();
///
/// let expected = Voice {
///     id: VoiceId::new("1"),
///     name: None,
///     measures: Vec::new(),
/// };
///
/// assert_eq!(voice, &expected);
/// ```
#[derive(Debug, PartialEq, Default, bon::Builder)]
pub struct Voice<'input> {
    /// The voice identifier.
    #[builder(default)]
    pub id: VoiceId<'input>,

    /// The voice name, if specified.
    pub name: Option<&'input str>,

    /// The measures in this voice.
    #[builder(default)]
    pub measures: Vec<Measure<'input>>,
}

impl<'input> Voice<'input> {
    /// Iterator over the measures in this voice.
    ///
    /// # Examples
    ///
    /// ```
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Test\nK:C\n";
    /// let doc = parse(input).unwrap();
    /// let tune = doc.tunes().first().unwrap();
    /// let voice = tune.primary_voice();
    ///
    /// // Empty body means no measures
    /// let expected_voice = Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: vec![],
    /// };
    ///
    /// assert_eq!(voice, &expected_voice);
    /// ```
    pub fn measures(&self) -> impl Iterator<Item = &Measure<'input>> {
        self.measures.iter()
    }
}

/// A measure of music.
///
/// A measure contains a sequence of musical events (notes, rests, chords) and
/// tracks the musical context (key, metre, tempo) in effect.
#[derive(Debug, PartialEq)]
pub struct Measure<'input> {
    /// The measure number (1-indexed).
    pub number: MeasureNumber,

    /// The events in this measure.
    pub events: Vec<Event<'input>>,

    /// The musical context in effect at the start of this measure.
    ///
    /// Context is shared via `Rc` when it doesn't change between measures,
    /// avoiding duplication.
    pub context: Rc<Context<'input>>,

    /// The part marker for this measure, if any.
    ///
    /// Part markers (P:A, P:B, etc.) identify structural sections of the tune.
    pub part: Option<PartLabel>,

    /// The variant ending(s) this measure belongs to, if any.
    ///
    /// When `None`, this measure is played on every iteration of a repeat.
    /// When `Some(variants)`, this measure is only played during the specified
    /// iterations. For example, `Some(vec![1])` means first time only,
    /// `Some(vec![2])` means second time only, and `Some(vec![1, 3])` means
    /// first and third times.
    ///
    /// # Examples
    ///
    /// ```text
    /// |: ABC |1 DEF :|2 GHI ||
    /// ```
    ///
    /// - ABC measure: `variant: None` (played every time)
    /// - DEF measure: `variant: Some(vec![1])` (first time only)
    /// - GHI measure: `variant: Some(vec![2])` (second time only)
    pub variant: Option<Vec<u8>>,
}

impl<'input> Measure<'input> {
    /// Iterator over the events in this measure.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Test\nK:C\nC|\n";
    /// let doc = parse(input).unwrap();
    /// let tune = doc.tunes().first().unwrap();
    /// let voice = tune.primary_voice();
    /// let measure = voice.measures().next().unwrap();
    ///
    /// // Build expected context using bon builder
    /// let expected_context = Context::builder()
    ///     .key(KeySignature {
    ///         tonic: PitchClass::C,
    ///         ..Default::default()
    ///     })
    ///     .meter(Meter { numerator: 4, denominator: 4 })
    ///     .unit_length(Duration::new(1, 8))
    ///     .build()
    ///     .into_rc();
    ///
    /// let expected_measure = Measure {
    ///     number: MeasureNumber(1),
    ///     events: vec![
    ///         Note {
    ///             pitch: Pitch::new(PitchClass::C, Octave(4), None),
    ///             duration: Duration::new(1, 8),
    ///             dynamics: Dynamics::default(),
    ///             dynamic_direction: DynamicDirection::None,
    ///             absolute_time: Duration::new(0, 1),
    ///             ..Default::default()
    ///         }.into()
    ///     ],
    ///     context: expected_context,
    ///     part: None,
    ///     variant: None,
    /// };
    ///
    /// assert_eq!(measure, &expected_measure);
    /// ```
    pub fn events(&self) -> impl Iterator<Item = &Event<'input>> {
        self.events.iter()
    }
}
