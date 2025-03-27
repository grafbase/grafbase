use crate::{cbor, sealed::Sealed, types::authorization::private::QueryElementOrResponseItem, wit, SdkError};
use serde::Deserialize;

use super::QueryElementId;

/// A list of elements present in the query on which one of the extension's directive was applied on their definition.
#[derive(Clone, Copy)]
pub struct ResponseElements<'a>(&'a wit::ResponseElements);

impl<'a> From<&'a wit::ResponseElements> for ResponseElements<'a> {
    fn from(value: &'a wit::ResponseElements) -> Self {
        Self(value)
    }
}

// is never empty, otherwise we wouldn't call the extension at all
#[allow(clippy::len_without_is_empty)]
impl<'a> ResponseElements<'a> {
    /// Number of elements within the query
    pub fn len(&self) -> usize {
        self.0.elements.len()
    }

    /// Iterate over all elements, regardless of the directive they're associated with. Useful if
    /// expect only one directive to be used.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = ResponseElement<'a>> + 'a {
        (*self).into_iter()
    }

    /// Iterate over all elements grouped by the directive name.
    pub fn iter_grouped_by_directive_name(
        &self,
    ) -> impl ExactSizeIterator<Item = (&'a str, impl ExactSizeIterator<Item = ResponseElement<'a>> + 'a)> + 'a {
        let resp = self.0;
        let items = &resp.items;
        self.0.directive_names.iter().map(move |(name, start, end)| {
            let start = *start;
            (
                name.as_str(),
                resp.elements[start as usize..*end as usize]
                    .iter()
                    .map(move |inner| ResponseElement {
                        items,
                        query_element_id: QueryElementId(inner.query_element_id),
                        items_range: inner.items_range,
                    }),
            )
        })
    }
}

impl<'a> IntoIterator for ResponseElements<'a> {
    type Item = ResponseElement<'a>;
    type IntoIter = ResponseElementsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ResponseElementsIterator {
            items: &self.0.items,
            iter: self.0.elements.iter(),
        }
    }
}

/// Iterator over the elements of the query on which a directive was applied.
pub struct ResponseElementsIterator<'a> {
    items: &'a [Vec<u8>],
    iter: std::slice::Iter<'a, wit::ResponseElement>,
}

impl ExactSizeIterator for ResponseElementsIterator<'_> {}

impl<'a> Iterator for ResponseElementsIterator<'a> {
    type Item = ResponseElement<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let items = self.items;
        self.iter.next().map(move |inner| ResponseElement {
            items,
            query_element_id: QueryElementId(inner.query_element_id),
            items_range: inner.items_range,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An element of the query on which a directive was applied.
#[derive(Clone, Copy)]
pub struct ResponseElement<'a> {
    items: &'a [Vec<u8>],
    query_element_id: QueryElementId,
    items_range: (u32, u32),
}

impl<'a> ResponseElement<'a> {
    /// When a directive requires response data, it'll be processed in two stages:
    /// - authorize_query will first receive the query element and all the arguments that do not
    ///   depend on response data. This allows fetching any relevant data from an external service.
    ///   Any data you need for later must be kept in the state vector.
    /// - authorize_response will receive the response data, but nothing else. Only a
    ///   `QueryElementId` is provided to allow finding any relevant data in the state vector.
    pub fn query_element_id(&self) -> QueryElementId {
        self.query_element_id
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn items(&self) -> impl ExactSizeIterator<Item = ResponseItem<'a>> {
        let (start, end) = self.items_range;
        self.items[start as usize..end as usize]
            .iter()
            .enumerate()
            .map(move |(offset, bytes)| ResponseItem {
                bytes,
                ix: start + offset as u32,
            })
    }
}

impl Sealed for ResponseItem<'_> {}
impl QueryElementOrResponseItem for ResponseItem<'_> {
    fn ix(&self) -> u32 {
        self.ix
    }
}

/// Represents a single item, object or field, that is subject to an authorization rule with the
/// data requested by the directive.
pub struct ResponseItem<'a> {
    bytes: &'a [u8],
    pub(in crate::types) ix: u32,
}

impl<'a> ResponseItem<'a> {
    /// Arguments that depend on response data will be provided here. All other arguments will only
    /// be provided in the `authorize_query()` step today.
    pub fn directive_arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        cbor::from_slice(self.bytes).map_err(Into::into)
    }
}
