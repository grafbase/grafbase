use cynic_parser::Span;

use crate::Location;

/// Tracks the offsets of newlines in a string.
///
/// We use this to convert cynic_parser::Span (which uses byte offsets) -> Location (which uses
/// line/column)
pub(super) struct LineOffsets(Vec<u32>);

impl LineOffsets {
    pub fn new(doc: &str) -> Self {
        LineOffsets(
            doc.char_indices()
                .filter(|(_, char)| *char == '\n')
                .map(|(index, _)| index as u32)
                .collect(),
        )
    }

    pub fn span_to_location(&self, span: Span) -> Option<Location> {
        let offsets = &self.0;

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

        Some(Location::new(u16::try_from(line).ok()?, u16::try_from(column).ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_translation() {
        let offsets = LineOffsets::new("Hello\nThere\nFool");
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
