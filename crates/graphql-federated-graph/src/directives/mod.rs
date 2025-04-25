mod authorized;
mod complexity_control;
mod deprecated;
mod extension;
mod federation;
mod require;

use crate::{ListSize, StringId, SubgraphId, Value};

pub use self::{
    complexity_control::{CostDirective, ListSizeDirective},
    deprecated::DeprecatedDirective,
    require::RequireDirective,
};
pub use authorized::*;
pub use extension::*;
pub use federation::*;

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Directive {
    Authenticated,
    CompositeLookup {
        graph: SubgraphId,
    },
    CompositeRequire {
        graph: SubgraphId,
        field: StringId,
    },
    Deprecated {
        reason: Option<StringId>,
    },
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
    Authorized(AuthorizedDirective),
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
    pub fn as_join_field(&self) -> Option<&JoinFieldDirective> {
        match self {
            Directive::JoinField(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_join_field_mut(&mut self) -> Option<&mut JoinFieldDirective> {
        match self {
            Directive::JoinField(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_join_type(&self) -> Option<&JoinTypeDirective> {
        match self {
            Directive::JoinType(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_join_union_member(&self) -> Option<&JoinUnionMemberDirective> {
        match self {
            Directive::JoinUnionMember(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_extension_directive(&self) -> Option<&ExtensionDirective> {
        match self {
            Directive::ExtensionDirective(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_join_implements(&self) -> Option<&JoinImplementsDirective> {
        match self {
            Directive::JoinImplements(d) => Some(d),
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
