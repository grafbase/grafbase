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

    /// Url of the subgraph.
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

impl Default for Contract {
    fn default() -> Self {
        Self::new()
    }
}

impl Contract {
    /// Create a new contact with the appropriate capacity which should match the number of
    /// directives.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(wit::Contract {
            accessible: Vec::with_capacity(capacity),
            accessible_by_default: true,
            subgraphs: Vec::new(),
        })
    }

    /// Creates a new contract
    pub fn new() -> Self {
        Self(wit::Contract {
            accessible: Vec::new(),
            accessible_by_default: true,
            subgraphs: Vec::new(),
        })
    }

    /// Whether the schema elements are accessible by default.
    pub fn accessible_by_default(&mut self, accessible: bool) {
        self.0.accessible_by_default = accessible;
    }

    /// Set the accessibility of a directive.
    pub fn accessible(&mut self, directive: ContractDirective<'_>, accessible: bool) {
        if self.0.accessible.len() < directive.index as usize {
            // Extend the vector with `false` values until it reaches the required index.
            self.0.accessible.resize(directive.index as usize + 1, -1);
        }
        self.0.accessible[directive.index as usize] = accessible as i8 - 1
    }

    /// Set the accessibility of a directive with a priority value. The higher the priority, the
    /// latter this directive will be taken into account, overriding any previous ones.
    ///
    /// A positive value indicates that the directive is accessible, so `[0, 127]` means accessible
    /// but `[-128, -1]` doesn't. The absolute value is used as the priority after shifting the
    /// positive values up by one. So both `-1` and `0` have the same priority, `-2` and `1` also
    /// and so forth up to `-128` and `127`.
    pub fn accessible_with_priority(&mut self, directive: ContractDirective<'_>, accessible: i8) {
        if self.0.accessible.len() < directive.index as usize {
            // Extend the vector with `false` values until it reaches the required index.
            self.0.accessible.resize(directive.index as usize + 1, -1);
        }
        self.0.accessible[directive.index as usize] = accessible;
    }

    /// Add a subgraph update to the subgraph. Unchanged subgraph do not need to be provided.
    /// The number of subgraphs and their name cannot be changed.
    pub fn subgraph(&mut self, subgraph: GraphqlSubgraph) {
        self.0.subgraphs.push(subgraph.0);
    }
}

impl From<Contract> for wit::Contract {
    fn from(contract: Contract) -> Self {
        contract.0
    }
}
