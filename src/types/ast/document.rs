//! Document-level structure in the AST.
//!
//! This module defines types for representing complete ABC documents, which may
//! contain multiple tunes along with optional document-level metadata.

use super::{InformationField, Tune};

/// A complete ABC document.
///
/// An ABC document can contain one or more tunes, optionally preceded by a
/// document header with metadata that applies to all tunes. The document header
/// is distinguished from tune headers by appearing before the first reference
/// number (X:) field.
///
/// # Structure
///
/// ```text
/// % ABC document begins
/// % Comments...
///
/// I:abc-version 2.1    % Document header (optional)
/// I:abc-creator Tool   % More document header fields
///
/// X:1                  % First tune begins
/// T:Tune One
/// K:G
/// ...
///
/// X:2                  % Second tune begins
/// T:Tune Two
/// K:D
/// ...
/// ```
#[derive(Debug, PartialEq)]
pub struct AbcDocument<'input> {
    /// Optional document header containing metadata for the entire document.
    pub header: Option<DocumentHeader<'input>>,
    /// The tunes contained in this document.
    pub tunes: Vec<Tune<'input>>,
}

/// The document header section of an ABC document.
///
/// The document header contains information fields that apply to all tunes in
/// the document. It appears before the first tune (before the first `X:` field).
/// Common document-level fields include version information, creator/software
/// identification, and default settings.
///
/// # Examples
///
/// ```text
/// I:abc-version 2.1
/// I:abc-charset utf-8
/// I:abc-creator Example Software v1.0
/// ```
#[derive(Debug, PartialEq)]
pub struct DocumentHeader<'input> {
    /// The information fields in this document header.
    pub fields: Vec<InformationField<'input>>,
}
