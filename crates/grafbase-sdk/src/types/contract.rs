use crate::{types::Directive, wit};

/// Represents a GraphQL subgraph with its name and URL.
pub struct GraphqlSubgraph(wit::GraphqlSubgraph);

impl From<wit::GraphqlSubgraph> for GraphqlSubgraph {
    fn from(subgraph: wit::GraphqlSubgraph) -> Self {
        Self(subgraph)
    }
}

impl GraphqlSubgraph {
    /// Name of the subgraph.
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// URL of the subgraph.
    pub fn url(&self) -> &str {
        self.0.url.as_str()
    }

    /// Mutable access the URL of the subgraph.
    pub fn url_mut(&mut self) -> &mut String {
        &mut self.0.url
    }
}

/// Represents a directive associated with a contract.
#[derive(Clone, Copy)]
pub struct ContractDirective<'a> {
    index: u32,
    directive: Directive<'a>,
}

impl<'a> From<(usize, &'a wit::Directive)> for ContractDirective<'a> {
    fn from((index, directive): (usize, &'a wit::Directive)) -> Self {
        Self {
            index: index as u32,
            directive: directive.into(),
        }
    }
}

impl<'a> std::ops::Deref for ContractDirective<'a> {
    type Target = Directive<'a>;
    fn deref(&self) -> &Self::Target {
        &self.directive
    }
}

/// Contract that must be applied on the schema.
pub struct Contract(wit::Contract);

impl Contract {
    /// Create a new contact with the appropriate capacity which should match the number of
    /// contract directives.
    pub fn new(directives: &[ContractDirective<'_>], accessible_by_default: bool) -> Self {
        Self(wit::Contract {
            accessible: vec![accessible_by_default as i8 - 1; directives.len()],
            accessible_by_default,
            hide_unreachable_types: true,
            subgraphs: Vec::new(),
        })
    }

    /// Set the accessibility of a directive with the lowest priority.
    pub fn accessible(&mut self, directive: ContractDirective<'_>, accessible: bool) -> &mut Self {
        let inaccessible_mask = accessible as i8 - 1; // 0xFF if false, 0x00 if true
        self.accessible_with_priority(directive, (inaccessible_mask & -2) | (!inaccessible_mask & 1))
    }

    /// Set the accessibility of a directive with the highest priority.
    pub fn override_accessible(&mut self, directive: ContractDirective<'_>, accessible: bool) -> &mut Self {
        self.accessible_with_priority(directive, i8::MAX.wrapping_add(!accessible as i8))
    }

    /// Set the accessibility of a directive with a priority value. The higher the priority, the
    /// latter this directive will be taken into account, overriding any previous ones.
    ///
    /// A positive value indicates that the directive is accessible, so `[0, 127]` means accessible
    /// but `[-128, -1]` doesn't. The absolute value is used as the priority after shifting the
    /// positive values up by one. So both `-1` and `0` have the same priority, `-2` and `1` also
    /// and so forth up to `-128` and `127`.
    ///
    /// The default behavior is encoded as `-1` and `0`, so to override the default accessibility
    /// of the contract, you must use at least `-2` or `1`.
    pub fn accessible_with_priority(&mut self, directive: ContractDirective<'_>, accessible: i8) -> &mut Self {
        self.0.accessible[directive.index as usize] = accessible;
        self
    }

    /// Whether to hide types that are not reachable from the root type. Defaults to true.
    pub fn hide_unreachable_types(&mut self, hide: bool) -> &mut Self {
        self.0.hide_unreachable_types = hide;
        self
    }

    /// Add a subgraph update to the subgraph. Unchanged subgraph do not need to be provided.
    /// The number of subgraphs and their name cannot be changed.
    pub fn subgraph(&mut self, subgraph: GraphqlSubgraph) -> &mut Self {
        self.0.subgraphs.push(subgraph.0);
        self
    }
}

impl From<Contract> for wit::Contract {
    fn from(contract: Contract) -> Self {
        contract.0
    }
}
