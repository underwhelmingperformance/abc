//! Directive types in the AST.
//!
//! This module defines types for representing `%%` directives as they appear
//! in ABC notation. Directives control formatting, layout, MIDI output, and
//! other aspects of ABC processing.
//!
//! Directives are categorised into:
//! - MIDI directives (`%%MIDI ...`) for playback control
//! - Stylesheet directives (`%%pagewidth`, `%%scale`, etc.) for layout
//! - Font directives (`%%titlefont`, etc.) for typography
//! - Text directives (`%%text`, `%%center`, etc.) for annotations

use super::MidiDirective;

/// A directive controlling ABC processing.
///
/// Directives in ABC notation are lines starting with `%%` followed by the
/// directive name and parameters. They appear in the file header, tune header,
/// or tune body.
///
/// # Examples
///
/// ```text
/// %%pagewidth 21cm
/// %%MIDI program 0
/// %%titlefont Times-Bold 16
/// %%center My Tune Collection
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Directive<'input> {
    /// A MIDI directive controlling playback.
    Midi(MidiDirective<'input>),

    /// A stylesheet directive controlling layout.
    Stylesheet(StylesheetDirective<'input>),

    /// A font directive controlling typography.
    Font(FontDirective<'input>),

    /// A text directive for annotations.
    Text(TextDirective<'input>),

    /// An unknown or unsupported directive.
    ///
    /// Preserved for forward compatibility with future ABC extensions.
    Unknown {
        /// The directive name (without `%%` prefix).
        name: &'input str,
        /// The directive value/parameters.
        value: &'input str,
    },
}

/// A stylesheet directive controlling page layout and formatting.
///
/// These directives control how the ABC notation is rendered, including
/// page dimensions, margins, staff spacing, and other visual parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum StylesheetDirective<'input> {
    /// `%%pagewidth` - Set page width.
    PageWidth(Measurement<'input>),

    /// `%%pageheight` - Set page height.
    PageHeight(Measurement<'input>),

    /// `%%topmargin` - Set top margin.
    TopMargin(Measurement<'input>),

    /// `%%botmargin` - Set bottom margin.
    BottomMargin(Measurement<'input>),

    /// `%%leftmargin` - Set left margin.
    LeftMargin(Measurement<'input>),

    /// `%%rightmargin` - Set right margin.
    RightMargin(Measurement<'input>),

    /// `%%scale` - Set scaling factor.
    Scale(f32),

    /// `%%staffsep` - Set space between staves.
    StaffSep(Measurement<'input>),

    /// `%%sysstaffsep` - Set space between systems.
    SysStaffSep(Measurement<'input>),

    /// `%%barsperstaff` - Set bars per staff line.
    BarsPerStaff(u8),

    /// `%%staffwidth` - Set staff width.
    StaffWidth(Measurement<'input>),

    /// `%%indent` - Set first line indent.
    Indent(Measurement<'input>),

    /// `%%lineskipfac` - Set line skip factor.
    LineSkipFac(f32),

    /// `%%parskipfac` - Set paragraph skip factor.
    ParSkipFac(f32),

    /// `%%musicspace` - Set space before music.
    MusicSpace(Measurement<'input>),

    /// `%%vocalspace` - Set space for vocals.
    VocalSpace(Measurement<'input>),

    /// `%%textspace` - Set space for text.
    TextSpace(Measurement<'input>),

    /// `%%notespacingfactor` - Set note spacing factor.
    NoteSpacingFactor(f32),

    /// `%%maxshrink` - Set maximum shrink factor.
    MaxShrink(f32),

    /// `%%linewarn` - Enable/disable line warnings.
    LineWarn(bool),

    /// `%%continueall` - Continue all lines.
    ContinueAll(bool),

    /// `%%landscape` - Set landscape orientation.
    Landscape(bool),

    /// `%%stretchstaff` - Stretch staff to fill page.
    StretchStaff(bool),

    /// `%%stretchlast` - Stretch last line.
    StretchLast(f32),

    /// `%%linebreak` - Line break control symbols.
    ///
    /// Defines symbols that control automatic line breaking in ABC notation.
    /// Common symbols:
    /// - `<` - decrease line break preference
    /// - `!` - prevent line break
    /// - `$` - force line break
    /// - ` ` (space) - allow line break
    LineBreak {
        /// The line break control symbols.
        symbols: Vec<char>,
    },

    /// `%%measurenb` - Measure numbering start/interval.
    ///
    /// Sets the interval at which measure numbers are displayed.
    /// A value of 0 disables measure numbering.
    /// A value of N displays every Nth measure number.
    MeasureNb(u32),

    /// `%%breaklimit` - Line fullness threshold.
    ///
    /// Sets the threshold (0.0-1.0) for when automatic line breaking occurs.
    /// Lower values allow more flexible breaking, higher values try to fill lines more.
    BreakLimit(f32),

    /// An unknown or vendor-specific stylesheet directive.
    Other {
        /// The directive name.
        name: &'input str,
        /// The directive value.
        value: &'input str,
    },
}

/// A measurement value with optional unit.
///
/// Used for page dimensions, margins, and spacing. When no unit is specified,
/// the default unit depends on the context (typically points or centimetres).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement<'input> {
    /// The numeric value.
    pub value: f32,
    /// The unit of measurement, if specified.
    ///
    /// Common units are `"cm"`, `"in"`, `"pt"` (points), and `"mm"`.
    pub unit: Option<&'input str>,
}

impl<'input> Measurement<'input> {
    /// Create a new measurement with the given value and unit.
    #[must_use]
    pub const fn new(value: f32, unit: Option<&'input str>) -> Self {
        Self { value, unit }
    }

    /// Create a measurement in centimetres.
    #[must_use]
    pub const fn cm(value: f32) -> Self {
        Self {
            value,
            unit: Some("cm"),
        }
    }

    /// Create a measurement in inches.
    #[must_use]
    pub const fn inches(value: f32) -> Self {
        Self {
            value,
            unit: Some("in"),
        }
    }

    /// Create a measurement in points.
    #[must_use]
    pub const fn points(value: f32) -> Self {
        Self {
            value,
            unit: Some("pt"),
        }
    }
}

/// A font directive controlling typography.
///
/// These directives specify fonts for various text elements in the rendered
/// output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontDirective<'input> {
    /// `%%titlefont` - Font for tune titles.
    TitleFont(FontSpec<'input>),

    /// `%%subtitlefont` - Font for subtitles.
    SubtitleFont(FontSpec<'input>),

    /// `%%composerfont` - Font for composer names.
    ComposerFont(FontSpec<'input>),

    /// `%%partsfont` - Font for part labels.
    PartsFont(FontSpec<'input>),

    /// `%%tempofont` - Font for tempo markings.
    TempoFont(FontSpec<'input>),

    /// `%%gchordfont` - Font for guitar chords.
    GchordFont(FontSpec<'input>),

    /// `%%annotationfont` - Font for annotations.
    AnnotationFont(FontSpec<'input>),

    /// `%%infofont` - Font for information fields.
    InfoFont(FontSpec<'input>),

    /// `%%textfont` - Font for free text.
    TextFont(FontSpec<'input>),

    /// `%%vocalfont` - Font for lyrics/vocals.
    VocalFont(FontSpec<'input>),

    /// `%%wordsfont` - Font for words.
    WordsFont(FontSpec<'input>),

    /// `%%historyfont` - Font for history field.
    HistoryFont(FontSpec<'input>),

    /// `%%footerfont` - Font for page footer.
    FooterFont(FontSpec<'input>),

    /// `%%headerfont` - Font for page header.
    HeaderFont(FontSpec<'input>),

    /// An unknown or vendor-specific font directive.
    Other {
        /// The font element name (e.g., "measurenb" for measure number font).
        element: &'input str,
        /// The font specification.
        spec: FontSpec<'input>,
    },
}

/// A font specification with family and optional size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSpec<'input> {
    /// The font family name.
    ///
    /// May include style modifiers like "Times-Bold" or "Helvetica-Italic".
    pub family: &'input str,

    /// The font size in points.
    pub size: Option<u8>,

    /// Additional font modifiers.
    ///
    /// May include encoding, style, or other attributes.
    pub modifiers: Option<&'input str>,
}

impl<'input> FontSpec<'input> {
    /// Create a new font specification.
    #[must_use]
    pub const fn new(family: &'input str, size: Option<u8>) -> Self {
        Self {
            family,
            size,
            modifiers: None,
        }
    }

    /// Create a font specification with modifiers.
    #[must_use]
    pub const fn with_modifiers(
        family: &'input str,
        size: Option<u8>,
        modifiers: &'input str,
    ) -> Self {
        Self {
            family,
            size,
            modifiers: Some(modifiers),
        }
    }
}

/// A text directive for annotations and free text.
///
/// These directives add text to the output that is not part of the music
/// notation itself.
#[derive(Debug, Clone, PartialEq)]
pub enum TextDirective<'input> {
    /// `%%text` - Add left-aligned text.
    Text(&'input str),

    /// `%%center` - Add centred text.
    Center(&'input str),

    /// `%%right` - Add right-aligned text.
    Right(&'input str),

    /// `%%vskip` - Add vertical space.
    VSkip(VSkipAmount<'input>),

    /// `%%sep` - Add a separator line.
    Sep {
        /// Height of the separator.
        height: Option<Measurement<'input>>,
        /// Width of the separator.
        width: Option<Measurement<'input>>,
        /// Length of the separator line.
        length: Option<Measurement<'input>>,
    },

    /// `%%newpage` - Start a new page.
    NewPage,

    /// `%%begintext` - Begin a text block.
    BeginText,

    /// `%%endtext` - End a text block.
    EndText,

    /// `%%header` - Set page header.
    Header(&'input str),

    /// `%%footer` - Set page footer.
    Footer(&'input str),
}

/// Amount of vertical space to skip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VSkipAmount<'input> {
    /// Skip a specific measurement.
    Measurement(Measurement<'input>),
    /// Skip a number of lines (default if just a number).
    Lines(f32),
}

impl<'input> From<Measurement<'input>> for VSkipAmount<'input> {
    fn from(m: Measurement<'input>) -> Self {
        Self::Measurement(m)
    }
}

impl From<f32> for VSkipAmount<'_> {
    fn from(lines: f32) -> Self {
        Self::Lines(lines)
    }
}
