use super::{Visitor, VisitorContext};
use engine_parser::{
    types::{
        ConstDirective, FieldDefinition, InputValueDefinition, ObjectType, SchemaDefinition, ServiceDocument,
        TypeDefinition,
    },
    Positioned,
};
use engine_value::{ConstValue, Name};

/// Concat rule
pub struct VisitorCons<A, B>(pub A, pub B);

impl<A, B> VisitorCons<A, B> {
    #[allow(dead_code)]
    pub(crate) const fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
        VisitorCons(visitor, self)
    }
}

/// The monoid implementation for Visitor
impl<'a, A, B> Visitor<'a> for VisitorCons<A, B>
where
    A: Visitor<'a> + 'a,
    B: Visitor<'a> + 'a,
{
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        self.0.enter_schema(ctx, doc);
        self.1.enter_schema(ctx, doc);
    }

    fn exit_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        self.0.exit_schema(ctx, doc);
        self.1.exit_schema(ctx, doc);
    }

    fn enter_scalar_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a Positioned<TypeDefinition>,
    ) {
        self.0.enter_scalar_definition(ctx, type_definition);
        self.1.enter_scalar_definition(ctx, type_definition);
    }

    fn exit_scalar_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a Positioned<TypeDefinition>,
    ) {
        self.0.exit_scalar_definition(ctx, type_definition);
        self.1.exit_scalar_definition(ctx, type_definition);
    }

    fn enter_document(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
        self.0.enter_document(ctx, doc);
        self.1.enter_document(ctx, doc);
    }

    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
        self.0.exit_document(ctx, doc);
        self.1.exit_document(ctx, doc);
    }

    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
        self.0.enter_directive(ctx, directive);
        self.1.enter_directive(ctx, directive);
    }

    fn exit_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
        self.0.exit_directive(ctx, directive);
        self.1.exit_directive(ctx, directive);
    }

    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        self.0.enter_type_definition(ctx, type_definition);
        self.1.enter_type_definition(ctx, type_definition);
    }

    fn exit_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        self.0.exit_type_definition(ctx, type_definition);
        self.1.exit_type_definition(ctx, type_definition);
    }

    fn enter_object_definition(&mut self, ctx: &mut VisitorContext<'a>, object_definition: &'a ObjectType) {
        self.0.enter_object_definition(ctx, object_definition);
        self.1.enter_object_definition(ctx, object_definition);
    }
    fn exit_object_definition(&mut self, ctx: &mut VisitorContext<'a>, object_definition: &'a ObjectType) {
        self.0.exit_object_definition(ctx, object_definition);
        self.1.exit_object_definition(ctx, object_definition);
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        parent_type: &'a Positioned<TypeDefinition>,
    ) {
        self.0.enter_field(ctx, field, parent_type);
        self.1.enter_field(ctx, field, parent_type);
    }
    fn exit_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        parent_type: &'a Positioned<TypeDefinition>,
    ) {
        self.0.exit_field(ctx, field, parent_type);
        self.1.exit_field(ctx, field, parent_type);
    }

    fn enter_input_value_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        value: &'a Positioned<InputValueDefinition>,
    ) {
        self.0.enter_input_value_definition(ctx, value);
        self.1.enter_input_value_definition(ctx, value);
    }
    fn exit_input_value_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        value: &'a Positioned<InputValueDefinition>,
    ) {
        self.0.exit_input_value_definition(ctx, value);
        self.1.exit_input_value_definition(ctx, value);
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        value: &'a Positioned<ConstValue>,
    ) {
        self.0.enter_argument(ctx, name, value);
        self.1.enter_argument(ctx, name, value);
    }
    fn exit_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        value: &'a Positioned<ConstValue>,
    ) {
        self.0.exit_argument(ctx, name, value);
        self.1.exit_argument(ctx, name, value);
    }
}
