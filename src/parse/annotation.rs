//! Annotation parsers.
//!
//! This module provides parsers for text annotations in ABC notation.
//! Annotations provide additional information placed around notes.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::is_not,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    sequence::delimited,
};

use crate::types::ast::{Annotation, AnnotationPlacement};

/// Parse an annotation.
///
/// Annotations are written in double quotes with an optional placement character:
/// - `"^text"` - above
/// - `"_text"` - below
/// - `"<text"` - left
/// - `">text"` - right
/// - `"text"` - centered above (default)
pub(crate) fn annotation(input: &str) -> IResult<&str, Annotation<'_>> {
    context(
        "annotation",
        delimited(
            char('"'),
            cut(map(
                (
                    opt(alt((char('^'), char('_'), char('<'), char('>')))),
                    is_not("\""),
                ),
                |(placement_char, text)| {
                    let placement = match placement_char {
                        Some('^') => AnnotationPlacement::Above,
                        Some('_') => AnnotationPlacement::Below,
                        Some('<') => AnnotationPlacement::Left,
                        Some('>') => AnnotationPlacement::Right,
                        None => AnnotationPlacement::CenterAbove,
                        _ => unreachable!(),
                    };
                    Annotation { text, placement }
                },
            )),
            cut(char('"')),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("\"^fine\"", ("", Annotation { text: "fine", placement: AnnotationPlacement::Above }))]
    #[case("\"_p dolce\"", ("", Annotation { text: "p dolce", placement: AnnotationPlacement::Below }))]
    #[case("\"<1\"", ("", Annotation { text: "1", placement: AnnotationPlacement::Left }))]
    #[case("\">mp\"", ("", Annotation { text: "mp", placement: AnnotationPlacement::Right }))]
    #[case("\"fermata\"", ("", Annotation { text: "fermata", placement: AnnotationPlacement::CenterAbove }))]
    fn test_annotation(#[case] input: &str, #[case] expected: (&str, Annotation)) {
        let result = annotation(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_annotation_with_trailing() {
        let result = annotation("\"^2\"ABC").unwrap();
        assert_eq!(
            result,
            (
                "ABC",
                Annotation {
                    text: "2",
                    placement: AnnotationPlacement::Above
                }
            )
        );
    }
}
