//! Rhythmic notation in the AST.
//!
//! This module defines types for representing tuplets and broken rhythm notation
//! as they appear in ABC notation.

/// A tuplet (irregular rhythmic grouping).
///
/// Tuplets allow notes to be grouped in irregular divisions, such as triplets
/// (3 notes in the time of 2) or duplets (2 notes in the time of 3). In ABC
/// notation, tuplets are written as `(p:q:r`, where `p` is the number of notes
/// in the tuplet, `q` is the number they replace, and `r` is their total duration.
///
/// # Examples
///
/// ```text
/// (3abc         - Triplet: 3 notes in the time of 2
/// (2ab          - Duplet: 2 notes in the time of 3
/// (3:2:4 abc    - 3 notes in the time of 2, occupying 4 unit lengths
/// (4abcd        - Quadruplet
/// ```
///
/// This is a placeholder; tuplets will contain `BodyElement` references
/// once that type is defined.
#[derive(Debug, PartialEq, Eq)]
pub struct Tuplet {
    /// The number of notes in the tuplet.
    pub p: u8,
    /// The number of notes they replace (optional, inferred if not specified).
    pub q: Option<u8>,
    /// The total duration they occupy (optional, inferred if not specified).
    pub r: Option<u32>,
}

/// Broken rhythm notation.
///
/// Broken rhythm is a shorthand in ABC notation for dotted rhythm patterns,
/// where one note is lengthened and the adjacent note is shortened. The `>`
/// operator makes the first note dotted and the second halved, whilst `<` does
/// the reverse.
///
/// # Examples
///
/// ```text
/// A>B    - A dotted, B halved (dotted eighth + sixteenth)
/// A<B    - A halved, B dotted (sixteenth + dotted eighth)
/// A>>B   - A double-dotted, B quartered
/// A<<B   - A quartered, B double-dotted
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrokenRhythm {
    /// `>` - First note dotted, second note halved.
    DotFirst,
    /// `>>` - First note double-dotted, second note quartered.
    DoubleDotFirst,
    /// `<` - First note halved, second note dotted.
    DotSecond,
    /// `<<` - First note quartered, second note double-dotted.
    DoubleDotSecond,
}
