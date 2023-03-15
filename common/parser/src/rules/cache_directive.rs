use crate::directive_de::parse_directive;
use crate::rules::visitor::VisitorContext;
use dynaql::CacheControl;
use dynaql_parser::types::{ConstDirective, FieldDefinition, TypeDefinition, TypeKind};
use dynaql_parser::{Pos, Positioned};

use super::{directive::Directive, visitor::Visitor};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheDirective {
    #[serde(rename = "maxAge")]
    pub max_age: usize,
    #[serde(default, rename = "staleWhileRevalidate")]
    pub stale_while_revalidate: usize,
}

impl CacheDirective {
    pub fn parse(directives: &[Positioned<ConstDirective>]) -> CacheControl {
        directives
            .iter()
            .find(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
            .and_then(|directive| parse_directive::<CacheDirective>(&directive.node).ok())
            .unwrap_or_default()
            .into()
    }
}

impl From<CacheDirective> for CacheControl {
    fn from(value: CacheDirective) -> Self {
        CacheControl {
            public: true,
            max_age: value.max_age,
            stale_while_revalidate: value.stale_while_revalidate,
        }
    }
}

const CACHE_DIRECTIVE_NAME: &str = "cache";

impl Directive for CacheDirective {
    fn definition() -> String {
        r#"
        directive @cache(
          "How long query results should be cached, in seconds"
          maxAge: Int!
          "How long, in seconds, stale cached results should be served while data is refreshed"
          staleWhileRevalidate: Int
        ) on SCHEMA | OBJECT | FIELD_DEFINITION
        "#
        .to_string()
    }
}

pub struct CacheVisitor;

impl CacheVisitor {
    fn validate_directive<'a>(
        &self,
        ctx: &mut VisitorContext<'a>,
        directives: impl Iterator<Item = &'a Positioned<ConstDirective>>,
        pos: Pos,
    ) -> Option<CacheDirective> {
        let directives: Vec<_> = directives
            .filter(|d| d.node.name.node == CACHE_DIRECTIVE_NAME)
            .collect();

        if directives.len() > 1 {
            ctx.report_error(vec![pos], "only one @cache directive is allowed");
        }

        directives.first().and_then(|pos_const_directive| {
            match parse_directive::<CacheDirective>(&pos_const_directive.node) {
                Ok(cache_directive) => Some(cache_directive),
                Err(err) => {
                    ctx.report_error(vec![pos_const_directive.pos], format!("@cache directive error: {err}"));
                    None
                }
            }
        })
    }
}

impl<'a> Visitor<'a> for CacheVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        doc: &'a Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        if let Some(global_cache_directive) = self.validate_directive(ctx, doc.node.directives.iter(), doc.pos) {
            ctx.global_cache_directive = global_cache_directive;
        }
    }

    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        if let TypeKind::Object(_) = &type_definition.node.kind {
            self.validate_directive(ctx, type_definition.node.directives.iter(), type_definition.pos);
        }
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        self.validate_directive(ctx, field.node.directives.iter(), field.pos);
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::cache_directive::CacheVisitor;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    #[rstest::rstest]
    // Global
    #[case(r#"
        extend schema @cache
    "#, &[
        "@cache directive error: missing field `maxAge`"
    ])]
    #[case(r#"
        extend schema @cache(maxAge: 60) @cache(maxAge: 40)
    "#, &[
        "only one @cache directive is allowed"
    ])]
    #[case(r#"
        extend schema @cache(maxAge: 60)
    "#, &[])]
    #[case(r#"
        extend schema @cache(maxAge: 60, staleWhileRevalidate: 300)
    "#, &[])]
    // Model
    #[case(r#"
        type Test @cache {
            balance: Int!
        }
    "#, &[
        "@cache directive error: missing field `maxAge`"
    ])]
    #[case(r#"
        type Test @cache(maxAge: 60) {
            balance: Int!
        }
    "#, &[])]
    #[case(r#"
        type Test @cache(maxAge: 60, staleWhileRevalidate: 300) {
            balance: Int!
        }
    "#, &[])]
    // Fields
    #[case(r#"
        type Test {
            balance: Int! @cache
        }
    "#, &[
        "@cache directive error: missing field `maxAge`"
    ])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 60)
        }
    "#, &[])]
    #[case(r#"
        type Test {
            balance: Int! @cache(maxAge: 60, staleWhileRevalidate: 300)
        }
    "#, &[])]
    fn test_parse_result(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CacheVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }
}
