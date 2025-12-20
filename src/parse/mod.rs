//! Parsing module for ABC notation.
//!
//! This module contains parsers for converting ABC notation text into the
//! AST types. The parsers are built using the `nom` parser combinator library
//! and are organised into sub-modules by feature area.
//!
//! # Parser Module Selection
//!
//! This crate uses `nom::*::complete` parsers exclusively (e.g., `nom::bytes::complete::tag`
//! rather than `nom::bytes::streaming::tag`). This is appropriate because ABC documents
//! are read entirely into memory before parsing—there's no need to handle partial input
//! from streaming sources like network sockets.
//!
//! The `complete` variants return an error immediately when input is exhausted, whereas
//! `streaming` variants return `Incomplete` to signal that more data might arrive. Since
//! ABC files are always complete when parsed, `complete` provides cleaner error handling.
//!
//! # Organisation
//!
//! - `primitives`: Building block parsers (pitch, accidental, octave, duration)
//! - `note`: Note and rest parsers
//! - `chord`: Chord and guitar chord parsers
//! - `barline`: Bar line and repeat parsers
//! - `rhythm`: Tuplet and broken rhythm parsers
//! - `information_field`: Information field parsers (X:, T:, K:, M:, etc.)
//! - `inline_field`: Inline field parsers ([K:D], [M:3/4], etc.)
//! - `tune`: Complete tune parser (header + body)
//! - `document`: Complete document parser (optional header + tunes)
//!
//! # Usage
//!
//! Parsers follow the `nom` convention of returning `IResult<&str, T>`, where
//! the input and output lifetimes are tied together. For types that borrow
//! from the input (those with `'input` lifetime), the lifetime is inferred
//! from the input string slice.
//!
//! # Whitespace Handling
//!
//! Whitespace is handled at different granularities depending on context:
//!
//! - Between elements in the tune body, `body_element` in `tune.rs` uses
//!   `multispace0` to consume any whitespace (including newlines). This allows
//!   flexibility in formatting the tune body.
//!
//! - For information fields, parsers use `space0` to allow optional horizontal
//!   whitespace after the field tag, but fields must be on separate lines.
//!   Line endings are consumed after each field.
//!
//! - Within individual elements (notes, chords, etc.), no internal whitespace
//!   is consumed. For example, `C D` parses as two separate notes with the
//!   space consumed by the body element parser, not by the note parser itself.
//!
//! This design keeps individual parsers simple and composable whilst allowing the
//! higher-level parsers to define the overall whitespace policy.

mod annotation;
mod barline;
mod chord;
mod decoration;
mod directive;
mod document;
mod grace;
mod information_field;
mod inline_field;
mod lyrics;
mod note;
mod parser;
mod primitives;
mod rhythm;
#[cfg(test)]
mod test_utils;
mod tie_slur;
mod tune;
mod utils;
mod voice;

pub use parser::parse;
