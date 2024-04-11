use engine::{registry::MetaInputValue, Positioned};
use engine_parser::types::FieldDefinition;
use indexmap::IndexMap;

pub trait FieldExtension {
    /// Returns true if this field is "synthetic"
    ///
    /// e.g. a field that only exists inside the engine and not in any connected api/db
    ///
    /// Currently this means fields with resolvers or joined fieds, but may be expanded.
    fn is_synthetic_field(&self) -> bool;

    fn converted_arguments(&self) -> IndexMap<String, MetaInputValue>;
}

impl FieldExtension for FieldDefinition {
    fn is_synthetic_field(&self) -> bool {
        self.directives
            .iter()
            .any(|directive| directive.name.node == "join" || directive.name.node == "resolver")
    }

    fn converted_arguments(&self) -> IndexMap<String, MetaInputValue> {
        self.arguments
            .iter()
            .map(|argument| {
                let name = argument.node.name.to_string();
                let input = MetaInputValue::new(name.clone(), argument.node.ty.to_string());

                (name, input)
            })
            .collect()
    }
}

impl FieldExtension for Positioned<FieldDefinition> {
    fn is_synthetic_field(&self) -> bool {
        self.node.is_synthetic_field()
    }

    fn converted_arguments(&self) -> IndexMap<String, MetaInputValue> {
        self.node.converted_arguments()
    }
}
