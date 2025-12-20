//! Abstract Syntax Tree (AST) types for ABC notation.
//!
//! The AST represents the parsed structure of ABC notation in a form that
//! is as close as possible to the original source. Types in this module do
//! not perform interpretation or validation; they simply capture what was
//! written.
//!
//! Where applicable, AST types borrow string data from the input using the
//! `'input` lifetime to minimise allocations.

pub(crate) mod annotation;
pub(crate) mod barline;
pub(crate) mod chord;
pub(crate) mod decoration;
pub(crate) mod directive;
pub(crate) mod document;
pub(crate) mod element;
pub(crate) mod information_field;
pub(crate) mod lyrics;
pub(crate) mod midi;
pub(crate) mod note;
pub(crate) mod rhythm;
pub(crate) mod tune;
pub(crate) mod voice;

pub(crate) use annotation::{Annotation, AnnotationPlacement};
pub(crate) use barline::{BarLine, VariantEnding};
pub(crate) use chord::{Chord, GuitarChord};
pub(crate) use decoration::{Decoration, Dynamic, GraceNotes};
pub(crate) use directive::{
    Directive, FontDirective, FontSpec, Measurement, StylesheetDirective, TextDirective,
    VSkipAmount,
};
pub(crate) use document::{AbcDocument, DocumentHeader};
pub(crate) use element::{BodyElement, InlineField, MacroDefinition};
pub(crate) use information_field::InformationField;
pub(crate) use lyrics::LyricLine;
pub(crate) use midi::MidiDirective;
pub(crate) use note::{Note, NoteDuration, NotePitch, Rest};
pub(crate) use rhythm::{BrokenRhythm, Tuplet};
pub(crate) use tune::{Tune, TuneBody, TuneHeader};
pub(crate) use voice::{StemDirection, VoiceAttributes, VoiceDeclaration};
