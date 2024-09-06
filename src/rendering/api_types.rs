//!Types used in the the renderer API
use crossterm::style::Color;

/// Struct to describe text style.
///
/// Used in [Draw].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TextStyle {
    pub foreground: Color,
    pub background: Color,
}

/// Struct to describe a styled segment of data.
///
/// Used in [Draw::StyledData].
#[derive(Debug)]
pub struct StyledSegment {
    /// Byte offset of the start of the segment from the start of data.
    pub start: usize,
    /// Length of the segment in bytes.
    pub length: usize,
    /// Style of the segment.
    pub style: TextStyle,
}
/// Struct to describe text that is drawn over the data.
///
/// Used in [Draw::StyledData].
#[derive(Debug)]
pub struct DataOverlay {
    // The text to draw.
    pub text: String,
    /// Byte offset from the start of data where to start drawing the text.
    pub location: usize,
}

/// Instruction to [Renderer] about what should be drawn to the screen.
#[derive(Debug)]
pub enum Draw {
    /// Draw the data, i.e. the text from which the selection is performed
    /// with parts of data in different styles and text drawn over some parts.
    ///
    /// If some of the segments are overlapping, the last one specified takes precedence.
    StyledData {
        /// The segments of the data to style differently than the source data.
        styled_segments: Vec<StyledSegment>,
        /// Parts of the data to replace with the given text.
        text_overlays: Vec<DataOverlay>,
    },
}
