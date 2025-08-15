mod complexity_control;
mod deprecated;
mod extension;
mod federation;
mod is;
mod link;
mod list_size;
mod r#override;
mod require;

use crate::federated_graph::{StringId, SubgraphId, Value};

pub(crate) use self::{
    complexity_control::{CostDirective, ListSizeDirective},
    deprecated::DeprecatedDirective,
    extension::*,
    federation::*,
    is::IsDirective,
    link::*,
    list_size::*,
    r#override::*,
    require::*,
};

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub(crate) enum Directive {
    Authenticated,
    CompositeLookup {
        graph: SubgraphId,
    },
    CompositeDerive {
        graph: SubgraphId,
    },
    CompositeInternal {
        graph: SubgraphId,
    },
    CompositeRequire {
        graph: SubgraphId,
        field: StringId,
    },
    CompositeIs {
        graph: SubgraphId,
        field: StringId,
    },
    Deprecated {
        reason: Option<StringId>,
    },
    OneOf,
    Inaccessible,
    Policy(Vec<Vec<StringId>>),
    RequiresScopes(Vec<Vec<StringId>>),
    Cost {
        weight: i32,
    },
    JoinGraph(JoinGraphDirective),
    JoinField(JoinFieldDirective),
    JoinType(JoinTypeDirective),
    JoinUnionMember(JoinUnionMemberDirective),
    JoinImplements(JoinImplementsDirective),
    Other {
        name: StringId,
        arguments: Vec<(StringId, Value)>,
    },
    ListSize(ListSize),

    ExtensionDirective(ExtensionDirective),
}

impl From<JoinFieldDirective> for Directive {
    fn from(d: JoinFieldDirective) -> Self {
        Self::JoinField(d)
    }
}

impl From<JoinTypeDirective> for Directive {
    fn from(d: JoinTypeDirective) -> Self {
        Self::JoinType(d)
    }
}

impl Directive {
    pub fn as_join_type(&self) -> Option<&JoinTypeDirective> {
        match self {
            Directive::JoinType(d) => Some(d),
            _ => None,
        }
    }
}

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
