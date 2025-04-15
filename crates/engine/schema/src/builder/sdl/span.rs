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
        let index = offsets.partition_point(|line_offset| (*line_offset as usize) <= target_offset);

        // We here + 1 to account for one-indexing
        let line = index + 1;

        let line_start = if index == 0 {
            0
        } else {
            (offsets.get(index - 1).copied()? + 1) as usize
        };
        let column = (target_offset - line_start) + 1;

        Some(Location { line, column })
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

    fn span(start: usize) -> Span {
        Span::new(start, start + 1)
    }
}
