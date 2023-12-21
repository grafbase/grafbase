pub(crate) mod create_type_context;
mod model_type;
mod queries;
pub(super) mod types;

use create_type_context::CreateTypeContext;
use engine::{
    names::{
        INPUT_FIELD_FILTER_ALL, INPUT_FIELD_FILTER_ANY, INPUT_FIELD_FILTER_NONE, INPUT_FIELD_FILTER_NOT,
        OUTPUT_FIELD_ID,
    },
    registry::MongoDBConfiguration,
    Positioned,
};
use engine_parser::types::{ObjectType, TypeDefinition, TypeKind};

use crate::{
    registry::names::MetaNames,
    rules::{
        auth_directive::AuthDirective,
        visitor::{Visitor, VisitorContext},
    },
};

const CONNECTOR_KEY: &str = "connector";
const COLLECTION_KEY: &str = "collection";

const RESERVED_FIELDS: [&str; 5] = [
    OUTPUT_FIELD_ID,
    INPUT_FIELD_FILTER_ALL,
    INPUT_FIELD_FILTER_ANY,
    INPUT_FIELD_FILTER_NONE,
    INPUT_FIELD_FILTER_NOT,
];

pub struct MongoDBModelDirective;

impl<'a> Visitor<'a> for MongoDBModelDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, r#type: &'a Positioned<TypeDefinition>) {
        let Some(config) = get_config(ctx, r#type) else { return };

        let TypeKind::Object(ref object) = r#type.node.kind else {
            return;
        };

        if !validate_model_name(ctx, r#type) {
            return;
        }

        validate_field_names(ctx, object);

        let model_auth = match AuthDirective::parse(ctx, &r#type.node.directives, false) {
            Ok(auth) => auth,
            Err(error) => {
                ctx.report_error(error.locations, error.message);
                None
            }
        };

        let create_ctx = CreateTypeContext::new(ctx, object, model_auth, r#type, config);

        model_type::create(ctx, &create_ctx);
        queries::create(ctx, &create_ctx);
    }
}

fn validate_field_names(ctx: &mut VisitorContext<'_>, object: &ObjectType) {
    for field in &object.fields {
        let name = field.node.name.node.as_str();

        if RESERVED_FIELDS.contains(&name) {
            ctx.report_error(
                vec![field.pos],
                format!("Field name '{name}' is reserved and cannot be used."),
            );
        }
    }
}

fn validate_model_name(ctx: &mut VisitorContext<'_>, r#type: &Positioned<TypeDefinition>) -> bool {
    let type_name = MetaNames::model(&r#type.node);
    let is_valid = r#type.node.name.node == type_name;

    if !is_valid {
        ctx.report_error(
            vec![r#type.node.name.pos],
            format!(
                "Models must be named in PascalCase.  Try renaming {} to {type_name}.",
                r#type.node.name.node
            ),
        );
    }

    is_valid
}

fn get_config<'a>(ctx: &'a VisitorContext<'_>, r#type: &'a Positioned<TypeDefinition>) -> Option<MongoDBConfiguration> {
    if !matches!(r#type.node.kind, TypeKind::Object(_)) {
        return None;
    }

    for directive in &r#type.node.directives {
        if !directive.is_model() {
            continue;
        }

        for (key, argument) in &directive.node.arguments {
            let Some(connector_name) = argument.node.as_str() else {
                continue;
            };

            if key.node.as_str() != CONNECTOR_KEY {
                continue;
            }

            return ctx
                .registry
                .borrow()
                .mongodb_configurations
                .get(connector_name)
                .cloned();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_schema,
        rules::{
            model_directive::ModelDirective,
            mongodb_directive::MongoDBModelDirective,
            visitor::{visit, VisitorContext},
        },
    };

    #[test]
    fn should_not_warn_about_deprecated_models() {
        let schema = r#"
            extend schema
              @mongodb(
                 name: "test",
                 apiKey: "TEST"
                 url: "https://example.com"
                 dataSource: "TEST"
                 database: "test"
                 namespace: false
              )

            type Product @model(connector: "test", collection: "test") {
                id: ID!
                test: String!
            }
        "#;

        let schema = parse_schema(schema).expect("");
        let mut ctx = VisitorContext::new_for_tests(&schema);

        visit(&mut ModelDirective, &mut ctx, &schema);
        visit(&mut MongoDBModelDirective, &mut ctx, &schema);

        assert!(ctx.warnings.is_empty());
    }
}
