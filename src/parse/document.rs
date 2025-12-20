//! Document parser.
//!
//! This module provides a parser for complete ABC documents, which may
//! contain multiple tunes along with an optional document-level header.

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::{line_ending, multispace0, space0},
    combinator::{cut, map, opt},
    error::context,
    multi::{many0, many1},
    sequence::{preceded, terminated},
};

use crate::{
    parse::{directive::directive, tune::tune, utils::till_line_ending},
    types::ast::{AbcDocument, DocumentHeader, InformationField},
};

/// Parse a document header (internal helper).
///
/// The document header contains information fields that appear before the
/// first tune (before the first X: field). These fields apply to all tunes
/// in the document. Common document-level fields include version information,
/// character set, and creator/software identification.
///
/// In ABC 2.1, document headers typically contain I: (instruction) fields.
///
/// This is an internal helper function used by `document()`.
fn document_header(input: &str) -> IResult<&str, DocumentHeader<'_>> {
    context(
        "document header",
        map(
            many0(terminated(document_field_line, many0(line_ending))),
            |fields| DocumentHeader { fields },
        ),
    )
    .parse(input)
}

/// Parse a single document-level information field line.
///
/// Document headers contain instruction fields (I:) and directives (%%)
/// that apply to all tunes in the document.
fn document_field_line(input: &str) -> IResult<&str, InformationField<'_>> {
    // Skip leading whitespace
    let (input, _) = space0(input)?;

    context(
        "document field",
        alt((
            // %% Directive (must come before I: since %% is more specific)
            map(directive, InformationField::Directive),
            // I: Instruction field
            preceded(
                nom::bytes::complete::tag("I:"),
                cut(map(till_line_ending, InformationField::Instruction)),
            ),
        )),
    )
    .parse(input)
}

/// Parse a complete ABC document.
///
/// A document consists of an optional document header followed by one or
/// more tunes. The document header (if present) contains information fields
/// that apply to all tunes in the document.
pub(crate) fn document(input: &str) -> IResult<&str, AbcDocument<'_>> {
    // Skip any leading whitespace
    let (input, _) = multispace0(input)?;

    context(
        "ABC document",
        map(
            (
                // Optional document header (only if fields appear before first X:)
                opt(document_header),
                // One or more tunes (with optional whitespace between them)
                many1(preceded(multispace0, tune)),
            ),
            |(header, tunes)| AbcDocument {
                header: header.filter(|h| !h.fields.is_empty()),
                tunes,
            },
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_single_tune_document() {
        let input = "X:1\nT:Example\nK:G\nGABc|dedB|\n";
        let (remaining, doc) = document(input).unwrap();

        assert_eq!(doc.tunes.len(), 1);
        assert_eq!(doc.header, None);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_multiple_tunes_document() {
        let input = "X:1\nT:First\nK:C\nCDEF|\n\nX:2\nT:Second\nK:G\nGABc|\n";
        let (remaining, doc) = document(input).unwrap();

        assert_eq!(doc.tunes.len(), 2);
        assert_eq!(doc.header, None);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_document_with_blank_lines() {
        let input = "\n\nX:1\nT:Test\nK:D\ndefg|\n\n\nX:2\nT:Another\nK:A\nABcd|\n\n";
        let (remaining, doc) = document(input).unwrap();

        assert_eq!(doc.tunes.len(), 2);
        assert_eq!(remaining, "");
    }
}
