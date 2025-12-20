//! Lyrics parser.
//!
//! This module parses lyric lines (w: fields) in ABC notation.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{char, line_ending, space1},
    combinator::{map, opt, value},
    error::context,
    multi::many0,
    sequence::terminated,
};

use crate::types::ast::lyrics::{LyricLine, LyricSyllable};

/// Parse a lyric line (content after `w:`).
///
/// Lyric lines contain syllables separated by spaces, with special markers:
/// - `-` at end of text continues to next syllable
/// - `_` holds the previous syllable
/// - `*` skips a note
/// - `|` marks a bar line
pub(crate) fn lyric_line(input: &str) -> IResult<&str, LyricLine<'_>> {
    context(
        "lyric line",
        map(
            terminated(many0(lyric_element), opt(line_ending)),
            |syllables| LyricLine { syllables },
        ),
    )
    .parse(input)
}

/// Parse a single lyric element (syllable or special marker).
fn lyric_element(input: &str) -> IResult<&str, LyricSyllable<'_>> {
    alt((
        // Space (note skip via whitespace)
        value(LyricSyllable::Space, space1),
        // Hold marker
        value(LyricSyllable::Hold, char('_')),
        // Skip marker
        value(LyricSyllable::Skip, char('*')),
        // Bar line marker
        value(LyricSyllable::BarLine, char('|')),
        // Text syllable (possibly with continuation hyphen)
        lyric_text,
    ))
    .parse(input)
}

/// Parse a text syllable, checking for continuation hyphen.
fn lyric_text(input: &str) -> IResult<&str, LyricSyllable<'_>> {
    // Take characters that aren't special markers or whitespace
    let (input, text) = take_while1(|c: char| {
        !c.is_whitespace() && c != '_' && c != '*' && c != '|'
    })(input)?;

    // Check if text ends with hyphen (continuation)
    let (text, continues) = if let Some(stripped) = text.strip_suffix('-') {
        (stripped, true)
    } else {
        (text, false)
    };

    Ok((input, LyricSyllable::Text { text, continues }))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_simple_lyrics() {
        let input = "Do Re Mi\n";
        let (remaining, result) = lyric_line(input).unwrap();

        let expected = LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "Do",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "Re",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "Mi",
                    continues: false,
                },
            ],
        };

        assert_eq!((remaining, result), ("", expected));
    }

    #[test]
    fn test_lyrics_with_continuation() {
        // Space after hyphen separates syllables in ABC notation
        let input = "Hel- lo world\n";
        let (remaining, result) = lyric_line(input).unwrap();

        let expected = LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "Hel",
                    continues: true,
                },
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "lo",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "world",
                    continues: false,
                },
            ],
        };

        assert_eq!((remaining, result), ("", expected));
    }

    #[test]
    fn test_lyrics_with_hold() {
        let input = "hold _ _ this\n";
        let (remaining, result) = lyric_line(input).unwrap();

        let expected = LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "hold",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Hold,
                LyricSyllable::Space,
                LyricSyllable::Hold,
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "this",
                    continues: false,
                },
            ],
        };

        assert_eq!((remaining, result), ("", expected));
    }

    #[test]
    fn test_lyrics_with_skip() {
        let input = "sing * * loud\n";
        let (remaining, result) = lyric_line(input).unwrap();

        let expected = LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "sing",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Skip,
                LyricSyllable::Space,
                LyricSyllable::Skip,
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "loud",
                    continues: false,
                },
            ],
        };

        assert_eq!((remaining, result), ("", expected));
    }

    #[test]
    fn test_lyrics_with_barline() {
        let input = "one two | three\n";
        let (remaining, result) = lyric_line(input).unwrap();

        let expected = LyricLine {
            syllables: vec![
                LyricSyllable::Text {
                    text: "one",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "two",
                    continues: false,
                },
                LyricSyllable::Space,
                LyricSyllable::BarLine,
                LyricSyllable::Space,
                LyricSyllable::Text {
                    text: "three",
                    continues: false,
                },
            ],
        };

        assert_eq!((remaining, result), ("", expected));
    }
}
