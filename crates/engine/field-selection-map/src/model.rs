use std::fmt;

use itertools::Itertools as _;

/// Represents the core parsed value of a FieldSelectionMap scalar.
/// It's a list of alternative entries, separated by '|'.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedValue<'a> {
    pub alternatives: Vec<SelectedValueEntry<'a>>,
}

impl<'a> TryFrom<&'a str> for SelectedValue<'a> {
    type Error = String;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        crate::parser::parse(value)
    }
}

impl fmt::Display for SelectedValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.alternatives
                .iter()
                .format_with(" | ", |entry, f| f(&format_args!("{}", entry)))
        )
    }
}

/// Represents one possible entry in a SelectedValue (potentially unioned with |).
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedValueEntry<'a> {
    /// A path to a scalar or enum value. `field` or `object.field`
    Path(Path<'a>),
    /// A path followed by an object selection. `path.{ field1 field2 }`
    ObjectWithPath {
        path: Path<'a>,
        object: SelectedObjectValue<'a>,
    },
    /// A path followed by a list selection. `path[ ... ]`
    ListWithPath {
        path: Path<'a>,
        list: SelectedListValue<'a>,
    },
    /// A standalone object selection. `{ field1 field2 }`
    Object(SelectedObjectValue<'a>),
}

impl fmt::Display for SelectedValueEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectedValueEntry::Path(p) => write!(f, "{}", p),
            SelectedValueEntry::ObjectWithPath { path, object } => write!(f, "{}.{}", path, object),
            SelectedValueEntry::ListWithPath { path, list } => write!(f, "{}{}", path, list),
            SelectedValueEntry::Object(o) => write!(f, "{}", o),
        }
    }
}

/// Represents a segment of a Path, potentially with a type constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSegment<'a> {
    pub field: &'a str,
    pub ty: Option<&'a str>,
}

impl fmt::Display for PathSegment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.ty {
            Some(ty) => write!(f, "{}<{}>", self.field, ty),
            None => write!(f, "{}", self.field),
        }
    }
}

/// Represents a path to a field, possibly nested.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path<'a> {
    pub ty: Option<&'a str>,
    pub segments: Vec<PathSegment<'a>>,
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "<{}>.", ty)?
        };
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .format_with(".", |segment, f| f(&format_args!("{}", segment)))
        )
    }
}

/// Represents an object selection, similar to GraphQL ObjectValue.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedObjectValue<'a> {
    pub fields: Vec<SelectedObjectField<'a>>,
}

impl fmt::Display for SelectedObjectValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ {} }}",
            self.fields
                .iter()
                .format_with(" ", |field, f| f(&format_args!("{}", field)))
        )
    }
}

/// Represents a field within a SelectedObjectValue.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedObjectField<'a> {
    pub key: &'a str,
    pub value: Option<SelectedValue<'a>>,
}

impl fmt::Display for SelectedObjectField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(value) => write!(f, "{}: {}", self.key, value),
            None => write!(f, "{}", self.key),
        }
    }
}

/// Represents a list selection. Note: Spec allows only one element.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedListValue<'a>(pub SelectedValue<'a>);

impl fmt::Display for SelectedListValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0)
    }
}
