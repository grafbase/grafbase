use serde::Deserialize;

use crate::{
    types::DirectiveSite,
    wit::{self},
    SdkError,
};

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
            .map(move |(ix, site)| QueryElement { site, ix: ix as u32 })
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
                    .map(move |(i, site)| QueryElement {
                        site,
                        ix: start + i as u32,
                    }),
            )
        })
    }
}

/// An element of the query on which a directive was applied.
#[derive(Clone, Copy)]
pub struct QueryElement<'a> {
    site: &'a wit::DirectiveSite,
    pub(super) ix: u32,
}

impl<'a> QueryElement<'a> {
    /// Site, where and with which arguments, of the directive associated with this element.
    /// The provided arguments will exclude anything that depend on response data such as
    /// `FieldSet`.
    pub fn site(&self) -> DirectiveSite<'a> {
        self.site.into()
    }
}

/// A list of elements present in the response on which one of the extension's directive was applied on their definition.
#[derive(Clone, Copy)]
pub struct ResponseElements<'a> {
    pub(crate) query: &'a wit::QueryElements,
    pub(crate) resp: &'a wit::ResponseElements,
}

// is never empty, otherwise we wouldn't call the extension at all
#[allow(clippy::len_without_is_empty)]
impl<'a> ResponseElements<'a> {
    /// Number of elements within the response
    pub fn len(&self) -> usize {
        self.resp.elements.len()
    }

    /// Iterate over all elements, regardless of the directive they're associated with. Useful if
    /// expect only one directive to be used.
    pub fn iter_grouped_by_query_element(
        &self,
    ) -> impl Iterator<
        Item = (
            QueryElement<'a>,
            impl ExactSizeIterator<Item = ResponseElement<'a>> + 'a,
        ),
    > + 'a {
        let query = self.query;
        let resp = self.resp;

        resp.query_to_resp.iter().map(|&(query_ix, resp_start, resp_end)| {
            let query_element = QueryElement {
                site: &query.elements[query_ix as usize],
                ix: query_ix,
            };
            let resp_items = resp.elements[resp_start as usize..resp_end as usize]
                .iter()
                .enumerate()
                .map(move |(i, data)| ResponseElement {
                    data,
                    ix: resp_start + i as u32,
                });
            (query_element, resp_items)
        })
    }

    /// Iterate over all elements grouped by the directive name.
    pub fn iter_grouped_by_directive_name_then_query_element(
        &self,
    ) -> impl Iterator<
        Item = (
            &'a str,
            impl Iterator<
                    Item = (
                        QueryElement<'a>,
                        impl ExactSizeIterator<Item = ResponseElement<'a>> + 'a,
                    ),
                > + 'a,
        ),
    > + 'a {
        let query = self.query;
        let resp = self.resp;

        resp.directive_names.iter().map(|(name, start, end)| {
            (
                name.as_str(),
                resp.query_to_resp[*start as usize..*end as usize]
                    .iter()
                    .map(|&(query_ix, resp_start, resp_end)| {
                        let query_element = QueryElement {
                            site: &query.elements[query_ix as usize],
                            ix: query_ix,
                        };
                        let resp_items = resp.elements[resp_start as usize..resp_end as usize]
                            .iter()
                            .enumerate()
                            .map(move |(i, data)| ResponseElement {
                                data,
                                ix: resp_start + i as u32,
                            });
                        (query_element, resp_items)
                    }),
            )
        })
    }
}

/// A response element containing a directive site and associated items
#[derive(Clone, Copy)]
pub struct ResponseElement<'a> {
    data: &'a [u8],
    pub(super) ix: u32,
}

impl<'a> ResponseElement<'a> {
    /// Get the response items
    pub fn data<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(self.data).map_err(Into::into)
    }
}
