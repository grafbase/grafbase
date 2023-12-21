mod model_directive;
mod type_directive;

use engine::registry::{MetaField, MongoDBConfiguration, ObjectType};
use engine_parser::types::SchemaDefinition;
use inflector::Inflector;
pub(super) use model_directive::create_type_context::CreateTypeContext;
pub use model_directive::MongoDBModelDirective;
pub use type_directive::MongoDBTypeDirective;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::{directive_de::parse_directive, validations::validate_connector_name};

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
    #[serde(default = "default_to_true")]
    namespace: bool,
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
}

fn default_to_true() -> bool {
    true
}

const MONGODB_DIRECTIVE_NAME: &str = "mongodb";

impl Directive for MongoDBDirective {
    fn definition() -> String {
        r#"
        directive @mongodb(
          """
          A unique name for the given directive.
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
          If true, namespaces queries and mutations with the
          connector name. Defaults to true.
          """
          namespace: Boolean
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct MongoDBVisitor;

impl<'a> Visitor<'a> for MongoDBVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == MONGODB_DIRECTIVE_NAME);

        let mut found_directive = false;

        for directive in directives {
            let result = parse_directive::<MongoDBDirective>(&directive.node, ctx.variables)
                .map_err(|error| error.to_string())
                .and_then(|directive| directive.validate());

            match result {
                Ok(parsed_directive) => {
                    ctx.registry.get_mut().create_mongo_config(
                        |_| MongoDBConfiguration {
                            name: parsed_directive.name().to_string(),
                            api_key: parsed_directive.api_key().to_string(),
                            url: parsed_directive.url().to_string(),
                            data_source: parsed_directive.data_source().to_string(),
                            database: parsed_directive.database().to_string(),
                            namespace: parsed_directive.namespace,
                        },
                        parsed_directive.name(),
                    );

                    if parsed_directive.namespace {
                        let namespace = parsed_directive.name.as_str();
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

                    ctx.mongodb_directives.push((parsed_directive, directive.pos));

                    found_directive = true;
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }

        if found_directive {
            model_directive::types::generic::register_input(ctx);
        }
    }
}

impl MongoDBDirective {
    fn validate(self) -> Result<Self, String> {
        validate_connector_name(&self.name)?;

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_validation_error;

    #[test]
    fn empty_name() {
        assert_validation_error!(
            r#"
            extend schema
              @mongodb(
                name: ""
                apiKey: "i am a key"
                url: "http://example.com/mongodbinnit"
                dataSource: "woop"
                database: "poow"
              )
            "#,
            "Connector names cannot be empty"
        );
    }

    #[test]
    fn invalid_name() {
        assert_validation_error!(
            r#"
            extend schema
              @mongodb(
                name: "1234"
                apiKey: "i am a key"
                url: "http://example.com/mongodbinnit"
                dataSource: "woop"
                database: "poow"
              )
            "#,
            "Connector names must be alphanumeric and cannot start with a number"
        );
    }

    #[test]
    fn basic_introspection() {
        let config = r#"
extend schema
  @mongodb(
    namespace: false
    name: "MongoDB"
    url: "https://example.org"
    apiKey: "gloubiboulga"
    dataSource: "data-source"
    database: "database"
  )

extend schema @federation(version: "2.3")

enum Enum1 {
  Variant1,
  Variant2,
  Variant3
}

type Model1 @model(connector: "MongoDB", collection: "Model1") @key(fields: "field3") {
  field1: String!
  field2: String!
  field3: String! @unique
}

type Model2 @model(connector: "MongoDB", collection: "Model2") @key(fields: field1) {
  field1: String!
  field2: Enum1!
  field3: [Model1!]!
}
        "#;

        let registry = crate::parse_registry(config).unwrap();

        let mut errs = Vec::new();

        for tpe in registry.types.values() {
            match tpe {
                engine::registry::MetaType::Object(obj) => {
                    for field in obj.fields.values() {
                        if let Err(err) = registry.lookup(&field.ty) {
                            errs.push(err);
                        }
                    }
                }
                engine::registry::MetaType::Interface(iface) => {
                    for field in iface.fields.values() {
                        if let Err(err) = registry.lookup(&field.ty) {
                            errs.push(err);
                        }
                    }
                }
                engine::registry::MetaType::Union(unn) => {
                    for possible_type in unn.possible_types.iter() {
                        assert!(registry.types.contains_key(possible_type));
                    }
                }
                engine::registry::MetaType::InputObject(input_object) => {
                    for field in input_object.input_fields.values() {
                        if let Err(err) = registry.lookup(&field.ty) {
                            errs.push(err);
                        }
                    }
                }
                _ => (),
            }
        }

        if !errs.is_empty() {
            panic!("{:#?}", errs)
        }
    }
}
