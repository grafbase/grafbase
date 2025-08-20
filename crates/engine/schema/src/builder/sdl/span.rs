pub(crate) use cynic_parser::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    #[cfg(test)]
    pub fn new(line: usize, column: usize) -> Self {
        Location { line, column }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Tracks the offsets of newlines in a string.
///
/// We use this to convert cynic_parser::Span (which uses byte offsets) -> Location (which uses
/// line/column)
pub(crate) struct SpanTranslator<'a> {
    pub sdl: &'a str,
    line_offsets: Vec<u32>,
}

impl<'a> SpanTranslator<'a> {
    pub fn new(doc: &'a str) -> Self {
        Self {
            sdl: doc,
            line_offsets: doc
                .char_indices()
                .filter(|(_, char)| *char == '\n')
                .map(|(index, _)| index as u32)
                .collect(),
        }
    }

    pub fn span_to_location(&self, span: Span) -> Option<Location> {
        let offsets = &self.line_offsets;

        let target_offset = span.start;

        // Find which line this offset belongs to
        // partition_point returns the index where line_offset would be inserted to maintain sort order
        let index = offsets.partition_point(|line_offset| (*line_offset as usize) < target_offset);

        // Line number (1-indexed)
        let line = index + 1;

        // Calculate the start of this line
        let line_start = if index == 0 {
            0 // First line starts at position 0
        } else {
            // Previous line ended at offsets[index-1] (the newline)
            // So this line starts at the next position
            (offsets.get(index - 1).copied()? + 1) as usize
        };

        // Column is the offset within the line (1-indexed)
        let column = (target_offset - line_start) + 1;

        Some(Location { line, column })
    }

    /// Creates a display formatter for a span
    pub fn display_span(&self, span: Span) -> SpanDisplay<'_> {
        SpanDisplay { translator: self, span }
    }
}

pub(crate) struct SpanDisplay<'a> {
    translator: &'a SpanTranslator<'a>,
    span: Span,
}

impl<'a> SpanDisplay<'a> {
    fn get_line(&self, line_num: usize) -> &str {
        let start = if line_num == 1 {
            0
        } else {
            (self.translator.line_offsets[line_num - 2] + 1) as usize
        };

        let end = if line_num <= self.translator.line_offsets.len() {
            self.translator.line_offsets[line_num - 1] as usize
        } else {
            self.translator.sdl.len()
        };

        &self.translator.sdl[start..end]
    }
}

impl<'a> std::fmt::Display for SpanDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(location) = self.translator.span_to_location(self.span) else {
            return Ok(());
        };

        let total_lines = self.translator.line_offsets.len() + 1;

        // Ensure we have valid line numbers (1-indexed)
        if location.line == 0 || location.line > total_lines {
            return Ok(());
        }

        // Calculate padding for line numbers
        let max_line = std::cmp::min(location.line + 1, total_lines);
        let line_num_width = max_line.to_string().len();

        // Add line before if it exists
        if location.line > 1 {
            let line_num = location.line - 1;
            let line_content = self.get_line(line_num);
            writeln!(f, "{:>width$} | {}", line_num, line_content, width = line_num_width)?;
        }

        // Add the main line with the error
        let error_line = self.get_line(location.line);
        writeln!(f, "{:>width$} | {}", location.line, error_line, width = line_num_width)?;

        // Calculate underline position and length
        let line_start_byte = if location.line == 1 {
            0
        } else {
            (self.translator.line_offsets[location.line - 2] + 1) as usize
        };

        let span_start_in_line = self.span.start.saturating_sub(line_start_byte);
        let span_end_in_line = self.span.end.saturating_sub(line_start_byte);

        // Ensure we don't go beyond the line length
        let line_byte_len = error_line.len();
        let underline_start = std::cmp::min(span_start_in_line, line_byte_len);
        let underline_end = std::cmp::min(span_end_in_line, line_byte_len);
        let underline_len = underline_end.saturating_sub(underline_start);

        // Add the underline
        if underline_len > 0 {
            for _ in 0..(line_num_width + 3 + underline_start) {
                write!(f, " ")?;
            }
            for _ in 0..underline_len {
                write!(f, "^")?;
            }
            writeln!(f)?;
        }

        // Add line after if it exists
        if location.line < total_lines {
            let line_num = location.line + 1;
            let line_content = self.get_line(line_num);
            writeln!(f, "{:>width$} | {}", line_num, line_content, width = line_num_width)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_translation() {
        let offsets = SpanTranslator::new("Hello\nThere\nFool");
        assert_eq!(offsets.span_to_location(span(0)), Some(Location::new(1, 1)));
        assert_eq!(offsets.span_to_location(span(1)), Some(Location::new(1, 2)));
        assert_eq!(offsets.span_to_location(span(4)), Some(Location::new(1, 5)));
        assert_eq!(offsets.span_to_location(span(6)), Some(Location::new(2, 1)));
        assert_eq!(offsets.span_to_location(span(10)), Some(Location::new(2, 5)));
        assert_eq!(offsets.span_to_location(span(12)), Some(Location::new(3, 1)));
    }

    #[test]
    fn test_span_to_location_edge_cases() {
        // Test with GraphQL SDL
        let sdl = "type Query {\n  field: String @deprecated\n  other: Int\n}";
        let translator = SpanTranslator::new(sdl);

        // First character
        assert_eq!(translator.span_to_location(Span::new(0, 1)), Some(Location::new(1, 1)));

        // Newline positions
        assert_eq!(
            translator.span_to_location(Span::new(12, 13)),
            Some(Location::new(1, 13))
        ); // Right before first \n
        assert_eq!(
            translator.span_to_location(Span::new(13, 14)),
            Some(Location::new(2, 1))
        ); // Right after first \n

        // Second line positions
        assert_eq!(
            translator.span_to_location(Span::new(29, 30)),
            Some(Location::new(2, 17))
        ); // '@' of @deprecated
        assert_eq!(
            translator.span_to_location(Span::new(40, 41)),
            Some(Location::new(2, 28))
        ); // 'd' at end of deprecated

        // Third line
        assert_eq!(
            translator.span_to_location(Span::new(41, 42)),
            Some(Location::new(3, 1))
        ); // Right after second \n (first space)
        assert_eq!(
            translator.span_to_location(Span::new(53, 54)),
            Some(Location::new(3, 13))
        ); // The third \n itself

        // Fourth line (closing brace)
        assert_eq!(
            translator.span_to_location(Span::new(54, 55)),
            Some(Location::new(4, 1))
        ); // The closing brace

        // Empty string
        let empty_translator = SpanTranslator::new("");
        assert_eq!(
            empty_translator.span_to_location(Span::new(0, 1)),
            Some(Location::new(1, 1))
        );

        // Single line
        let single_line = SpanTranslator::new("type Query");
        assert_eq!(single_line.span_to_location(Span::new(0, 1)), Some(Location::new(1, 1)));
        assert_eq!(single_line.span_to_location(Span::new(5, 6)), Some(Location::new(1, 6)));
    }

    #[test]
    fn test_format_span_error() {
        let sdl = "type Query {\n  field: String @deprecated\n  other: Int\n}";
        let translator = SpanTranslator::new(sdl);

        // Test underlining "@deprecated" on line 2
        // sdl[0..13] = "type Query {\n"
        // sdl[13..41] = "  field: String @deprecated\n"
        // So "@deprecated" starts at 29 (13 + 16)
        let span = Span::new(29, 40); // Position of "@deprecated"
        let formatted = format!("{}", translator.display_span(span));
        insta::assert_snapshot!(formatted, @r###"
        1 | type Query {
        2 |   field: String @deprecated
                            ^^^^^^^^^^^
        3 |   other: Int
        "###);

        // Test first line
        let span = Span::new(0, 4); // "type"
        let formatted = format!("{}", translator.display_span(span));
        insta::assert_snapshot!(formatted, @r###"
        1 | type Query {
            ^^^^
        2 |   field: String @deprecated
        "###);

        // Test last line
        // sdl[41..53] = "  other: Int"
        // sdl[53..54] = "\n"
        // sdl[54..55] = "}"
        // So the closing brace is at position 54
        let span = Span::new(54, 55); // "}"
        let formatted = format!("{}", translator.display_span(span));
        insta::assert_snapshot!(formatted, @r###"
        3 |   other: Int
        4 | }
            ^
        "###);

        // Test multi-character span on middle line
        let span = Span::new(15, 28); // "field: String"
        let formatted = format!("{}", translator.display_span(span));
        insta::assert_snapshot!(formatted, @r###"
        1 | type Query {
        2 |   field: String @deprecated
              ^^^^^^^^^^^^^
        3 |   other: Int
        "###);
    }

    fn span(start: usize) -> Span {
        Span::new(start, start + 1)
    }
}
