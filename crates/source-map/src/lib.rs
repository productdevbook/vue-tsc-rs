//! Source position tracking and mapping for vue-tsc-rs.
//!
//! This crate provides utilities for tracking source positions and mapping
//! between original Vue source code and generated TypeScript code.

use std::ops::Range;
pub use text_size::{TextRange, TextSize};

/// A span in the source code, representing a half-open range [start, end).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Start offset (inclusive)
    pub start: u32,
    /// End offset (exclusive)
    pub end: u32,
}

impl Span {
    /// Create a new span from start and end offsets.
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create an empty span at the given offset.
    #[inline]
    pub const fn empty(offset: u32) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Create a span from a range.
    #[inline]
    pub fn from_range(range: Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }

    /// Get the length of the span.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if the span is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Check if this span contains another span.
    #[inline]
    pub const fn contains(&self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Check if this span contains an offset.
    #[inline]
    pub const fn contains_offset(&self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Merge two spans into one that covers both.
    #[inline]
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Convert to a TextRange.
    #[inline]
    pub fn to_text_range(self) -> TextRange {
        TextRange::new(TextSize::new(self.start), TextSize::new(self.end))
    }

    /// Convert to a Range<usize>.
    #[inline]
    pub fn to_range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl From<TextRange> for Span {
    fn from(range: TextRange) -> Self {
        Self {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
}

impl From<Span> for TextRange {
    fn from(span: Span) -> Self {
        TextRange::new(TextSize::new(span.start), TextSize::new(span.end))
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self::from_range(range)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.to_range()
    }
}

/// A line index for converting between byte offsets and line/column positions.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets of the start of each line.
    line_starts: Vec<u32>,
    /// Total length of the source.
    len: u32,
}

impl LineIndex {
    /// Create a new line index from source text.
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in text.char_indices() {
            if c == '\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self {
            line_starts,
            len: text.len() as u32,
        }
    }

    /// Get the line and column for a byte offset.
    /// Line and column are 0-indexed.
    pub fn line_col(&self, offset: u32) -> LineCol {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        let col = offset - line_start;
        LineCol {
            line: line as u32,
            col,
        }
    }

    /// Get the byte offset for a line and column.
    /// Returns None if the position is out of bounds.
    pub fn offset(&self, line_col: LineCol) -> Option<u32> {
        let line_start = self.line_starts.get(line_col.line as usize)?;
        let offset = line_start + line_col.col;
        if offset <= self.len {
            Some(offset)
        } else {
            None
        }
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the start offset of a line.
    pub fn line_start(&self, line: u32) -> Option<u32> {
        self.line_starts.get(line as usize).copied()
    }

    /// Get the end offset of a line (exclusive, including newline if present).
    pub fn line_end(&self, line: u32) -> Option<u32> {
        let line_idx = line as usize;
        if line_idx + 1 < self.line_starts.len() {
            Some(self.line_starts[line_idx + 1])
        } else if line_idx < self.line_starts.len() {
            Some(self.len)
        } else {
            None
        }
    }
}

/// A line and column position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineCol {
    /// 0-indexed line number.
    pub line: u32,
    /// 0-indexed column (byte offset within line).
    pub col: u32,
}

impl LineCol {
    /// Create a new line/column position.
    #[inline]
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }

    /// Convert to 1-indexed for display.
    #[inline]
    pub const fn to_display(self) -> (u32, u32) {
        (self.line + 1, self.col + 1)
    }
}

/// A mapping from generated code back to original source.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceMapping {
    /// Offset in the generated code.
    pub generated_offset: u32,
    /// Length in the generated code.
    pub generated_length: u32,
    /// Offset in the original source.
    pub source_offset: u32,
    /// Length in the original source.
    pub source_length: u32,
    /// Optional source file name for multi-file mappings.
    pub source_file: Option<String>,
}

impl SourceMapping {
    /// Create a new source mapping with equal lengths.
    pub fn new(generated_offset: u32, source_offset: u32, length: u32) -> Self {
        Self {
            generated_offset,
            generated_length: length,
            source_offset,
            source_length: length,
            source_file: None,
        }
    }

    /// Create a mapping with different generated and source lengths.
    pub fn new_with_lengths(
        generated_offset: u32,
        generated_length: u32,
        source_offset: u32,
        source_length: u32,
    ) -> Self {
        Self {
            generated_offset,
            generated_length,
            source_offset,
            source_length,
            source_file: None,
        }
    }

    /// Set the source file name.
    pub fn with_source_file(mut self, file: String) -> Self {
        self.source_file = Some(file);
        self
    }

    /// Get the generated span.
    pub fn generated_span(&self) -> Span {
        Span::new(
            self.generated_offset,
            self.generated_offset + self.generated_length,
        )
    }

    /// Get the source span.
    pub fn source_span(&self) -> Span {
        Span::new(self.source_offset, self.source_offset + self.source_length)
    }
}

/// A source map containing multiple mappings.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceMap {
    /// All mappings, sorted by generated offset.
    mappings: Vec<SourceMapping>,
}

impl SourceMap {
    /// Create a new empty source map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping to the source map.
    pub fn add_mapping(&mut self, mapping: SourceMapping) {
        // Insert in sorted order by generated offset
        let pos = self
            .mappings
            .partition_point(|m| m.generated_offset < mapping.generated_offset);
        self.mappings.insert(pos, mapping);
    }

    /// Add a simple mapping with equal lengths.
    pub fn add(&mut self, generated_offset: u32, source_offset: u32, length: u32) {
        self.add_mapping(SourceMapping::new(generated_offset, source_offset, length));
    }

    /// Find the source position for a generated offset.
    pub fn find_source(&self, generated_offset: u32) -> Option<&SourceMapping> {
        // Binary search for the mapping containing this offset
        let idx = self
            .mappings
            .partition_point(|m| m.generated_offset + m.generated_length <= generated_offset);
        if idx > 0 {
            let mapping = &self.mappings[idx - 1];
            if mapping.generated_offset <= generated_offset
                && generated_offset < mapping.generated_offset + mapping.generated_length
            {
                return Some(mapping);
            }
        }
        if idx < self.mappings.len() {
            let mapping = &self.mappings[idx];
            if mapping.generated_offset <= generated_offset
                && generated_offset < mapping.generated_offset + mapping.generated_length
            {
                return Some(mapping);
            }
        }
        None
    }

    /// Map a generated offset to a source offset.
    pub fn to_source_offset(&self, generated_offset: u32) -> Option<u32> {
        self.find_source(generated_offset).map(|m| {
            let delta = generated_offset - m.generated_offset;
            // Scale the delta if lengths differ
            if m.generated_length == m.source_length {
                m.source_offset + delta
            } else if m.generated_length > 0 {
                m.source_offset + (delta * m.source_length / m.generated_length)
            } else {
                m.source_offset
            }
        })
    }

    /// Get all mappings.
    pub fn mappings(&self) -> &[SourceMapping] {
        &self.mappings
    }

    /// Check if the source map is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Get the number of mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Merge another source map into this one.
    pub fn merge(&mut self, other: &SourceMap) {
        for mapping in &other.mappings {
            self.add_mapping(mapping.clone());
        }
    }
}

/// Builder for generating code with source mappings.
#[derive(Debug, Default)]
pub struct CodeBuilder {
    /// The generated code.
    code: String,
    /// The source map.
    source_map: SourceMap,
}

impl CodeBuilder {
    /// Create a new code builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current offset in the generated code.
    pub fn offset(&self) -> u32 {
        self.code.len() as u32
    }

    /// Append code without mapping.
    pub fn push_str(&mut self, code: &str) {
        self.code.push_str(code);
    }

    /// Append a character without mapping.
    pub fn push(&mut self, c: char) {
        self.code.push(c);
    }

    /// Append code with a mapping to the source.
    pub fn push_mapped(&mut self, code: &str, source_offset: u32) {
        let generated_offset = self.offset();
        let len = code.len() as u32;
        self.code.push_str(code);
        if len > 0 {
            self.source_map.add(generated_offset, source_offset, len);
        }
    }

    /// Append code with a custom mapping.
    pub fn push_with_mapping(&mut self, code: &str, source_offset: u32, source_length: u32) {
        let generated_offset = self.offset();
        let generated_length = code.len() as u32;
        self.code.push_str(code);
        if generated_length > 0 || source_length > 0 {
            self.source_map.add_mapping(SourceMapping::new_with_lengths(
                generated_offset,
                generated_length,
                source_offset,
                source_length,
            ));
        }
    }

    /// Append a newline.
    pub fn newline(&mut self) {
        self.code.push('\n');
    }

    /// Get the generated code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the source map.
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Consume the builder and return the code and source map.
    pub fn finish(self) -> (String, SourceMap) {
        (self.code, self.source_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span() {
        let span = Span::new(10, 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
        assert!(span.contains_offset(15));
        assert!(!span.contains_offset(5));
        assert!(!span.contains_offset(25));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(15, 30);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_line_index() {
        let text = "hello\nworld\nfoo";
        let index = LineIndex::new(text);

        assert_eq!(index.line_count(), 3);

        // First line
        assert_eq!(index.line_col(0), LineCol::new(0, 0));
        assert_eq!(index.line_col(5), LineCol::new(0, 5));

        // Second line (after newline)
        assert_eq!(index.line_col(6), LineCol::new(1, 0));
        assert_eq!(index.line_col(11), LineCol::new(1, 5));

        // Third line
        assert_eq!(index.line_col(12), LineCol::new(2, 0));

        // Reverse mapping
        assert_eq!(index.offset(LineCol::new(0, 0)), Some(0));
        assert_eq!(index.offset(LineCol::new(1, 0)), Some(6));
        assert_eq!(index.offset(LineCol::new(2, 0)), Some(12));
    }

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        map.add(0, 100, 10);
        map.add(20, 200, 10);

        // Within first mapping
        assert_eq!(map.to_source_offset(5), Some(105));

        // Within second mapping
        assert_eq!(map.to_source_offset(25), Some(205));

        // Outside mappings
        assert_eq!(map.to_source_offset(15), None);
    }

    #[test]
    fn test_code_builder() {
        let mut builder = CodeBuilder::new();
        builder.push_str("const x = ");
        builder.push_mapped("value", 50);
        builder.push_str(";");

        let (code, map) = builder.finish();
        assert_eq!(code, "const x = value;");
        assert_eq!(map.to_source_offset(10), Some(50));
    }
}
