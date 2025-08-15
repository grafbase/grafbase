use super::*;

#[derive(Clone, Debug)]
pub struct Extension {
    /// Name of the extension within the federated graph. It does NOT necessarily matches the extension's name
    /// in its manifest, see the `id` field for this.
    pub(crate) enum_value_id: EnumValueId,
    pub url: StringId,
    pub(crate) schema_directives: Vec<ExtensionLinkSchemaDirective>,
}

impl FederatedGraph {
    pub(crate) fn push_extension(&mut self, extension: Extension) -> ExtensionId {
        let id = self.extensions.len().into();
        self.extensions.push(extension);
        id
    }
}
