mod cons;
mod context;
mod error;
mod nil;
mod r#trait;
mod warnings;

use engine::Positioned;
use engine_parser::types::{
    ConstDirective, FieldDefinition, ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};

pub(crate) use self::{
    cons::VisitorCons, context::VisitorContext, error::RuleError, nil::VisitorNil, r#trait::Visitor, warnings::Warning,
};

type TypeStackType<'a> = Vec<(Option<&'a Positioned<Type>>, Option<&'a Positioned<TypeDefinition>>)>;

pub const QUERY_TYPE: &str = "Query";
pub const MUTATION_TYPE: &str = "Mutation";

pub fn visit<'a, V: Visitor<'a>>(v: &mut V, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
    v.enter_document(ctx, doc);

    for operation in &doc.definitions {
        match operation {
            TypeSystemDefinition::Type(ty) => {
                v.enter_type_definition(ctx, ty);
                // Inside Type Definition we should visit_field
                match &ty.node.kind {
                    TypeKind::Object(object) => {
                        ctx.with_definition_type(Some(ty), |ctx| visit_directives(v, ctx, &ty.node.directives));

                        v.enter_object_definition(ctx, object);
                        for field in &object.fields {
                            visit_field(v, ctx, field, ty);
                        }
                        v.exit_object_definition(ctx, object);
                    }
                    TypeKind::Scalar => {
                        v.enter_scalar_definition(ctx, ty);
                        visit_directives(v, ctx, &ty.node.directives);
                        v.exit_scalar_definition(ctx, ty);
                    }
                    _ => {}
                };
                v.exit_type_definition(ctx, ty);
            }
            TypeSystemDefinition::Schema(schema) => {
                v.enter_schema(ctx, schema);
                visit_directives(v, ctx, &schema.node.directives);
                v.exit_schema(ctx, schema);
            }
            _ => {}
        }
    }

    v.exit_document(ctx, doc);
}

fn visit_field<'a, V: Visitor<'a>>(
    v: &mut V,
    ctx: &mut VisitorContext<'a>,
    field: &'a Positioned<FieldDefinition>,
    parent_type: &'a Positioned<TypeDefinition>,
) {
    v.enter_field(ctx, field, parent_type);

    for value in &field.node.arguments {
        v.enter_input_value_definition(ctx, value);
        ctx.with_type(Some(&field.node.ty), |ctx| {
            visit_directives(v, ctx, &value.node.directives);
        });
        v.exit_input_value_definition(ctx, value);
    }

    visit_directives(v, ctx, &field.node.directives);

    v.exit_field(ctx, field, parent_type);
}

fn visit_directives<'a, V: Visitor<'a>>(
    v: &mut V,
    ctx: &mut VisitorContext<'a>,
    directives: &'a [Positioned<ConstDirective>],
) {
    for d in directives {
        v.enter_directive(ctx, d);

        // TODO: Should check than directive is inside schema defined Directives.
        for (name, value) in &d.node.arguments {
            v.enter_argument(ctx, name, value);
            v.exit_argument(ctx, name, value);
        }

        v.exit_directive(ctx, d);
    }
}
