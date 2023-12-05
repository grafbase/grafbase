use engine_value::ConstValue;
use schema::{
    sources::introspection::Metadata, Definition, DefinitionWalker, EnumValue, FieldWalker, InputValueWalker, Schema,
    TypeWalker,
};

use crate::response::{GroupedFieldWriter, ResponseValue, WriteResult};

pub struct IntrospectionWriter<'a> {
    pub schema: &'a Schema,
    pub types: &'a Metadata,
}

impl<'a> IntrospectionWriter<'a> {
    #[allow(clippy::panic)]
    pub fn write_type_field(&self, mut writer: GroupedFieldWriter<'_>) -> WriteResult<ResponseValue> {
        // There is a single argument if any so don't need to match anything, the
        // query is already validated.
        let name = writer
            .expected_field
            .bound_arguments()
            .next()
            .map(|arg| match arg.resolved_value() {
                ConstValue::String(s) => s,
                _ => panic!("Validation failure: Expected string argument"),
            })
            .expect("Validation failure: missing argument");
        match self.schema.definition_by_name(&name) {
            Some(definition) => self.__type_inner(writer, self.schema.walker().walk(definition)),
            None => writer.write_null(),
        }
    }

    pub fn write_schema_field(&self, mut writer: GroupedFieldWriter<'_>) -> WriteResult<ResponseValue> {
        let schema = self.schema.walker();
        writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
            "description" => writer.write_opt_string_id(writer.expected_field.description),
            "types" => writer.write_list_with(schema.definitions(), |field, item| self.__type_inner(field, item)),
            "queryType" => self.__type_inner(writer, schema.query().into()),
            "mutationType" => match schema.mutation() {
                Some(mutation) => self.__type_inner(writer, mutation.into()),
                None => writer.write_null(),
            },
            "subscriptionType" => match schema.subscription() {
                Some(subscription) => self.__type_inner(writer, subscription.into()),
                None => writer.write_null(),
            },
            // TODO: Need to implemented directives...
            "directives" => writer.write_empty_list(),
            name => unresolvable(name),
        })
    }

    // Ignoring any wrapping
    #[allow(clippy::panic)]
    fn __type_inner(
        &self,
        mut writer: GroupedFieldWriter<'_>,
        definition: DefinitionWalker<'_>,
    ) -> WriteResult<ResponseValue> {
        writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
            "kind" => writer.write_string_id(match definition.id() {
                Definition::Scalar(_) => self.types.type_kind.scalar,
                Definition::Object(_) => self.types.type_kind.object,
                Definition::Interface(_) => self.types.type_kind.interface,
                Definition::Union(_) => self.types.type_kind.union,
                Definition::Enum(_) => self.types.type_kind.r#enum,
                Definition::InputObject(_) => self.types.type_kind.input_object,
            }),
            "name" => writer.write_string_id(definition.schema_name_id()),
            "description" => writer.write_opt_string_id(definition.schema_description_id()),
            "fields" => writer.write_opt_list_with(
                definition.fields().map(|fields| {
                    let include_deprecated = writer
                        .expected_field
                        .bound_arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::Boolean(b) => b,
                            _ => panic!("Expected boolean argument"),
                        })
                        .unwrap_or_default();
                    fields.filter(move |field| {
                        (!field.is_deprecated || include_deprecated) && !self.types.meta_fields.contains(&field.id())
                    })
                }),
                |writer, item| self.__field(writer, item),
            ),
            "interfaces" => writer.write_opt_list_with(definition.interfaces(), |field, item| {
                self.__type_inner(field, item.into())
            }),
            "possibleTypes" => writer.write_opt_list_with(definition.possible_types(), |field, item| {
                self.__type_inner(field, item.into())
            }),
            "enumValues" => writer
                .write_opt_list_with(definition.as_enum().map(|r#enum| r#enum.values()), |field, item| {
                    self.__enum_value(field, item)
                }),
            "inputFields" => writer.write_opt_list_with(
                definition
                    .as_input_object()
                    .map(|input_object| input_object.input_fields()),
                |field, item| self.__input_value(field, item),
            ),
            "ofType" => writer.write_null(),
            "specifiedByURL" => {
                writer.write_opt_string_id(definition.as_scalar().and_then(|scalar| scalar.specified_by_url))
            }
            name => unresolvable(name),
        })
    }

    fn __field(&self, mut writer: GroupedFieldWriter<'_>, field: FieldWalker<'_>) -> WriteResult<ResponseValue> {
        writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
            "name" => writer.write_string_id(field.name),
            "description" => writer.write_opt_string_id(field.description),
            "args" => writer.write_list_with(field.arguments(), |field, item| self.__input_value(field, item)),
            "type" => self.__type(writer, field.ty()),
            "isDeprecated" => writer.write_boolean(field.is_deprecated),
            "deprecationReason" => writer.write_opt_string_id(field.deprecation_reason),
            name => unresolvable(name),
        })
    }

    fn __enum_value(&self, mut writer: GroupedFieldWriter<'_>, field: &EnumValue) -> WriteResult<ResponseValue> {
        writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
            "name" => writer.write_string_id(field.name),
            "description" => writer.write_opt_string_id(field.description),
            "isDeprecated" => writer.write_boolean(field.is_deprecated),
            "deprecationReason" => writer.write_opt_string_id(field.deprecation_reason),
            name => unresolvable(name),
        })
    }

    fn __input_value(
        &self,
        mut writer: GroupedFieldWriter<'_>,
        input_value: InputValueWalker<'_>,
    ) -> WriteResult<ResponseValue> {
        writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
            "name" => writer.write_string_id(input_value.name),
            "description" => writer.write_opt_string_id(input_value.description),
            "type" => self.__type(writer, input_value.ty()),
            // TODO: add default value...
            "defaultValue" => writer.write_null(),
            name => unresolvable(name),
        })
    }

    fn __type(&self, writer: GroupedFieldWriter<'_>, ty: TypeWalker<'_>) -> WriteResult<ResponseValue> {
        // Building it from outermost to innermost
        let mut wrapping = Wrapping::new();
        let mut schema_wrapping = ty.wrapping.clone();
        while let Some(list_wrapping) = schema_wrapping.list_wrapping.pop() {
            match list_wrapping {
                schema::ListWrapping::RequiredList => wrapping.extend([WrappingType::NonNull, WrappingType::List]),
                schema::ListWrapping::NullableList => wrapping.push(WrappingType::List),
            }
        }
        if schema_wrapping.inner_is_required {
            wrapping.push(WrappingType::NonNull);
        }
        wrapping.reverse();
        self.__type_recursive(writer, ty.inner(), wrapping)
    }

    fn __type_recursive(
        &self,
        mut writer: GroupedFieldWriter<'_>,
        definition: DefinitionWalker<'_>,
        mut wrapping: Wrapping,
    ) -> WriteResult<ResponseValue> {
        match wrapping.pop() {
            Some(WrappingType::NonNull) => {
                writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
                    "kind" => writer.write_string_id(self.types.type_kind.non_null),
                    "ofType" => self.__type_recursive(writer, definition, wrapping.clone()),
                    "name" | "description" | "interfaces" | "possibleTypes" | "enumValues" | "inputFields"
                    | "specifiedByURL" => writer.write_null(),
                    name => unresolvable(name),
                })
            }
            Some(WrappingType::List) => {
                writer.write_known_object_with(|mut writer| match writer.expected_field.name() {
                    "kind" => writer.write_string_id(self.types.type_kind.list),
                    "ofType" => self.__type_recursive(writer, definition, wrapping.clone()),
                    "name" | "description" | "interfaces" | "possibleTypes" | "enumValues" | "inputFields"
                    | "specifiedByURL" => writer.write_null(),
                    name => unresolvable(name),
                })
            }
            None => self.__type_inner(writer, definition),
        }
    }
}

pub fn unresolvable(name: &str) -> WriteResult<ResponseValue> {
    Err(format!("Unresolvable field named: '{name}'").into())
}

// Innermort to outermost
type Wrapping = Vec<WrappingType>;

#[derive(Clone, Copy)]
enum WrappingType {
    NonNull,
    List,
}
