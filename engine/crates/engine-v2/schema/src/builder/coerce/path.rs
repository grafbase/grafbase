use crate::{Schema, StringId};
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
