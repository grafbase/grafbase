use crate::{StringId, builder::Context};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValuePathSegment {
    Field(StringId),
    FieldStr(String),
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

impl From<String> for ValuePathSegment {
    fn from(s: String) -> ValuePathSegment {
        ValuePathSegment::FieldStr(s)
    }
}

impl From<&str> for ValuePathSegment {
    fn from(s: &str) -> ValuePathSegment {
        ValuePathSegment::FieldStr(s.to_string())
    }
}

pub(super) fn value_path_to_string(ctx: &Context<'_>, value_path: &[ValuePathSegment]) -> String {
    let mut output = String::new();
    if value_path.is_empty() {
        return output;
    }
    output.push_str(" at path '");
    for segment in value_path {
        output.push('.');
        match segment {
            ValuePathSegment::Field(id) => {
                output.push_str(&ctx.strings[*id]);
            }
            ValuePathSegment::Index(idx) => {
                write!(&mut output, "{}", idx).unwrap();
            }
            ValuePathSegment::FieldStr(s) => {
                output.push_str(s);
            }
        }
    }
    output.push('\'');

    output
}
