//! Tune structure in the AST.
//!
//! This module defines types for representing complete ABC tunes, including
//! their header (metadata and musical parameters) and body (the actual music).

use super::{BodyElement, InformationField};

/// A complete ABC tune.
///
/// An ABC tune consists of a header section containing metadata and musical
/// parameters, followed by a body section containing the actual musical
/// notation. Every valid tune must have a reference number (X:), at least
/// one title (T:), and a key signature (K:) which must be the last header field.
///
/// # Structure
///
/// ```text
/// X:1                  % Header begins (reference number required)
/// T:Example Tune       % Title (required)
/// C:Traditional        % Composer (optional)
/// M:4/4                % Metre (optional)
/// L:1/8                % Unit note length (optional)
/// K:G                  % Key (required, must be last in header)
/// |:GABc dedB|         % Body begins
/// ```
#[derive(Debug, PartialEq)]
pub struct Tune<'input> {
    /// The tune header containing metadata and musical parameters.
    pub header: TuneHeader<'input>,
    /// The tune body containing the musical notation.
    pub body: TuneBody<'input>,
}

/// The header section of an ABC tune.
///
/// The header contains information fields that provide metadata about the
/// tune (title, composer, etc.) and set musical parameters (key, metre,
/// tempo, etc.). The header must contain at minimum:
/// - One `X:` (reference number) field, which must be first
/// - One `T:` (title) field
/// - One `K:` (key signature) field, which must be last
///
/// The key signature field marks the end of the header and the beginning
/// of the body.
#[derive(Debug, PartialEq)]
pub struct TuneHeader<'input> {
    /// The information fields in this header.
    ///
    /// The fields should be ordered as they appear in the ABC notation,
    /// with `X:` first and `K:` last.
    pub fields: Vec<InformationField<'input>>,
}

/// The body section of an ABC tune.
///
/// The body contains the musical notation: notes, rests, chords, bar lines,
/// and various directives. It begins immediately after the key signature
/// field in the header and continues until the start of the next tune or
/// the end of the file.
///
/// The body may be divided into multiple voices, each with its own sequence
/// of elements. Voice switching can occur inline, and elements apply to the
/// currently active voice.
#[derive(Debug, PartialEq)]
pub struct TuneBody<'input> {
    /// The sequence of musical elements in this tune body.
    pub elements: Vec<BodyElement<'input>>,
}
