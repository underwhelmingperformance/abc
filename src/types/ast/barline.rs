//! Bar line and repeat notation in the AST.
//!
//! This module defines types for representing bar lines, repeat signs, and
//! variant endings as they appear in ABC notation.

/// A bar line marking measure boundaries and repeat structures.
///
/// Bar lines in ABC notation serve both to delimit measures and to indicate
/// repeat structures. Different bar line styles have different musical meanings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarLine {
    /// `|` - Standard bar line marking the end of a measure.
    Single,
    /// `||` - Double bar line marking a section ending or significant division.
    Double,
    /// `|]` - Thin-thick double bar indicating the end of a piece or major section.
    FinalDouble,
    /// `[|` - Thick-thin double bar indicating the start of a new section.
    StartDouble,
    /// `|:` - Start of a repeated section.
    RepeatStart,
    /// `:|` - End of a repeated section (return to the most recent repeat start).
    RepeatEnd,
    /// `::` - Both end the previous repeat and start a new one (double repeat).
    RepeatBoth,
}

/// A variant ending (first/second time ending).
///
/// Variant endings specify different music to play on different iterations
/// through a repeated section. For example, `|1` marks the first ending and
/// `|2` marks the second ending.
///
/// # Examples
///
/// ```text
/// |: ABC |1 DEF :|2 GHI ||
/// ```
///
/// First time through: ABC DEF (then repeat)
/// Second time through: ABC GHI (then continue)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantEnding {
    /// The bar line that introduces this ending.
    pub bar: BarLine,
    /// The iteration numbers when this ending should be played (e.g., [1, 3] for `[1,3`).
    pub variants: Vec<u8>,
}
