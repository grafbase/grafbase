#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum ExtensionDirectiveType {
    FieldResolver,
    Resolver,
    SelectionSetResolver,
    Authorization {
        grouping: AuthorizationGrouping,
    },
    #[default]
    Unknown,
}

impl ExtensionDirectiveType {
    pub fn is_field_resolver(&self) -> bool {
        matches!(self, ExtensionDirectiveType::FieldResolver)
    }

    pub fn is_resolver(&self) -> bool {
        matches!(self, ExtensionDirectiveType::Resolver)
    }

    pub fn is_selection_set_resolver(&self) -> bool {
        matches!(self, ExtensionDirectiveType::SelectionSetResolver)
    }

    pub fn is_authorization(&self) -> bool {
        matches!(self, ExtensionDirectiveType::Authorization { .. })
    }
}

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags::bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    pub struct AuthorizationGrouping: u8 {
        const Subgraph = 1;
    }
}

impl AuthorizationGrouping {
    pub fn has_subgraph(&self) -> bool {
        self.contains(AuthorizationGrouping::Subgraph)
    }
}
