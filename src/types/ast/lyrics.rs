//! Lyrics representation in the AST.
//!
//! This module defines types for representing lyrics as they appear in ABC
//! notation. Lyrics are aligned with notes in the melody and support various
//! notations for syllable continuation, held notes, and skipped notes.

/// A line of lyrics aligned with notes in the melody.
///
/// Lyric lines in ABC notation are specified with the `w:` field and contain
/// syllables that correspond to notes in the melody. Special characters
/// control syllable alignment and continuation.
///
/// # Examples
///
/// ```text
/// w:Hel-lo world      - Three syllables: "Hel", "lo", "world"
/// w:a-maz-ing grace   - "a", "maz", "ing", "grace"
/// w:sing * * loud     - "sing" on first note, skip two notes, "loud" on fourth
/// w:hold___ this      - "hold" held across three notes, then "this"
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct LyricLine<'input> {
    /// The syllables in this lyric line.
    pub syllables: Vec<LyricSyllable<'input>>,
}

/// A single syllable or special symbol in a lyric line.
///
/// Syllables can be ordinary text, continuation markers (hyphens), holds
/// (underscores to extend the previous syllable), skips (asterisks for
/// notes without lyrics), or bar line markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LyricSyllable<'input> {
    /// Text to be sung with the corresponding note.
    Text {
        /// The text of this syllable.
        text: &'input str,
        /// Whether this syllable continues to the next (ends with hyphen).
        continues: bool,
    },
    /// Hold the previous syllable for this note (underscore).
    ///
    /// Indicates that the previous syllable should be held across this
    /// note rather than starting a new syllable.
    Hold,
    /// Skip this note (asterisk).
    ///
    /// Indicates that this note has no lyrics associated with it.
    Skip,
    /// A bar line marker in the lyrics (vertical bar).
    ///
    /// Used to align lyrics with bar lines for readability, but does
    /// not affect lyric-note alignment.
    BarLine,
    /// A space in the lyric line.
    ///
    /// Multiple spaces can be used to skip multiple notes at once.
    Space,
}
