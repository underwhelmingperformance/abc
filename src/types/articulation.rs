//! Articulation and ornament types.
//!
//! This module defines semantic articulations and ornaments that can be
//! applied to notes and chords. These are the resolved versions of the
//! various shorthand and explicit decorations in ABC notation.

/// An articulation or ornament applied to a note.
///
/// These represent the semantic meaning of decorations, converted from
/// the various shorthand and explicit notations in ABC. Dynamics are
/// handled separately in the `Dynamics` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Articulation {
    // Ornaments
    /// A trill (rapid alternation with note above).
    Trill,
    /// A trill natural (trill with natural upper note).
    TrillNatural,
    /// A trill sharp (trill with sharp upper note).
    TrillSharp,
    /// A trill flat (trill with flat upper note).
    TrillFlat,
    /// An upper mordent (single alternation to note above).
    Mordent,
    /// A lower mordent (single alternation to note below).
    LowerMordent,
    /// A turn (note-upper-note-lower-note).
    Turn,
    /// An inverted turn.
    InvertedTurn,

    // Articulations
    /// Staccato (short, detached).
    Staccato,
    /// Tenuto (held for full value).
    Tenuto,
    /// Accent (emphasis).
    Accent,
    /// Strong accent / marcato.
    Marcato,
    /// Fermata (pause, hold).
    Fermata,
    /// Short fermata.
    ShortFermata,
    /// Long fermata.
    LongFermata,

    // Bowing and technique
    /// Up bow (string instruments).
    UpBow,
    /// Down bow (string instruments).
    DownBow,
    /// Open string / harmonic.
    Open,
    /// Snap pizzicato (Bartok pizz).
    SnapPizzicato,
    /// Thumb position / left-hand pizzicato.
    Thumb,

    // Other marks
    /// Breath mark.
    Breath,
    /// Slide/glissando into note.
    Slide,
    /// Wedge (marcato variant).
    Wedge,
    /// Segno sign.
    Segno,
    /// Coda sign.
    Coda,
    /// D.S. (dal segno).
    DalSegno,
    /// D.C. (da capo).
    DaCapo,
    /// Fine (end).
    Fine,
    /// Arpeggio (rolled chord).
    Arpeggio,
    /// Roll (drum roll or tremolo).
    Roll,
}

impl Articulation {
    /// Create an articulation from a shorthand character.
    ///
    /// Returns `None` if the character doesn't map to a known articulation.
    pub fn from_shorthand(c: char) -> Option<Self> {
        match c {
            '~' => Some(Articulation::Roll), // or general ornament
            '.' => Some(Articulation::Staccato),
            'T' => Some(Articulation::Trill),
            'H' => Some(Articulation::Fermata),
            'L' => Some(Articulation::Accent),
            'M' => Some(Articulation::LowerMordent),
            'O' => Some(Articulation::Coda),
            'P' => Some(Articulation::Mordent),
            'S' => Some(Articulation::Segno),
            'u' => Some(Articulation::UpBow),
            'v' => Some(Articulation::DownBow),
            _ => None,
        }
    }

    /// Create an articulation from an explicit decoration name.
    ///
    /// Returns `None` if the name doesn't map to a known articulation.
    pub fn from_explicit(name: &str) -> Option<Self> {
        match name {
            // Ornaments
            "trill" => Some(Articulation::Trill),
            "trill(" => Some(Articulation::Trill), // start of trill
            "trill)" => None,                      // end of trill (handled elsewhere)
            "lowermordent" | "mordent" => Some(Articulation::LowerMordent),
            "uppermordent" | "pralltriller" => Some(Articulation::Mordent),
            "turn" => Some(Articulation::Turn),
            "turnx" | "invertedturn" => Some(Articulation::InvertedTurn),
            "roll" => Some(Articulation::Roll),

            // Articulations
            "." | "staccato" => Some(Articulation::Staccato),
            "tenuto" => Some(Articulation::Tenuto),
            "accent" | "emphasis" | ">" => Some(Articulation::Accent),
            "^" | "marcato" => Some(Articulation::Marcato),
            "fermata" | "pause" => Some(Articulation::Fermata),
            "shortfermata" => Some(Articulation::ShortFermata),
            "longfermata" => Some(Articulation::LongFermata),

            // Bowing
            "upbow" => Some(Articulation::UpBow),
            "downbow" => Some(Articulation::DownBow),
            "open" => Some(Articulation::Open),
            "snap" | "nail" => Some(Articulation::SnapPizzicato),
            "thumb" | "+" => Some(Articulation::Thumb),

            // Other
            "breath" | "," => Some(Articulation::Breath),
            "slide" => Some(Articulation::Slide),
            "wedge" => Some(Articulation::Wedge),
            "segno" => Some(Articulation::Segno),
            "coda" => Some(Articulation::Coda),
            "D.S." | "D.S" | "dalsegno" => Some(Articulation::DalSegno),
            "D.C." | "D.C" | "dacapo" => Some(Articulation::DaCapo),
            "fine" => Some(Articulation::Fine),
            "arpeggio" => Some(Articulation::Arpeggio),

            _ => None,
        }
    }
}
