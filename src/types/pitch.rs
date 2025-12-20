//! Pitch and note-related types.
//!
//! This module provides types for representing musical pitch: pitch classes (C-B),
//! accidentals, octave offsets, and the primary `Pitch` type that preserves musical spelling.
//!
//! For MIDI-specific pitch representation, see [`midi::MidiNote`](super::midi::MidiNote)
//! and [`midi::MidiPitch`](super::midi::MidiPitch).

use super::midi::MidiNote;

/// A pitch class in Western music notation.
///
/// Represents the seven natural notes (C, D, E, F, G, A, B) without regard to
/// octave or accidentals. Pitch classes form the basis of musical keys and scales.
///
/// # Examples
///
/// ```
/// # use abc::types::PitchClass;
/// let c = PitchClass::C;
/// let d = PitchClass::D;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PitchClass {
    #[default]
    /// C natural
    C,
    /// D natural
    D,
    /// E natural
    E,
    /// F natural
    F,
    /// G natural
    G,
    /// A natural
    A,
    /// B natural
    B,
}

impl TryFrom<char> for PitchClass {
    type Error = ();

    /// Convert a character to its pitch class.
    ///
    /// Accepts A-G and a-g (case insensitive).
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_uppercase() {
            'C' => Ok(PitchClass::C),
            'D' => Ok(PitchClass::D),
            'E' => Ok(PitchClass::E),
            'F' => Ok(PitchClass::F),
            'G' => Ok(PitchClass::G),
            'A' => Ok(PitchClass::A),
            'B' => Ok(PitchClass::B),
            _ => Err(()),
        }
    }
}

/// An accidental modifier for a note.
///
/// Accidentals in ABC notation apply only to the note they precede and
/// last through the measure unless barlines are disabled.
///
/// # Examples
///
/// ```
/// # use abc::types::Accidental;
/// let sharp = Accidental::Sharp;
/// let flat = Accidental::Flat;
/// let natural = Accidental::Natural;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    /// `__` - Double flat (lowers pitch by two semitones)
    DoubleFlat,
    /// `_` - Flat (lowers pitch by one semitone)
    Flat,
    /// `=` - Natural (cancels key signature or previous accidental)
    Natural,
    /// `^` - Sharp (raises pitch by one semitone)
    Sharp,
    /// `^^` - Double sharp (raises pitch by two semitones)
    DoubleSharp,
    /// Three-quarter sharp (microtonal, raises pitch by three quarter-tones)
    ThreeQuarterSharp,
    /// Quarter sharp (microtonal, raises pitch by one quarter-tone)
    QuarterSharp,
    /// Quarter flat (microtonal, lowers pitch by one quarter-tone)
    QuarterFlat,
    /// Three-quarter flat (microtonal, lowers pitch by three quarter-tones)
    ThreeQuarterFlat,
}

/// An octave offset relative to the middle octave.
///
/// In ABC notation, uppercase letters (`C D E F G A B`) represent notes in
/// the octave around middle C, whilst lowercase letters (`c d e f g a b`)
/// represent the octave above. Octave modifiers (`,` and `'`) further adjust
/// the pitch up or down by octaves.
///
/// This type represents the net octave offset from the middle octave, where
/// 0 represents the middle octave, positive values represent higher octaves,
/// and negative values represent lower octaves.
///
/// # Examples
///
/// ```
/// # use abc::types::Octave;
/// let middle = Octave(0);
/// let higher = Octave(1);
/// let lower = Octave(-1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Octave(pub i8);

/// A musical pitch with pitch class, octave, and accidental.
///
/// This type preserves the musical spelling of a pitch. C# and Db are distinct
/// even though they're enharmonically equivalent (the same pitch on a piano).
///
/// This representation is useful for:
/// - Displaying notation correctly (showing the intended spelling)
/// - Key signature analysis
/// - Transposition that respects musical spelling
///
/// For MIDI-based pitch representation with microtonal support, see
/// [`midi::MidiPitch`](super::midi::MidiPitch).
///
/// # Examples
///
/// ```
/// # use abc::types::{Pitch, PitchClass, Octave, Accidental};
/// // Middle C
/// let c4 = Pitch {
///     pitch_class: PitchClass::C,
///     octave: Octave(4),
///     accidental: None,
/// };
///
/// // F# in octave 5
/// let f_sharp_5 = Pitch {
///     pitch_class: PitchClass::F,
///     octave: Octave(5),
///     accidental: Some(Accidental::Sharp),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pitch {
    /// The pitch class (C, D, E, F, G, A, or B).
    pub pitch_class: PitchClass,
    /// The octave number (4 is the middle octave containing middle C).
    pub octave: Octave,
    /// The accidental modifier, if any.
    pub accidental: Option<Accidental>,
}

impl Pitch {
    /// Creates a new pitch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::{Pitch, PitchClass, Octave, Accidental};
    /// let middle_c = Pitch::new(PitchClass::C, Octave(4), None);
    /// let f_sharp = Pitch::new(PitchClass::F, Octave(4), Some(Accidental::Sharp));
    /// ```
    #[must_use]
    pub const fn new(
        pitch_class: PitchClass,
        octave: Octave,
        accidental: Option<Accidental>,
    ) -> Self {
        Self {
            pitch_class,
            octave,
            accidental,
        }
    }

    /// Returns this pitch transposed by the given number of semitones.
    ///
    /// Note: This performs a simple transposition that may change the
    /// enharmonic spelling. For musically-aware transposition that
    /// respects key signatures, use a dedicated transposition function.
    ///
    /// Returns `None` if the transposition would result in an invalid pitch.
    #[must_use]
    pub fn transpose_semitones(self, semitones: i8) -> Option<Self> {
        let midi: MidiNote = self.try_into().ok()?;
        let transposed = midi.transpose(semitones)?;
        Some(transposed.into())
    }

    /// Returns the semitone offset for this pitch's accidental.
    fn accidental_semitones(self) -> i16 {
        match self.accidental {
            None => 0,
            Some(Accidental::DoubleFlat) => -2,
            Some(Accidental::Flat) => -1,
            Some(Accidental::Natural) => 0,
            Some(Accidental::Sharp) => 1,
            Some(Accidental::DoubleSharp) => 2,
            Some(Accidental::ThreeQuarterSharp) => 2, // Approximate
            Some(Accidental::QuarterSharp) => 0,      // Approximate
            Some(Accidental::QuarterFlat) => 0,       // Approximate
            Some(Accidental::ThreeQuarterFlat) => -1, // Approximate
        }
    }

    /// Returns the base semitone for this pitch's pitch class within an octave.
    fn pitch_class_semitones(self) -> i16 {
        match self.pitch_class {
            PitchClass::C => 0,
            PitchClass::D => 2,
            PitchClass::E => 4,
            PitchClass::F => 5,
            PitchClass::G => 7,
            PitchClass::A => 9,
            PitchClass::B => 11,
        }
    }
}

impl TryFrom<Pitch> for MidiNote {
    type Error = ();

    /// Converts a pitch to a MIDI note number.
    ///
    /// Returns `Err(())` if the resulting MIDI note would be out of range (0-127).
    fn try_from(pitch: Pitch) -> Result<Self, Self::Error> {
        // MIDI note = 12 * (octave + 1) + pitch_class_offset + accidental
        // C4 = 12 * 5 + 0 = 60
        let midi = 12 * (pitch.octave.0 as i16 + 1)
            + pitch.pitch_class_semitones()
            + pitch.accidental_semitones();

        MidiNote::new(midi as u8).ok_or(())
    }
}

impl From<MidiNote> for Pitch {
    /// Creates a pitch from a MIDI note number.
    ///
    /// This uses a default spelling (preferring sharps over flats).
    /// For musically-aware spelling based on key signature, use a
    /// dedicated conversion function.
    fn from(midi: MidiNote) -> Self {
        let note = midi.get();
        let octave = (note / 12) as i8 - 1;
        let pitch_in_octave = note % 12;

        let (pitch_class, accidental) = match pitch_in_octave {
            0 => (PitchClass::C, None),
            1 => (PitchClass::C, Some(Accidental::Sharp)),
            2 => (PitchClass::D, None),
            3 => (PitchClass::D, Some(Accidental::Sharp)),
            4 => (PitchClass::E, None),
            5 => (PitchClass::F, None),
            6 => (PitchClass::F, Some(Accidental::Sharp)),
            7 => (PitchClass::G, None),
            8 => (PitchClass::G, Some(Accidental::Sharp)),
            9 => (PitchClass::A, None),
            10 => (PitchClass::A, Some(Accidental::Sharp)),
            11 => (PitchClass::B, None),
            _ => unreachable!(),
        };

        Self {
            pitch_class,
            octave: Octave(octave),
            accidental,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[test]
    fn pitch_new_middle_c() {
        let pitch = Pitch::new(PitchClass::C, Octave(4), None);
        assert_eq!(pitch.pitch_class, PitchClass::C);
        assert_eq!(pitch.octave, Octave(4));
        assert_eq!(pitch.accidental, None);
    }

    #[rstest]
    #[case(PitchClass::C, Octave(4), None)]
    #[case(PitchClass::D, Octave(4), None)]
    #[case(PitchClass::E, Octave(4), None)]
    #[case(PitchClass::F, Octave(4), None)]
    #[case(PitchClass::G, Octave(4), None)]
    #[case(PitchClass::A, Octave(4), None)]
    #[case(PitchClass::B, Octave(4), None)]
    fn pitch_new_all_pitch_classes(
        #[case] pitch_class: PitchClass,
        #[case] octave: Octave,
        #[case] accidental: Option<Accidental>,
    ) {
        let pitch = Pitch::new(pitch_class, octave, accidental);
        assert_eq!(pitch.pitch_class, pitch_class);
        assert_eq!(pitch.octave, octave);
        assert_eq!(pitch.accidental, accidental);
    }

    #[rstest]
    #[case(Octave(0))]
    #[case(Octave(1))]
    #[case(Octave(4))]
    #[case(Octave(7))]
    #[case(Octave(-1))]
    fn pitch_new_various_octaves(#[case] octave: Octave) {
        let pitch = Pitch::new(PitchClass::C, octave, None);
        assert_eq!(pitch.octave, octave);
    }

    #[rstest]
    #[case(Some(Accidental::Sharp))]
    #[case(Some(Accidental::Flat))]
    #[case(Some(Accidental::Natural))]
    #[case(Some(Accidental::DoubleSharp))]
    #[case(Some(Accidental::DoubleFlat))]
    #[case(None)]
    fn pitch_new_various_accidentals(#[case] accidental: Option<Accidental>) {
        let pitch = Pitch::new(PitchClass::F, Octave(4), accidental);
        assert_eq!(pitch.accidental, accidental);
    }

    #[test]
    fn pitch_to_midi_middle_c() {
        let pitch = Pitch::new(PitchClass::C, Octave(4), None);
        let midi: MidiNote = pitch.try_into().unwrap();
        assert_eq!(midi.get(), 60); // Middle C = 60
    }

    #[rstest]
    #[case(PitchClass::C, Octave(4), None, 60)] // C4 = 60
    #[case(PitchClass::D, Octave(4), None, 62)] // D4 = 62
    #[case(PitchClass::E, Octave(4), None, 64)] // E4 = 64
    #[case(PitchClass::F, Octave(4), None, 65)] // F4 = 65
    #[case(PitchClass::G, Octave(4), None, 67)] // G4 = 67
    #[case(PitchClass::A, Octave(4), None, 69)] // A4 = 69
    #[case(PitchClass::B, Octave(4), None, 71)] // B4 = 71
    #[case(PitchClass::C, Octave(5), None, 72)] // C5 = 72
    fn pitch_to_midi_natural_notes(
        #[case] pitch_class: PitchClass,
        #[case] octave: Octave,
        #[case] accidental: Option<Accidental>,
        #[case] expected_midi: u8,
    ) {
        let pitch = Pitch::new(pitch_class, octave, accidental);
        let midi: MidiNote = pitch.try_into().unwrap();
        assert_eq!(midi.get(), expected_midi);
    }

    #[rstest]
    #[case(PitchClass::C, Octave(4), Some(Accidental::Sharp), 61)] // C#4 = 61
    #[case(PitchClass::F, Octave(4), Some(Accidental::Sharp), 66)] // F#4 = 66
    #[case(PitchClass::B, Octave(4), Some(Accidental::Flat), 70)] // Bb4 = 70
    #[case(PitchClass::E, Octave(4), Some(Accidental::Flat), 63)] // Eb4 = 63
    #[case(PitchClass::F, Octave(4), Some(Accidental::DoubleSharp), 67)] // F##4 = 67 (same as G)
    #[case(PitchClass::G, Octave(4), Some(Accidental::DoubleFlat), 65)] // Gbb4 = 65 (same as F)
    fn pitch_to_midi_with_accidentals(
        #[case] pitch_class: PitchClass,
        #[case] octave: Octave,
        #[case] accidental: Option<Accidental>,
        #[case] expected_midi: u8,
    ) {
        let pitch = Pitch::new(pitch_class, octave, accidental);
        let midi: MidiNote = pitch.try_into().unwrap();
        assert_eq!(midi.get(), expected_midi);
    }

    #[test]
    fn pitch_to_midi_octave_boundaries() {
        // C0 = 12
        let c0 = Pitch::new(PitchClass::C, Octave(0), None);
        let midi: MidiNote = c0.try_into().unwrap();
        assert_eq!(midi.get(), 12);

        // C-1 = 0 (lowest MIDI note)
        let c_minus1 = Pitch::new(PitchClass::C, Octave(-1), None);
        let midi: MidiNote = c_minus1.try_into().unwrap();
        assert_eq!(midi.get(), 0);
    }

    #[test]
    fn midi_to_pitch_middle_c() {
        let midi = MidiNote::new(60).unwrap();
        let pitch: Pitch = midi.into();
        assert_eq!(pitch.pitch_class, PitchClass::C);
        assert_eq!(pitch.octave, Octave(4));
        assert_eq!(pitch.accidental, None);
    }

    #[rstest]
    #[case(60, PitchClass::C, Octave(4), None)] // C4
    #[case(61, PitchClass::C, Octave(4), Some(Accidental::Sharp))] // C#4
    #[case(62, PitchClass::D, Octave(4), None)] // D4
    #[case(63, PitchClass::D, Octave(4), Some(Accidental::Sharp))] // D#4
    #[case(64, PitchClass::E, Octave(4), None)] // E4
    #[case(65, PitchClass::F, Octave(4), None)] // F4
    #[case(66, PitchClass::F, Octave(4), Some(Accidental::Sharp))] // F#4
    #[case(67, PitchClass::G, Octave(4), None)] // G4
    #[case(68, PitchClass::G, Octave(4), Some(Accidental::Sharp))] // G#4
    #[case(69, PitchClass::A, Octave(4), None)] // A4
    #[case(70, PitchClass::A, Octave(4), Some(Accidental::Sharp))] // A#4
    #[case(71, PitchClass::B, Octave(4), None)] // B4
    fn midi_to_pitch_chromatic_scale(
        #[case] midi_num: u8,
        #[case] expected_class: PitchClass,
        #[case] expected_octave: Octave,
        #[case] expected_accidental: Option<Accidental>,
    ) {
        let midi = MidiNote::new(midi_num).unwrap();
        let pitch: Pitch = midi.into();
        assert_eq!(pitch.pitch_class, expected_class);
        assert_eq!(pitch.octave, expected_octave);
        assert_eq!(pitch.accidental, expected_accidental);
    }

    #[test]
    fn pitch_transpose_up_semitone() {
        let c4 = Pitch::new(PitchClass::C, Octave(4), None);
        let c_sharp = c4.transpose_semitones(1).unwrap();
        // After transposition, spelling may change (C -> C#)
        let midi: MidiNote = c_sharp.try_into().unwrap();
        assert_eq!(midi.get(), 61);
    }

    #[test]
    fn pitch_transpose_down_semitone() {
        let c4 = Pitch::new(PitchClass::C, Octave(4), None);
        let b3 = c4.transpose_semitones(-1).unwrap();
        let midi: MidiNote = b3.try_into().unwrap();
        assert_eq!(midi.get(), 59);
    }

    #[test]
    fn pitch_transpose_octave() {
        let c4 = Pitch::new(PitchClass::C, Octave(4), None);
        let c5 = c4.transpose_semitones(12).unwrap();
        let midi: MidiNote = c5.try_into().unwrap();
        assert_eq!(midi.get(), 72);
    }

    #[test]
    fn enharmonic_equivalence_c_sharp_d_flat() {
        let c_sharp = Pitch::new(PitchClass::C, Octave(4), Some(Accidental::Sharp));
        let d_flat = Pitch::new(PitchClass::D, Octave(4), Some(Accidental::Flat));

        // Different spellings
        assert_ne!(c_sharp.pitch_class, d_flat.pitch_class);
        assert_ne!(c_sharp, d_flat);

        // Same MIDI note
        let midi_c_sharp: MidiNote = c_sharp.try_into().unwrap();
        let midi_d_flat: MidiNote = d_flat.try_into().unwrap();
        assert_eq!(midi_c_sharp, midi_d_flat);
        assert_eq!(midi_c_sharp.get(), 61);
    }

    #[test]
    fn pitch_class_ordering() {
        let mut pitch_classes = vec![
            PitchClass::G,
            PitchClass::B,
            PitchClass::D,
            PitchClass::A,
            PitchClass::C,
            PitchClass::F,
            PitchClass::E,
        ];
        pitch_classes.sort();

        assert_eq!(
            pitch_classes,
            vec![
                PitchClass::C,
                PitchClass::D,
                PitchClass::E,
                PitchClass::F,
                PitchClass::G,
                PitchClass::A,
                PitchClass::B,
            ]
        );
    }

    #[test]
    fn octave_ordering() {
        let mut octaves = vec![Octave(5), Octave(-1), Octave(3), Octave(0), Octave(4)];
        octaves.sort();

        assert_eq!(
            octaves,
            vec![Octave(-1), Octave(0), Octave(3), Octave(4), Octave(5)]
        );
    }
}
