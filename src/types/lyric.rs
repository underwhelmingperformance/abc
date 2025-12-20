//! Lyric types for resolved musical events.
//!
//! This module defines the semantic representation of lyrics after alignment
//! with notes. Unlike the AST representation which includes alignment markers
//! (holds, skips, bar lines), the semantic type represents what actually
//! happens at each note.

/// A lyric element attached to a note.
///
/// This is the resolved form of a lyric after alignment with notes.
/// Notes can have a syllable, be held from the previous syllable, or have
/// no lyric at all (represented as `None` in `Option<Syllable>`).
///
/// # Examples
///
/// ```
/// use abc::types::Syllable;
///
/// // "Hel-" continues to next syllable
/// let syllable = Syllable::Text {
///     text: "Hel",
///     continues: true,
/// };
/// assert!(matches!(syllable, Syllable::Text { continues: true, .. }));
///
/// // "world" is a complete word
/// let syllable = Syllable::Text {
///     text: "world",
///     continues: false,
/// };
///
/// // A held note (previous syllable extends)
/// let held = Syllable::Hold;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syllable<'input> {
    /// A text syllable to be sung.
    Text {
        /// The text of this syllable.
        ///
        /// This is borrowed from the original input string.
        text: &'input str,

        /// Whether this syllable continues to the next note's syllable.
        ///
        /// When `true`, a hyphen should be displayed after this syllable
        /// to indicate the word continues (e.g., "Hel-" "lo").
        continues: bool,
    },

    /// The previous syllable is held across this note.
    ///
    /// This corresponds to `_` in ABC notation. The renderer should show
    /// an extender line or similar indicator that the previous syllable
    /// continues.
    Hold,
}

impl<'input> Syllable<'input> {
    /// Create a new text syllable.
    pub fn new(text: &'input str, continues: bool) -> Self {
        Self::Text { text, continues }
    }

    /// Create a hold marker.
    pub fn hold() -> Self {
        Self::Hold
    }
}
