//! Directive parsing for ABC notation.
//!
//! This module parses `%%` directives which control formatting, layout, MIDI
//! output, and other aspects of ABC processing. Directives are pseudo-comments
//! that begin with `%%` followed by the directive name and parameters.
//!
//! # Directive Categories
//!
//! - **MIDI directives** (`%%MIDI ...`) control playback parameters
//! - **Stylesheet directives** (`%%pagewidth`, etc.) control layout
//! - **Font directives** (`%%titlefont`, etc.) control typography
//! - **Text directives** (`%%text`, `%%center`, etc.) add annotations
//!
//! # Examples
//!
//! ```text
//! %%pagewidth 21cm
//! %%MIDI program 0
//! %%titlefont Times-Bold 16
//! %%center My Tune Collection
//! ```

mod font;
mod midi;
mod parse;
mod stylesheet;
mod text;

pub(crate) use parse::directive;
