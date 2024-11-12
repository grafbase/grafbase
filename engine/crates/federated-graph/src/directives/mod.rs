mod complexity_control;
mod deprecated;

pub use self::{
    complexity_control::{CostDirective, ListSizeDirective},
    deprecated::DeprecatedDirective,
};

#[cfg(test)]
/// Helper for tests
fn parse_directive<T>(input: &str) -> Result<T, cynic_parser_deser::Error>
where
    T: cynic_parser_deser::ValueDeserializeOwned,
{
    let doc = directive_test_document(input);
    parse_from_test_document(&doc)
}

#[cfg(test)]
/// Helper for tests where the directive has a lifetime
///
/// Should be used with parse_from_test_document
fn directive_test_document(directive: &str) -> cynic_parser::TypeSystemDocument {
    cynic_parser::parse_type_system_document(&format!("type Object {directive} {{name: String}}")).unwrap()
}

#[cfg(test)]
/// Helper for tests where the directive has a lifetime
///
/// Should be used with the document from directive_test_document
fn parse_from_test_document<'a, T>(doc: &'a cynic_parser::TypeSystemDocument) -> Result<T, cynic_parser_deser::Error>
where
    T: cynic_parser_deser::ValueDeserialize<'a>,
{
    use cynic_parser::type_system::Definition;
    use cynic_parser_deser::ConstDeserializer;
    let Definition::Type(definition) = doc.definitions().next().unwrap() else {
        unreachable!()
    };
    definition.directives().next().unwrap().deserialize::<T>()
}
