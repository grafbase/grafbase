use std::collections::HashMap;

use dynaql_parser::types::SchemaDefinition;

use crate::directive_de::parse_directive;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBDirective {
    name: String,
    api_key: String,
    app_id: String,
    data_source: String,
    database: String,
    namespace: Option<String>,
}

impl MongoDBDirective {
    /// A unique name for the given directive. Used in the model
    /// definitions to map them into the correct datasource.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// An API key for the MongoDB Atlas Data API. Generated
    /// in the Atlas dashboard.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// A unique ID for the application. Found from the
    /// MongoDB Atlas dashboard.
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// The name of the database cluster. Found from the
    /// MongoDB Atlas dashboard.
    pub fn data_source(&self) -> &str {
        &self.data_source
    }

    /// The database holding the collections. Found from the
    /// Atlas cluster management in the Collections tab.
    pub fn database(&self) -> &str {
        &self.database
    }

    /// For now, does nothing. Could be used for the generated
    /// types when implementing introspection for the connector.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

const MONGODB_DIRECTIVE_NAME: &str = "mongodb";

impl Directive for MongoDBDirective {
    fn definition() -> String {
        r#"
        directive @mongodb(
          """
          A unique name for the given directive. Used in the model
          definitions to map them into the correct datasource.
          """
          name: String!

          """
          An API key for the MongoDB Atlas Data API. Generated
          in the Atlas dashboard.

          """
          api_key: String!

          """
          A unique ID for the application. Found from the
          MongoDB Atlas dashboard.

          """
          app_id: String!

          """
          The name of the database cluster. Found from the
          MongoDB Atlas dashboard.
          """
          dataSource: String!

          """
          The database holding the collections. Found from the
          Atlas cluster management in the Collections tab.
          """
          database: String!

          """
          For now, does nothing. Could be used for the generated
          types when implementing introspection for the connector.
          """
          namespace: String
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct MongoDBVisitor;

impl<'a> Visitor<'a> for MongoDBVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a dynaql::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == MONGODB_DIRECTIVE_NAME);

        let mut directive_names: HashMap<String, Vec<dynaql::Pos>> = HashMap::new();

        for directive in directives {
            match parse_directive::<MongoDBDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    directive_names
                        .entry(parsed_directive.name().to_string())
                        .or_default()
                        .push(directive.name.pos);

                    ctx.mongodb_directives.push((parsed_directive, directive.pos));
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }

        for (name, positions) in directive_names.into_iter().filter(|(_, positions)| positions.len() > 1) {
            let message = format!(
                "Directive name '{}' is already in use in more than one MongoDB connector, please use a distinctive name.",
                name
            );

            ctx.report_error(positions, message);
        }
    }
}
