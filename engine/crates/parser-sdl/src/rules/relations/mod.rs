use super::visitor::{VisitorCons, VisitorNil};

mod relations_engine;
pub use relations_engine::RelationEngine;

pub const fn relations_rules() -> VisitorCons<RelationEngine, VisitorNil> {
    // TODO: Add Check to ensure the directive is not used outside of Modelized node.
    VisitorNil.with(RelationEngine)
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use insta::assert_debug_snapshot;
    use serde_json as _;

    use super::relations_rules;
    use crate::rules::visitor::{visit, VisitorContext};

    #[test]
    fn multiple_relations() {
        let schema = r"
            type Author @model {
                id: ID!
                postsToday: [Post!]
                postsYesterday: [Post!]
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!]
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut relations_rules(), &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_debug_snapshot!(ctx.errors);
    }

    #[test]
    fn multiple_relations_directive_not_defined() {
        let schema = r#"
            type Author @model {
                id: ID!
                postsToday: [Post!] @relation(name: "postsToday")
                postsYesterday: [Post!] @relation(name: "postsYesterday")
                posts: [Post!] @relation(name: "published")
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!] @relation(name: "published")
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut relations_rules(), &mut ctx, &schema);

        assert_debug_snapshot!(ctx.relations);
        assert!(ctx.errors.is_empty(), "should be empty");
    }
}
