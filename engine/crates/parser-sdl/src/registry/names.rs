use std::borrow::Cow;

use case::CaseExt;
pub use engine::names::*;
use engine::registry::NamedType;
use engine_parser::types::TypeDefinition;

use crate::{registry::ParentRelation, utils::to_lower_camelcase};

pub const PAGINATION_INPUT_ARG_FIRST: &str = "first";
pub const PAGINATION_INPUT_ARG_LAST: &str = "last";
pub const PAGINATION_INPUT_ARG_BEFORE: &str = "before";
pub const PAGINATION_INPUT_ARG_AFTER: &str = "after";
pub const PAGINATION_INPUT_ARG_ORDER_BY: &str = "orderBy";

// Pagination must be consistent for every query/mutation hence the search fields
pub const PAGINATION_FIELD_EDGES: &str = "edges";
pub const PAGINATION_FIELD_SEARCH_INFO: &str = "searchInfo";
pub const PAGINATION_FIELD_PAGE_INFO: &str = "pageInfo";
pub const PAGINATION_FIELD_EDGE_NODE: &str = "node";
pub const PAGINATION_FIELD_EDGE_CURSOR: &str = "cursor";
pub const PAGINATION_FIELD_EDGE_SEARCH_SCORE: &str = "score";

pub const SEARCH_INFO_TYPE: &str = "SearchInfo";
pub const SEARCH_INFO_FIELD_TOTAL_HITS: &str = "totalHits";

pub const PAGE_INFO_TYPE: &str = "PageInfo";
pub const PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE: &str = "hasPreviousPage";
pub const PAGE_INFO_FIELD_HAS_NEXT_PAGE: &str = "hasNextPage";
pub const PAGE_INFO_FIELD_START_CURSOR: &str = "startCursor";
pub const PAGE_INFO_FIELD_END_CURSOR: &str = "endCursor";

pub const INPUT_ARG_BY: &str = "by";
pub const INPUT_ARG_INPUT: &str = "input";
pub const INPUT_ARG_FILTER: &str = "filter";
pub const INPUT_ARG_QUERY: &str = "query";
pub const INPUT_ARG_FIELDS: &str = "fields";

pub const INPUT_FIELD_RELATION_CREATE: &str = "create";
pub const INPUT_FIELD_RELATION_LINK: &str = "link";
pub const INPUT_FIELD_RELATION_UNLINK: &str = "unlink";
pub const INPUT_FIELD_NUM_OP_SET: &str = "set";
pub const INPUT_FIELD_NUM_OP_INCREMENT: &str = "increment";
pub const INPUT_FIELD_NUM_OP_DECREMENT: &str = "decrement";
pub const INPUT_FIELD_NUM_OP_MULTIPLY: &str = "multiply";
pub const INPUT_FIELD_NUM_OP_DIVIDE: &str = "divide";

pub const INPUT_FIELD_COLLECTION_OP_APPEND: &str = "append";
pub const INPUT_FIELD_COLLECTION_OP_PREPEND: &str = "prepend";
pub const INPUT_FIELD_COLLECTION_OP_DELETE_ELEM: &str = "deleteElem";
pub const INPUT_FIELD_COLLECTION_OP_DELETE_KEY: &str = "deleteKey";
pub const INPUT_FIELD_COLLECTION_OP_DELETE_AT_PATH: &str = "deleteAtPath";

pub const INPUT_FIELD_OP_EQ: &str = "eq";
pub const INPUT_FIELD_OP_NE: &str = "ne";
pub const INPUT_FIELD_OP_GT: &str = "gt";
pub const INPUT_FIELD_OP_LT: &str = "lt";
pub const INPUT_FIELD_OP_GTE: &str = "gte";
pub const INPUT_FIELD_OP_LTE: &str = "lte";
pub const INPUT_FIELD_OP_IN: &str = "in";
pub const INPUT_FIELD_OP_NIN: &str = "nin";
pub const INPUT_FIELD_OP_NOT: &str = "not";
pub const INPUT_FIELD_OP_CONTAINS: &str = "contains";
pub const INPUT_FIELD_OP_CONTAINED: &str = "contained";
pub const INPUT_FIELD_OP_OVERLAPS: &str = "overlaps";

pub const ORDER_BY_DIRECTION: &str = "OrderByDirection";
pub const ORDER_BY_ASC: &str = "ASC";
pub const ORDER_BY_DESC: &str = "DESC";

pub struct MetaNames;

/// CONVENTIONS:
///     - Input must be suffixed by "input"
///     - The model name must be the prefix
///     - All types/inputs must be CamelCase, all fields must be camelCase.
impl MetaNames {
    pub fn entity_type(model_type_definition: &TypeDefinition) -> String {
        // FIXME: At several places the lowercase for the id & entity_type is
        // used. A single code path should handle that.
        MetaNames::model(model_type_definition).to_lowercase()
    }

    pub fn model(model_type_definition: &TypeDefinition) -> String {
        MetaNames::model_name_from_str(model_type_definition.name.node.as_str())
    }

    pub fn model_name_from_str(name: &str) -> String {
        name.to_camel()
    }

    pub fn collection_by_str(type_name: &str) -> String {
        format!("{type_name}Collection")
    }

    pub fn collection(model_type_definition: &TypeDefinition) -> String {
        Self::collection_by_str(&Self::model(model_type_definition))
    }

    //
    // GET
    //
    pub fn by_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}ByInput", Self::model(model_type_definition))
    }

    //
    // SEARCH
    //
    pub fn query_search(model_type_definition: &TypeDefinition) -> String {
        to_lower_camelcase(format!("{}Search", Self::model(model_type_definition)))
    }

    pub fn search_filter_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}SearchFilterInput", Self::model(model_type_definition))
    }

    pub fn search_scalar_list_filter_input(scalar: &str) -> String {
        format!("{scalar}ListSearchFilterInput")
    }

    pub fn search_scalar_filter_input(scalar: &str, nullable: bool) -> String {
        let scalar = if nullable {
            Cow::Owned(format!("{scalar}OrNull"))
        } else {
            Cow::Borrowed(scalar)
        };
        format!("{scalar}SearchFilterInput")
    }

    pub fn search_connection_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}SearchConnection", Self::model(model_type_definition))
    }

    pub fn search_edge_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}SearchEdge", Self::model(model_type_definition))
    }

    //
    // COLLECTION
    //
    pub fn query_collection_by_str(type_name: &str) -> String {
        to_lower_camelcase(Self::collection_by_str(type_name))
    }

    pub fn query_collection(model_type_definition: &TypeDefinition) -> String {
        to_lower_camelcase(Self::collection(model_type_definition))
    }

    pub fn pagination_connection_type_by_str(type_name: &str) -> String {
        format!("{type_name}Connection")
    }

    pub fn pagination_connection_type(model_type_definition: &TypeDefinition) -> String {
        Self::pagination_connection_type_by_str(&Self::model(model_type_definition))
    }

    pub fn pagination_edge_type_by_str(type_name: &str) -> String {
        format!("{type_name}Edge")
    }

    pub fn pagination_edge_type(model_type_definition: &TypeDefinition) -> String {
        Self::pagination_edge_type_by_str(&Self::model(model_type_definition))
    }

    pub fn pagination_orderby_input_by_str(type_name: &str) -> NamedType<'static> {
        format!("{type_name}OrderByInput").into()
    }

    pub fn pagination_orderby_input(model_type_definition: &TypeDefinition) -> NamedType<'static> {
        // FIXME: Should have been postCollectionOrderByInput instead of postOrderByInput...
        Self::pagination_orderby_input_by_str(Self::model(model_type_definition).as_str())
    }

    pub fn collection_filter_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}FilterInput", Self::query_collection(model_type_definition)).to_camel()
    }

    pub fn collection_scalar_filter_input(scalar: &str) -> String {
        format!("{scalar}CollectionFilterInput")
    }

    //
    // DELETE
    //

    pub fn mutation_delete_by_str(type_name: &str) -> String {
        to_lower_camelcase(format!("{type_name}Delete"))
    }

    pub fn mutation_delete(model_type_definition: &TypeDefinition) -> String {
        Self::mutation_delete_by_str(&Self::model(model_type_definition))
    }

    pub fn mutation_delete_many_by_str(type_name: &str) -> String {
        let delete = Self::mutation_delete_by_str(type_name);
        format!("{delete}Many")
    }

    pub fn mutation_delete_many(model_type_definition: &TypeDefinition) -> String {
        format!("{}Many", Self::mutation_delete(model_type_definition))
    }

    pub fn delete_payload_type_by_str(type_name: &str) -> String {
        let mutation_delete = Self::mutation_delete_by_str(type_name);
        format!("{mutation_delete}Payload").to_camel()
    }

    pub fn delete_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_delete(model_type_definition)).to_camel()
    }

    pub fn delete_many_payload_type_by_str(model_type_definition: &str) -> String {
        let delete_many = Self::mutation_delete_many_by_str(model_type_definition);
        format!("{delete_many}Payload").to_camel()
    }

    pub fn delete_many_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_delete_many(model_type_definition)).to_camel()
    }

    pub fn delete_many_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}Input", Self::mutation_delete_many(model_type_definition)).to_camel()
    }

    //
    // CREATE
    //
    pub fn mutation_create_by_str(type_name: &str) -> String {
        to_lower_camelcase(format!("{type_name}Create"))
    }

    pub fn mutation_create(model_type_definition: &TypeDefinition) -> String {
        Self::mutation_create_by_str(&Self::model(model_type_definition))
    }

    pub fn create_payload_type_by_str(type_name: &str) -> String {
        let mutation_create = Self::mutation_create_by_str(type_name);
        format!("{mutation_create}Payload").to_camel()
    }

    pub fn create_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_create(model_type_definition)).to_camel()
    }

    pub fn mutation_create_many_by_str(type_name: &str) -> String {
        format!("{}Many", Self::mutation_create_by_str(type_name))
    }

    pub fn mutation_create_many(model_type_definition: &TypeDefinition) -> String {
        format!("{}Many", Self::mutation_create(model_type_definition))
    }

    pub fn create_many_payload_type_by_str(type_name: &str) -> String {
        let create_many = Self::mutation_create_many_by_str(type_name);
        format!("{create_many}Payload").to_camel()
    }

    pub fn create_many_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_create_many(model_type_definition)).to_camel()
    }

    pub fn create_many_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}Input", Self::mutation_create_many(model_type_definition)).to_camel()
    }

    pub fn create_input_by_str(type_name: &str, parent_type_name: Option<&str>) -> String {
        match parent_type_name {
            None => format!("{}Input", Self::mutation_create_by_str(type_name)).to_camel(),
            Some(parent_relation) => format!("{parent_relation}Create{type_name}",),
        }
    }

    /// Defines
    /// - without parent, the create mutation input type name.
    /// - with parent, the nested input type name to create said type when creating the parent.
    pub fn create_input(
        model_type_definition: &TypeDefinition,
        maybe_parent_relation: Option<&ParentRelation<'_>>,
    ) -> String {
        let parent_relation = maybe_parent_relation.map(|parent| Self::relation_prefix(parent));
        Self::create_input_by_str(&Self::model(model_type_definition), parent_relation.as_deref())
    }

    /// For a given relation, one can either link to an existing object or create a new one.
    pub fn create_relation_input(
        parent_relation: &ParentRelation<'_>,
        field_model_type_definition: &TypeDefinition,
    ) -> String {
        format!(
            "{}Create{}Relation",
            Self::relation_prefix(parent_relation),
            Self::model(field_model_type_definition)
        )
    }

    //
    // UPDATE
    //
    pub fn mutation_update_by_str(type_name: &str) -> String {
        to_lower_camelcase(format!("{type_name}Update"))
    }

    pub fn mutation_update(model_type_definition: &TypeDefinition) -> String {
        Self::mutation_update_by_str(&Self::model(model_type_definition))
    }

    pub fn update_payload_type_by_str(type_name: &str) -> String {
        let mutation_update = Self::mutation_update_by_str(type_name);
        format!("{mutation_update}Payload").to_camel()
    }

    pub fn update_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_update(model_type_definition)).to_camel()
    }

    pub fn update_input_by_str(type_name: &str) -> String {
        let mutation_update = Self::mutation_update_by_str(type_name);
        format!("{mutation_update}Input").to_camel()
    }

    pub fn update_input(model_type_definition: &TypeDefinition) -> String {
        Self::update_input_by_str(&Self::model(model_type_definition))
    }

    pub fn mutation_update_many_by_str(type_name: &str) -> String {
        format!("{}Many", Self::mutation_update_by_str(type_name))
    }

    pub fn mutation_update_many(model_type_definition: &TypeDefinition) -> String {
        format!("{}Many", Self::mutation_update(model_type_definition))
    }

    pub fn update_many_payload_type_by_str(type_name: &str) -> String {
        format!("{}Payload", Self::mutation_update_many_by_str(type_name)).to_camel()
    }

    pub fn update_many_payload_type(model_type_definition: &TypeDefinition) -> String {
        format!("{}Payload", Self::mutation_update_many(model_type_definition)).to_camel()
    }

    pub fn update_many_input(model_type_definition: &TypeDefinition) -> String {
        format!("{}Input", Self::mutation_update_many(model_type_definition)).to_camel()
    }

    /// For a given relation, one can either change the (un)link to an existing object or create a new one
    pub fn update_relation_input(
        parent_relation: &ParentRelation<'_>,
        field_model_type_definition: &TypeDefinition,
    ) -> String {
        format!(
            "{}Update{}Relation",
            Self::relation_prefix(parent_relation),
            Self::model(field_model_type_definition)
        )
    }

    /// Prefix used for any input/output type created for a relation.
    fn relation_prefix(parent_relation: &ParentRelation<'_>) -> String {
        parent_relation.meta.name.to_camel()
    }

    // Name of the struct that looks up a model by a composite index.
    pub fn nested_order_by_input(model_name: &str, constraint_name: &str) -> String {
        format!("{model_name}By{}", constraint_name.to_camel())
    }
}
