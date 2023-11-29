use engine_parser::types::SchemaDefinition;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::directive_de::parse_directive;

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GraphType {
    Single,
    Federated,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphDirective {
    pub r#type: GraphType,
}
const GRAPH_DIRECTIVE_NAME: &str = "graph";

impl Directive for GraphDirective {
    fn definition() -> String {
        r#"
        enum GraphType {
            SINGLE
            FEDERATED
        }
         
        directive @graph(
          """
          The type of the graph.
          """
          type: GraphType
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct GraphVisitor;

impl<'a> Visitor<'a> for GraphVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a engine::Positioned<SchemaDefinition>) {
        use itertools::Itertools;

        if let Ok(Some(directive)) = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == GRAPH_DIRECTIVE_NAME)
            .at_most_one()
            .map_err(|mut err| {
                let second_occurrence = err.nth(1).unwrap();
                ctx.report_error(
                    vec![second_occurrence.pos],
                    "duplicate `extend schema @graph`".to_string(),
                );
            })
        {
            let result: Result<crate::GraphDirective, String> =
                parse_directive::<GraphDirective>(&directive.node, ctx.variables).map_err(|error| error.to_string());

            match result {
                Ok(parsed_directive) => {
                    if parsed_directive.r#type == GraphType::Federated {
                        ctx.registry.borrow_mut().is_federated = true;
                    }
                }
                Err(err) => ctx.report_error(vec![directive.pos], err),
            }
        }
    }
}
