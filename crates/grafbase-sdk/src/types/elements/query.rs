use std::iter::Enumerate;

use crate::{
    SdkError,
    types::{DirectiveSite, authorization::private::QueryElementOrResponseItem},
    wit,
};
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
        (*self).into_iter()
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

impl<'a> IntoIterator for QueryElements<'a> {
    type Item = QueryElement<'a>;
    type IntoIter = QueryElementsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        QueryElementsIterator(self.0.elements.iter().enumerate())
    }
}

/// Iterator over the elements of the query on which a directive was applied.
pub struct QueryElementsIterator<'a>(Enumerate<std::slice::Iter<'a, wit::QueryElement>>);

impl ExactSizeIterator for QueryElementsIterator<'_> {}

impl<'a> Iterator for QueryElementsIterator<'a> {
    type Item = QueryElement<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(move |(ix, element)| QueryElement { element, ix: ix as u32 })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl crate::sealed::Sealed for QueryElement<'_> {}
impl QueryElementOrResponseItem for QueryElement<'_> {
    fn ix(&self) -> u32 {
        self.ix
    }
}

/// An element of the query on which a directive was applied.
#[derive(Clone, Copy)]
pub struct QueryElement<'a> {
    element: &'a wit::QueryElement,
    ix: u32,
}

/// An identifier for a query element. Only relevant for response authorization as data provided in
/// `authorize_query` won't be re-sent in `authorize_response`. So this ID allows finding the
/// relevant data in the custom state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct QueryElementId(pub(super) u32);

impl From<QueryElementId> for u32 {
    fn from(value: QueryElementId) -> u32 {
        value.0
    }
}

impl std::fmt::Display for QueryElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> QueryElement<'a> {
    /// ID of the query element, only relevant for response authorization.
    pub fn id(&self) -> QueryElementId {
        QueryElementId(self.element.id)
    }

    /// Directive site, where and with which arguments, of the directive associated with this element.
    /// The provided arguments will exclude anything that depend on response data such as
    /// `FieldSet`.
    pub fn directive_site(&self) -> DirectiveSite<'a> {
        (&self.element.site).into()
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn directive_arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.element.arguments).map_err(Into::into)
    }
}
