//! Implement the Relation Engine
use engine::{indexmap::map::Entry, registry::relations::MetaRelation, Positioned, Value};
use engine_parser::types::{FieldDefinition, Type, TypeKind};
use if_chain::if_chain;
use regex::Regex;

use crate::{
    registry::names::MetaNames,
    rules::{directive::Directive, model_directive::ModelDirective},
    Visitor, VisitorContext,
};

/// Implement the Relation Engine
///
/// We need to define the relation before hand, to do that we have two mecanism
/// working to define relation:
///
/// - Implicit: By having an explicit relation on two modelized node.
/// - Explicit: By having the relation defined by the `@relation` directive
///
/// A relation can only exist between two nodes.
///
/// # Algorithm
///
/// For each modelized node, we go into each fields, for each field:
///
/// -> We pass into the field
/// --> If modelized
/// --> Attribute a relation name based on those two types.
/// --> Store it into the VisitorContext (Will be used to compare between two iteration of a schema
/// if there is a change in relations)
/// --> (Store it into the generated type, need engine work)
pub struct RelationEngine;

pub const RELATION_DIRECTIVE: &str = "relation";
pub const NAME_ARGUMENT: &str = "name";

static NAME_CHARS: &str = r"[_a-zA-Z0-9]";

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(&format!("^{NAME_CHARS}*$")).unwrap();
}

impl RelationEngine {
    /// Can only be safely used after the RelationEngine has parsed the schema.
    pub fn get(ctx: &VisitorContext<'_>, type_name: &str, field: &FieldDefinition) -> Option<MetaRelation> {
        // partial relation because the full relation can only be generated through the full
        // schema parsing. We're only using it as parts of it are correct like the relation name.
        let partial_relation = generate_metarelation(type_name, field);
        ctx.relations.get(&partial_relation.name).cloned().map(|relation| {
            if relation.relation == partial_relation.relation {
                relation
            } else {
                MetaRelation {
                    kind: relation.kind.inverse(),
                    // Getting proper field order from the partial relation
                    relation: partial_relation.relation,
                    ..relation
                }
            }
        })
    }
}

/// Generate a `MetaRelation` if possible
fn generate_metarelation(type_name: &str, field: &FieldDefinition) -> MetaRelation {
    let type_name = type_name.to_string();
    let name = relation_name(field).and_then(|name| match &name.node {
        Value::String(inner) => Some(inner.clone()),
        _ => None,
    });

    let from_field = Type::new(&type_name).expect("Shouldn't fail");
    let to_field = &field.ty.node;
    let from_model = MetaNames::model_name_from_str(from_field.base.to_base_type_str());
    let to_model = MetaNames::model_name_from_str(to_field.base.to_base_type_str());

    MetaRelation::new(name, &from_field, to_field, from_model, to_model)
}

fn relation_name(field: &FieldDefinition) -> Option<&Positioned<engine_value::ConstValue>> {
    field
        .directives
        .iter()
        .find(|directive| directive.node.name.node == RELATION_DIRECTIVE)?
        .node
        .get_argument(NAME_ARGUMENT)
}

impl Directive for RelationEngine {
    fn definition() -> String {
        r#"
        directive @relation(
          """
          The name of the relation
          """
          name: String!
        ) on FIELD_DEFINITION
        "#
        .to_string()
    }
}

impl<'a> Visitor<'a> for RelationEngine {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            // We do check if it's a modelized node
            // TODO: Create an abstraction over it
            if directives.iter().any(|directive| directive.is_model());
            if let TypeKind::Object(object) = &type_definition.node.kind;
            // We do check if it's a modelized node
            then {
                // We iterate over fields that reprensent a relation to check than
                let mut errors = Vec::new();
                for field in &object.fields {
                    if ModelDirective::is_model(ctx, &field.node.ty.node) {
                        let relation = generate_metarelation(&type_definition.node.name.node, &field.node);
                        if !NAME_RE.is_match(&relation.name) {
                            let name = &relation.name;
                            ctx.report_error(
                                vec![relation_name(&field.node).unwrap().pos],
                                format!("Relation names should only contain {NAME_CHARS} but {name} does not"),
                            );
                        }
                        match ctx.relations.entry(relation.name.clone()) {
                            Entry::Vacant(vac) => {
                                vac.insert(relation);
                            }
                            Entry::Occupied(mut oqp) => {
                                if let Err(err) = oqp.get_mut().with(relation) {
                                    errors.push((field.pos, err));
                                }
                            }
                        };
                    }

                }

                for (pos, err) in errors {
                    ctx.report_error(
                    vec![pos],
                    format!("Relations issues: {err}"),
                );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use insta::assert_debug_snapshot;
    use serde_json as _;

    use super::RelationEngine;
    use crate::rules::visitor::{visit, VisitorContext};

    #[test]
    fn one_to_one_relation_monodirectional() {
        let schema = r"
            type Author @model {
                id: ID!
            }

            type Post @model {
                id: ID!
                publishedBy: Author
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            !ctx.relations.iter().next().unwrap().1.birectional,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_one_relation_bidirectionnal() {
        let schema = r"
            type Author @model {
                id: ID!
                published: Post
            }

            type Post @model {
                id: ID!
                publishedBy: Author
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            ctx.relations.iter().next().unwrap().1.birectional,
            "Should be birectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_monodirectional_1() {
        let schema = r"
            type Author @model {
                id: ID!
            }

            type Post @model {
                id: ID!
                publishedBy: [Author]
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            !ctx.relations.iter().next().unwrap().1.birectional,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_monodirectional_2() {
        let schema = r"
            type Author @model {
                id: ID!
                posts: [Post]
            }

            type Post @model {
                id: ID!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            !ctx.relations.iter().next().unwrap().1.birectional,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_bidirectional_1() {
        let schema = r"
            type Author @model {
                id: ID!
                post: Post!
            }

            type Post @model {
                id: ID!
                publishedBy: [Author]
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            ctx.relations.iter().next().unwrap().1.birectional,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_bidirectional_2() {
        let schema = r"
            type Author @model {
                id: ID!
                posts: [Post]
            }

            type Post @model {
                id: ID!
                author: Author!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            ctx.relations.iter().next().unwrap().1.birectional,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn many_to_many_relation_monodirectional() {
        let schema = r"
            type Author @model {
                id: ID!
                posts: [Post!]
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!]
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1_usize, "Should have only one relation");
        assert!(
            ctx.relations.iter().next().unwrap().1.birectional,
            "Should be birectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }
}
