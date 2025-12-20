//! Parsing utilities.
//!
//! This module provides helper functions and combinators used across
//! multiple parser modules.

use nom::{
    IResult, Parser,
    character::complete::{line_ending, not_line_ending},
    combinator::opt,
    sequence::terminated,
};

/// Calculate line and column numbers from the original input and error position.
///
/// Returns (line, column) where both are 1-indexed.
pub(crate) fn calculate_position(original: &str, remaining: &str) -> (usize, usize) {
    // Calculate byte offset of remaining slice within original
    let offset = original.len().saturating_sub(remaining.len());
    let consumed = &original[..offset];

    // Count newlines to get line number
    let line = consumed.chars().filter(|&c| c == '\n').count() + 1;

    // Find column by counting chars after last newline
    let column = match consumed.rfind('\n') {
        Some(pos) => consumed[pos + 1..].chars().count() + 1,
        None => consumed.chars().count() + 1,
    };

    (line, column)
}

/// Parse content up to and including an optional line ending.
///
/// This is a common pattern for parsing field content that ends at a newline.
pub(crate) fn till_line_ending(input: &str) -> IResult<&str, &str> {
    terminated(not_line_ending, opt(line_ending)).parse(input)
}

/// Preprocess ABC input to handle line continuation with backslash.
///
/// Lines ending with a backslash are joined with the next line.
/// The backslash and following newline are removed.
pub fn preprocess_line_continuation(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Look ahead to see if this is followed by a newline
            match chars.peek() {
                Some(&'\n') => {
                    // Consume the newline and continue on next line
                    chars.next();
                }
                Some(&'\r') => {
                    // Consume \r and check for \n
                    chars.next();
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                }
                _ => {
                    // Not a line continuation, keep the backslash
                    result.push(ch);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_no_continuation() {
        let input = "T:Simple Title\nK:G\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_preprocess_single_continuation() {
        let input = "T:Long \\\nTitle\nK:G\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "T:Long Title\nK:G\n");
    }

    #[test]
    fn test_preprocess_multiple_continuations() {
        let input = "T:Very \\\nLong \\\nTitle\nK:G\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "T:Very Long Title\nK:G\n");
    }

    #[test]
    fn test_preprocess_continuation_in_body() {
        let input = "X:1\nT:Test\nK:G\nGAB\\\nc|dedB|\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "X:1\nT:Test\nK:G\nGABc|dedB|\n");
    }

    #[test]
    fn test_preprocess_backslash_not_at_line_end() {
        let input = "T:Title with \\ backslash\nK:G\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_preprocess_crlf_line_ending() {
        let input = "T:Long \\\r\nTitle\r\nK:G\r\n";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "T:Long Title\r\nK:G\r\n");
    }

    #[test]
    fn test_preprocess_empty_input() {
        let input = "";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_preprocess_continuation_at_end_of_input() {
        let input = "T:Title\\";
        let output = preprocess_line_continuation(input);
        assert_eq!(output, "T:Title\\");
    }
}
