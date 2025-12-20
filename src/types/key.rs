//! Key signature and clef representation in the AST.
//!
//! This module defines types for representing key signatures, modes, clefs,
//! and transposition information as they appear in ABC notation.

use super::{Accidental, PitchClass};

/// A key signature specifying the tonic, mode, and additional notation settings.
///
/// The key signature determines which notes are sharp or flat by default and
/// establishes the tonal centre of the music. In ABC notation, key signatures
/// are specified with the `K:` field.
///
/// # Examples
///
/// ```text
/// K:C       - C major (no sharps or flats)
/// K:G       - G major (F♯)
/// K:Am      - A minor (natural minor)
/// K:D dor   - D dorian mode
/// K:HP      - Highland bagpipe key
/// K:D middle=d stafflines=5  - D major with rendering info
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, bon::Builder)]
pub struct KeySignature {
    /// The tonic (root note) of the key.
    #[builder(default)]
    pub tonic: PitchClass,
    /// An accidental applied to the tonic, if any (e.g., the ♭ in B♭ major).
    pub accidental: Option<Accidental>,
    /// The mode of the key (major, minor, dorian, etc.).
    #[builder(default)]
    pub mode: Mode,
    /// Explicit accidentals shown in the key signature beyond those implied by the key.
    ///
    /// Used for advisory or non-standard key signatures.
    #[builder(default)]
    pub explicit_accidentals: Vec<(PitchClass, Accidental)>,
    /// The clef to use for notation, if specified.
    pub clef: Option<Clef>,
    /// Transposition in semitones, if specified.
    pub transpose: Option<i8>,
    /// Octave shift, if specified.
    pub octave_shift: Option<i8>,
    /// Which pitch is on the middle line of the staff (rendering info).
    ///
    /// This is used for custom staff configurations and doesn't affect pitch resolution.
    /// Example: `middle=d` in `K:D middle=d`
    pub middle: Option<PitchClass>,
    /// Custom number of staff lines (rendering info).
    ///
    /// Typically 5, but can be customized (usually 1-6).
    /// This is rendering information and doesn't affect musical interpretation.
    /// Example: `stafflines=4` in `K:G stafflines=4`
    pub stafflines: Option<u8>,
}

/// A musical mode.
///
/// Modes determine the pattern of whole and half steps in a scale, giving
/// each mode its characteristic sound. ABC notation supports all seven
/// diatonic modes plus special cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Ionian mode (major scale). This is the default if no mode is specified.
    #[default]
    Major,
    /// Aeolian mode (natural minor scale).
    Minor,
    /// Ionian mode (same as Major, but explicitly named).
    Ionian,
    /// Dorian mode (minor scale with raised 6th).
    Dorian,
    /// Phrygian mode (minor scale with lowered 2nd).
    Phrygian,
    /// Lydian mode (major scale with raised 4th).
    Lydian,
    /// Mixolydian mode (major scale with lowered 7th).
    Mixolydian,
    /// Locrian mode (diminished scale).
    Locrian,
    /// Highland Bagpipe key (A mixolydian with F# and C#, G natural).
    ///
    /// This is the standard bagpipe key. The uppercase `HP` variant enables
    /// bagpipe-style grace notes and ornaments.
    HighlandBagpipe,
    /// Highland Bagpipe key (alternative form).
    ///
    /// Same accidentals as `HP` but with standard grace note interpretation.
    /// Written as `Hp` in ABC notation.
    HighlandBagpipeLower,
    /// No key signature (all notes natural).
    ///
    /// Written as `K:none` in ABC notation. Useful for atonal music or
    /// when all accidentals are written explicitly.
    None,
}

impl KeySignature {
    /// Get the implied accidentals from the key signature.
    ///
    /// This computes the accidentals implied by the tonic and mode, based on
    /// the circle of fifths. For special modes like Highland Bagpipe, it returns
    /// the standard accidentals for that mode.
    ///
    /// # Returns
    ///
    /// A vector of (PitchClass, Accidental) pairs representing the accidentals
    /// implied by this key signature. Returns an empty vector for `K:none`.
    #[must_use]
    pub fn implied_accidentals(&self) -> Vec<(PitchClass, Accidental)> {
        match self.mode {
            Mode::None => Vec::new(),
            Mode::HighlandBagpipe | Mode::HighlandBagpipeLower => {
                // HP/Hp: A mixolydian with F# and C# (G natural)
                vec![
                    (PitchClass::F, Accidental::Sharp),
                    (PitchClass::C, Accidental::Sharp),
                ]
            }
            _ => self.compute_diatonic_accidentals(),
        }
    }

    /// Compute the accidentals for a standard diatonic key.
    ///
    /// Uses the circle of fifths to determine which notes are sharp or flat
    /// based on the tonic and mode.
    fn compute_diatonic_accidentals(&self) -> Vec<(PitchClass, Accidental)> {
        // The number of sharps/flats is determined by the position on the circle of fifths
        // relative to the mode's natural position.
        //
        // Mode offsets (relative to major):
        // - Major/Ionian: 0
        // - Mixolydian: -1 (one more flat / one fewer sharp)
        // - Dorian: -2
        // - Minor/Aeolian: -3
        // - Phrygian: -4
        // - Locrian: -5
        // - Lydian: +1 (one more sharp)

        let mode_offset: i8 = match self.mode {
            Mode::Major | Mode::Ionian => 0,
            Mode::Mixolydian => -1,
            Mode::Dorian => -2,
            Mode::Minor => -3,
            Mode::Phrygian => -4,
            Mode::Locrian => -5,
            Mode::Lydian => 1,
            Mode::None | Mode::HighlandBagpipe | Mode::HighlandBagpipeLower => {
                return Vec::new();
            }
        };

        // Base position on circle of fifths for major keys (0 = C, positive = sharps)
        let base_position: i8 = match (self.tonic, self.accidental) {
            (PitchClass::C, None) => 0,
            (PitchClass::G, None) => 1,
            (PitchClass::D, None) => 2,
            (PitchClass::A, None) => 3,
            (PitchClass::E, None) => 4,
            (PitchClass::B, None) => 5,
            (PitchClass::F, Some(Accidental::Sharp)) => 6,
            (PitchClass::C, Some(Accidental::Sharp)) => 7,
            (PitchClass::F, None) => -1,
            (PitchClass::B, Some(Accidental::Flat)) => -2,
            (PitchClass::E, Some(Accidental::Flat)) => -3,
            (PitchClass::A, Some(Accidental::Flat)) => -4,
            (PitchClass::D, Some(Accidental::Flat)) => -5,
            (PitchClass::G, Some(Accidental::Flat)) => -6,
            (PitchClass::C, Some(Accidental::Flat)) => -7,
            // Handle enharmonic equivalents and edge cases
            (PitchClass::G, Some(Accidental::Sharp)) => 8,  // Same as Ab
            (PitchClass::D, Some(Accidental::Sharp)) => 9,  // Same as Eb
            (PitchClass::A, Some(Accidental::Sharp)) => 10, // Same as Bb
            (PitchClass::E, Some(Accidental::Sharp)) => 11, // Same as F
            (PitchClass::F, Some(Accidental::Flat)) => -8,  // Same as E
            _ => 0, // Default to C major for unusual cases
        };

        let position = base_position + mode_offset;

        // Order of sharps: F C G D A E B
        const SHARP_ORDER: [PitchClass; 7] = [
            PitchClass::F,
            PitchClass::C,
            PitchClass::G,
            PitchClass::D,
            PitchClass::A,
            PitchClass::E,
            PitchClass::B,
        ];

        // Order of flats: B E A D G C F
        const FLAT_ORDER: [PitchClass; 7] = [
            PitchClass::B,
            PitchClass::E,
            PitchClass::A,
            PitchClass::D,
            PitchClass::G,
            PitchClass::C,
            PitchClass::F,
        ];

        if position > 0 {
            let count = position.min(7) as usize;
            SHARP_ORDER[..count]
                .iter()
                .map(|&pc| (pc, Accidental::Sharp))
                .collect()
        } else if position < 0 {
            let count = (-position).min(7) as usize;
            FLAT_ORDER[..count]
                .iter()
                .map(|&pc| (pc, Accidental::Flat))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the effective accidental for a pitch class in this key signature.
    ///
    /// This checks explicit accidentals first, then falls back to implied accidentals.
    /// Returns `None` if the pitch is natural in this key.
    #[must_use]
    pub fn get_accidental(&self, pitch: PitchClass) -> Option<Accidental> {
        // Check explicit accidentals first (they override implied)
        if let Some(&(_, acc)) = self.explicit_accidentals.iter().find(|(pc, _)| *pc == pitch) {
            // A natural accidental explicitly cancels the implied accidental
            if acc == Accidental::Natural {
                return None;
            }
            return Some(acc);
        }

        // Check implied accidentals
        self.implied_accidentals()
            .iter()
            .find(|(pc, _)| *pc == pitch)
            .map(|(_, acc)| *acc)
    }
}

/// A musical clef.
///
/// The clef determines which pitch corresponds to which line or space on
/// the staff. Different clefs are used for different pitch ranges and
/// instruments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Clef {
    /// Treble clef (G clef), with G4 on the second line.
    Treble,
    /// Bass clef (F clef), with F3 on the fourth line.
    Bass,
    /// Alto clef (C clef), with C4 on the middle line.
    Alto,
    /// Tenor clef (C clef), with C4 on the fourth line.
    Tenor,
    /// Percussion clef (no specific pitch).
    Perc,
    /// Treble clef transposed down one octave (sounds 8va bassa).
    TrebleMinus8,
    /// Treble clef transposed up one octave (sounds 8va alta).
    TreblePlus8,
    /// Bass clef transposed up one octave.
    BassPlus8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Helper to create a key signature for testing.
    fn make_key(
        tonic: PitchClass,
        accidental: Option<Accidental>,
        mode: Mode,
    ) -> KeySignature {
        KeySignature {
            tonic,
            accidental,
            mode,
            explicit_accidentals: Vec::new(),
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        }
    }

    #[rstest]
    // Major keys - sharp side of circle of fifths
    #[case(PitchClass::C, None, Mode::Major, &[])]
    #[case(PitchClass::G, None, Mode::Major, &[(PitchClass::F, Accidental::Sharp)])]
    #[case(PitchClass::D, None, Mode::Major, &[(PitchClass::F, Accidental::Sharp), (PitchClass::C, Accidental::Sharp)])]
    #[case(PitchClass::A, None, Mode::Major, &[(PitchClass::F, Accidental::Sharp), (PitchClass::C, Accidental::Sharp), (PitchClass::G, Accidental::Sharp)])]
    // Major keys - flat side of circle of fifths
    #[case(PitchClass::F, None, Mode::Major, &[(PitchClass::B, Accidental::Flat)])]
    #[case(PitchClass::B, Some(Accidental::Flat), Mode::Major, &[(PitchClass::B, Accidental::Flat), (PitchClass::E, Accidental::Flat)])]
    #[case(PitchClass::E, Some(Accidental::Flat), Mode::Major, &[(PitchClass::B, Accidental::Flat), (PitchClass::E, Accidental::Flat), (PitchClass::A, Accidental::Flat)])]
    // Minor keys (relative to major)
    #[case(PitchClass::A, None, Mode::Minor, &[])] // A minor = C major
    #[case(PitchClass::E, None, Mode::Minor, &[(PitchClass::F, Accidental::Sharp)])] // E minor = G major
    #[case(PitchClass::D, None, Mode::Minor, &[(PitchClass::B, Accidental::Flat)])] // D minor = F major
    // Modal keys
    #[case(PitchClass::D, None, Mode::Dorian, &[])] // D dorian = C major
    #[case(PitchClass::G, None, Mode::Mixolydian, &[])] // G mixolydian = C major
    #[case(PitchClass::F, None, Mode::Lydian, &[])] // F lydian = C major
    #[case(PitchClass::E, None, Mode::Phrygian, &[])] // E phrygian = C major
    // Special keys
    #[case(PitchClass::A, None, Mode::HighlandBagpipe, &[(PitchClass::F, Accidental::Sharp), (PitchClass::C, Accidental::Sharp)])]
    #[case(PitchClass::A, None, Mode::HighlandBagpipeLower, &[(PitchClass::F, Accidental::Sharp), (PitchClass::C, Accidental::Sharp)])]
    #[case(PitchClass::C, None, Mode::None, &[])]
    fn test_implied_accidentals(
        #[case] tonic: PitchClass,
        #[case] accidental: Option<Accidental>,
        #[case] mode: Mode,
        #[case] expected: &[(PitchClass, Accidental)],
    ) {
        let key = make_key(tonic, accidental, mode);
        assert_eq!(key.implied_accidentals(), expected.to_vec());
    }

    #[test]
    fn test_explicit_accidental_overrides_implied() {
        // D major has F# and C#, but we explicitly make C natural
        let key = KeySignature {
            tonic: PitchClass::D,
            accidental: None,
            mode: Mode::Major,
            explicit_accidentals: vec![(PitchClass::C, Accidental::Natural)],
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };
        // F should still be sharp
        assert_eq!(key.get_accidental(PitchClass::F), Some(Accidental::Sharp));
        // C should be natural (None means natural)
        assert_eq!(key.get_accidental(PitchClass::C), None);
    }

    #[test]
    fn test_explicit_accidental_adds_to_key() {
        // A minor with explicit F# and G# (harmonic minor)
        let key = KeySignature {
            tonic: PitchClass::A,
            accidental: None,
            mode: Mode::Minor,
            explicit_accidentals: vec![
                (PitchClass::G, Accidental::Sharp),
            ],
            clef: None,
            transpose: None,
            octave_shift: None,
            middle: None,
            stafflines: None,
        };
        // A minor has no implied accidentals, but we explicitly add G#
        assert_eq!(key.get_accidental(PitchClass::G), Some(Accidental::Sharp));
        // Other notes remain natural
        assert_eq!(key.get_accidental(PitchClass::F), None);
    }
}
