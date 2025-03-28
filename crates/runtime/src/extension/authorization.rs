use std::ops::Range;

use engine_schema::DirectiveSite;
use error::GraphqlError;
use extension_catalog::ExtensionId;

#[derive(Clone, Debug)]
pub struct QueryElement<'a, A> {
    pub site: DirectiveSite<'a>,
    pub arguments: A,
}

#[derive(Debug)]
pub enum AuthorizationDecisions {
    GrantAll,
    DenyAll(GraphqlError),
    DenySome {
        element_to_error: Vec<(u32, u32)>,
        errors: Vec<GraphqlError>,
    },
}

pub struct QueryAuthorizationDecisions {
    pub extension_id: ExtensionId,
    pub query_elements_range: Range<usize>,
    pub decisions: AuthorizationDecisions,
}
