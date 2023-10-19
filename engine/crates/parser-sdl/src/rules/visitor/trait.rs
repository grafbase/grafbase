use super::VisitorContext;
use engine_parser::{
    types::{
        ConstDirective, FieldDefinition, InputValueDefinition, ObjectType, SchemaDefinition, ServiceDocument,
        TypeDefinition,
    },
    Positioned,
};
use engine_value::{ConstValue, Name};

pub trait Visitor<'a> {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}
    fn exit_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}

    fn enter_schema(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a Positioned<SchemaDefinition>) {}
    fn exit_schema(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a Positioned<SchemaDefinition>) {}

    fn enter_type_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn exit_type_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_object_definition(&mut self, _ctx: &mut VisitorContext<'a>, _object_definition: &'a ObjectType) {}
    fn exit_object_definition(&mut self, _ctx: &mut VisitorContext<'a>, _object_definition: &'a ObjectType) {}

    fn enter_scalar_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn exit_scalar_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Positioned<ConstDirective>) {}
    fn exit_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Positioned<ConstDirective>) {}

    fn enter_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn exit_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_input_value_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _value: &'a Positioned<InputValueDefinition>,
    ) {
    }

    fn exit_input_value_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _value: &'a Positioned<InputValueDefinition>,
    ) {
    }

    fn enter_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: &'a Positioned<ConstValue>,
    ) {
    }

    fn exit_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: &'a Positioned<ConstValue>,
    ) {
    }
}
