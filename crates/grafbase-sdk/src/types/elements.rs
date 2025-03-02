use crate::{types::DirectiveSite, wit, SdkError};
use serde::Deserialize;

/// A list of elements present in the query on which one of the extension's directive was applied on their definition.
#[derive(Clone, Copy)]
pub struct QueryElements<'a>(&'a wit::QueryElements);

impl<'a> From<&'a wit::QueryElements> for QueryElements<'a> {
    fn from(value: &'a wit::QueryElements) -> Self {
        Self(value)
    }
}

// is never empty, otherwise we wouldn't call the extension at all
#[allow(clippy::len_without_is_empty)]
impl<'a> QueryElements<'a> {
    /// Number of elements within the query
    pub fn len(&self) -> usize {
        self.0.elements.len()
    }

    /// Iterate over all elements, regardless of the directive they're associated with. Useful if
    /// expect only one directive to be used.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = QueryElement<'a>> + 'a {
        self.0
            .elements
            .iter()
            .enumerate()
            .map(move |(ix, element)| QueryElement { element, ix: ix as u32 })
    }

    /// Iterate over all elements grouped by the directive name.
    pub fn iter_grouped_by_directive_name(
        &self,
    ) -> impl ExactSizeIterator<Item = (&'a str, impl ExactSizeIterator<Item = QueryElement<'a>> + 'a)> + 'a {
        let query = self.0;
        self.0.directive_names.iter().map(|(name, start, end)| {
            let start = *start;
            (
                name.as_str(),
                query.elements[start as usize..*end as usize]
                    .iter()
                    .enumerate()
                    .map(move |(i, element)| QueryElement {
                        element,
                        ix: start + i as u32,
                    }),
            )
        })
    }
}

/// An element of the query on which a directive was applied.
#[derive(Clone, Copy)]
pub struct QueryElement<'a> {
    element: &'a wit::QueryElement,
    pub(super) ix: u32,
}

impl<'a> QueryElement<'a> {
    /// Site, where and with which arguments, of the directive associated with this element.
    /// The provided arguments will exclude anything that depend on response data such as
    /// `FieldSet`.
    pub fn site(&self) -> DirectiveSite<'a> {
        (&self.element.site).into()
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.element.arguments).map_err(Into::into)
    }
}
