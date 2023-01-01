use crate::{registry::ParentRelation, utils::to_lower_camelcase};
use case::CaseExt;
use dynaql_parser::types::TypeDefinition;

use super::NumericFieldKind;

pub const PAGINATION_INPUT_ARG_FIRST: &str = "first";
pub const PAGINATION_INPUT_ARG_LAST: &str = "last";
pub const PAGINATION_INPUT_ARG_BEFORE: &str = "before";
pub const PAGINATION_INPUT_ARG_AFTER: &str = "after";

pub const INPUT_ARG_BY: &str = "by";
pub const INPUT_ARG_INPUT: &str = "input";

pub const INPUT_FIELD_RELATION_CREATE: &str = "create";
pub const INPUT_FIELD_RELATION_LINK: &str = "link";
pub const INPUT_FIELD_RELATION_UNLINK: &str = "unlink";
pub const INPUT_FIELD_NUM_OP_SET: &str = "set";
pub const INPUT_FIELD_NUM_OP_INCREMENT: &str = "increment";
pub const INPUT_FIELD_NUM_OP_DECREMENT: &str = "decrement";

pub struct MetaNames;

/// Defines the names used by the different generated types.
/// It looks a bit silly to centralize names, but they're part of the public API and they SHOULD be
/// consistent.
impl MetaNames {
    // FIXME: Several places used to_camel() but not everywhere... Do we want to enforce it?
    pub fn model(model_type_definition: &TypeDefinition) -> String {
        model_type_definition.name.node.to_camel()
    }

    pub fn mutation_create(model_type_definition: &TypeDefinition) -> String {
        to_lower_camelcase(format!("{}Create", Self::model(model_type_definition)))
    }

    pub fn mutation_update(model_type_definition: &TypeDefinition) -> String {
        to_lower_camelcase(format!("{}Update", Self::model(model_type_definition)))
    }

    pub fn mutation_create_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}CreatePayload", Self::model(model_type_definition))
    }

    pub fn mutation_update_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}UpdatePayload", Self::model(model_type_definition))
    }

    /// Defines
    /// - without parent, the create mutation input type name.
    /// - with parent, the nested input type name to create said type when creating the parent.
    pub fn create_input_type(
        model_type_definition: &TypeDefinition,
        maybe_parent_relation: Option<&ParentRelation<'_>>,
    ) -> String {
        format!(
            "{}CreateInput",
            maybe_parent_relation
                .map(|parent_relation| Self::relation_prefix(parent_relation, model_type_definition))
                .unwrap_or_else(|| Self::model(model_type_definition))
        )
    }

    pub fn update_input_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}UpdateInput", Self::model(model_type_definition))
    }

    /// For a given relation, one can either link to an existing object or create a new one.
    pub fn create_relation_input_type(
        parent_relation: &ParentRelation<'_>,
        field_model_type_definition: &TypeDefinition,
    ) -> String {
        format!(
            "{}CreateRelationInput",
            Self::relation_prefix(parent_relation, field_model_type_definition)
        )
    }

    /// For a given relation, one can either change the (un)link to an existing object or create a new one
    pub fn update_relation_input_type(
        parent_relation: &ParentRelation<'_>,
        field_model_type_definition: &TypeDefinition,
    ) -> String {
        format!(
            "{}UpdateRelationInput",
            Self::relation_prefix(parent_relation, field_model_type_definition)
        )
    }

    pub fn numerical_operation(kind: &NumericFieldKind) -> String {
        format!("{}OperationsInput", kind.as_str())
    }

    /// Prefix used for any input/output type created for a relation.
    fn relation_prefix(parent_relation: &ParentRelation<'_>, field_model_type_definition: &TypeDefinition) -> String {
        format!(
            "{}{}{}",
            &Self::model(parent_relation.model_type_definition),
            &parent_relation.meta.name.to_camel(),
            Self::model(field_model_type_definition)
        )
    }
}
