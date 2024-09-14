use schema::{Schema, StringId};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ValuePathSegment {
    Field(StringId),
    Index(usize),
}

impl From<usize> for ValuePathSegment {
    fn from(index: usize) -> ValuePathSegment {
        ValuePathSegment::Index(index)
    }
}

impl From<StringId> for ValuePathSegment {
    fn from(id: StringId) -> ValuePathSegment {
        ValuePathSegment::Field(id)
    }
}

/// Converts a series of `ValuePathSegment` into a human-readable string representation
/// based on the provided `Schema`.
///
/// # Arguments
///
/// * `schema` - A reference to the `Schema` that provides the mapping of `StringId` to string values.
/// * `values` - A slice of `ValuePathSegment` representing the path to convert.
///
/// # Returns
///
/// A `String` representing the value path, formatted as a dot-separated string,
/// prefixed with " at path '" and suffixed with "'".
pub(super) fn value_path_to_string(schema: &Schema, values: &[ValuePathSegment]) -> String {
    let mut output = String::new();
    if values.is_empty() {
        return output;
    }
    write!(&mut output, " at path '").unwrap();
    for segment in values {
        match segment {
            ValuePathSegment::Field(id) => {
                write!(&mut output, ".{}", schema[*id]).unwrap();
            }
            ValuePathSegment::Index(idx) => {
                write!(&mut output, ".{idx}").unwrap();
            }
        }
    }
    write!(output, "'").unwrap();

    output
}
