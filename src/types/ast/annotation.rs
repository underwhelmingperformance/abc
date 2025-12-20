//! Annotations in the AST.
//!
//! This module defines types for representing text annotations as they appear
//! in ABC notation. Annotations provide additional performance instructions,
//! fingerings, or other textual information placed around notes.

/// A text annotation placed around a note or chord.
///
/// Annotations in ABC notation provide textual information that appears
/// around notes, such as fingerings, performance instructions, or other
/// notational details. They are written in double quotes with a placement
/// character indicating their position.
///
/// # Examples
///
/// ```text
/// "^fine"C          - "fine" above C
/// "_p dolce"D       - "p dolce" below D
/// "<1"E             - "1" (fingering) to the left of E
/// ">mp"[CEG]        - "mp" to the right of the chord
/// "^2"C             - "2" centered above C
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation<'input> {
    /// The text content of the annotation.
    pub text: &'input str,
    /// The placement of the annotation relative to the note.
    pub placement: AnnotationPlacement,
}

/// The placement of an annotation relative to a note.
///
/// Annotations can be positioned above, below, to the left, or to the right
/// of notes and chords. The placement affects how the annotation is rendered
/// in traditional notation and tablature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnotationPlacement {
    /// Above the note or chord (^).
    Above,
    /// Below the note or chord (_).
    Below,
    /// To the left of the note or chord (<).
    Left,
    /// To the right of the note or chord (>).
    Right,
    /// Centered above the note or chord (default if no marker).
    CenterAbove,
}
