mod model_directive;
mod type_directive;

use std::collections::HashMap;

use grafbase_engine::registry::{MetaField, MongoDBConfiguration, ObjectType};
use grafbase_engine_parser::types::SchemaDefinition;
use inflector::Inflector;
pub(super) use model_directive::create_type_context::CreateTypeContext;
pub use model_directive::MongoDBModelDirective;
pub use type_directive::MongoDBTypeDirective;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::directive_de::parse_directive;

static NUMERIC_SCALARS: &[&str] = &["BigInt", "Decimal", "Float", "Int"];

static MONGODB_SCALARS: &[&str] = &[
    "Boolean",
    "BigInt",
    "Bytes",
    "Decimal",
    "Date",
    "DateTime",
    "Float",
    "ID",
    "Int",
    "JSON",
    "PhoneNumber",
    "String",
    "Timestamp",
    "URL",
];

static DATE_TIME_SCALARS: &[&str] = &["Date", "DateTime", "Timestamp"];

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBDirective {
    name: String,
    url: String,
    api_key: String,
    data_source: String,
    database: String,
    namespace: Option<String>,
}

impl MongoDBDirective {
    /// The host url for the Atlas Data API.
    pub fn url(&self) -> &str {
        &self.url
    }

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
          apiKey: String!

          """
          The full URL. Found from the MongoDB Atlas dashboard.
          """
          url: String!

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
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a grafbase_engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == MONGODB_DIRECTIVE_NAME);

        let mut directive_names: HashMap<String, Vec<grafbase_engine::Pos>> = HashMap::new();
        let mut found_directive = false;

        for directive in directives {
            match parse_directive::<MongoDBDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    directive_names
                        .entry(parsed_directive.name().to_string())
                        .or_default()
                        .push(directive.name.pos);

                    ctx.registry.get_mut().create_mongo_directive(
                        |_| MongoDBConfiguration {
                            name: parsed_directive.name().to_string(),
                            api_key: parsed_directive.api_key().to_string(),
                            url: parsed_directive.url().to_string(),
                            data_source: parsed_directive.data_source().to_string(),
                            database: parsed_directive.database().to_string(),
                            namespace: parsed_directive.namespace.clone(),
                        },
                        parsed_directive.name(),
                    );

                    if let Some(namespace) = parsed_directive.namespace() {
                        let query_type_name = format!("{namespace}Query").to_pascal_case();
                        let mutation_type_name = format!("{namespace}Mutation").to_pascal_case();

                        ctx.registry.borrow_mut().create_type(
                            |_| ObjectType::new(query_type_name.clone(), []).into(),
                            &query_type_name,
                            &query_type_name,
                        );

                        ctx.queries
                            .push(MetaField::new(namespace.to_camel_case(), query_type_name.clone()));

                        ctx.registry.borrow_mut().create_type(
                            |_| ObjectType::new(mutation_type_name.clone(), []).into(),
                            &mutation_type_name,
                            &mutation_type_name,
                        );

                        ctx.mutations
                            .push(MetaField::new(namespace.to_camel_case(), mutation_type_name.clone()));
                    }

                    found_directive = true;
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }

        if found_directive {
            model_directive::types::generic::register_input(ctx);
        }

        for (name, positions) in directive_names.into_iter().filter(|(_, positions)| positions.len() > 1) {
            let message = format!(
                "Directive name '{name}' is already in use in more than one MongoDB connector, please use a distinctive name.",
            );

            ctx.report_error(positions, message);
        }
    }
}
