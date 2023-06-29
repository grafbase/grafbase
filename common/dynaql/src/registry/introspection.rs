//! Allow converting a [`cynic_introspection::Schema`] into a new [`Registry`].
//!
//! The conversion takes all the known information from the introspection and adds it to the
//! registry.

use super::{
    Deprecation, MetaDirective, MetaEnumValue, MetaField, MetaInputValue, MetaType, ObjectType,
    Registry, __DirectiveLocation,
};

impl From<cynic_introspection::Schema> for Registry {
    fn from(schema: cynic_introspection::Schema) -> Self {
        let mut registry = Self::new();

        // root types
        registry.query_type = schema.query_type;
        registry.mutation_type = schema.mutation_type;
        registry.subscription_type = schema.subscription_type;

        // directives
        registry.directives = schema
            .directives
            .into_iter()
            .map(|d| (d.name.clone(), d.into()))
            .collect();

        // types
        registry.types = schema
            .types
            .into_iter()
            .map(|ty| (ty.name().to_owned(), ty.into()))
            .collect();

        registry
    }
}

impl From<cynic_introspection::Directive> for MetaDirective {
    fn from(directive: cynic_introspection::Directive) -> Self {
        Self {
            name: directive.name,
            description: directive.description,
            locations: directive.locations.into_iter().map(Into::into).collect(),
            args: directive
                .args
                .into_iter()
                .map(|v| (v.name.clone(), v.into()))
                .collect(),
            is_repeatable: false,
            visible: None,
        }
    }
}

impl From<cynic_introspection::Field> for MetaField {
    fn from(field: cynic_introspection::Field) -> Self {
        Self {
            name: field.name,
            description: field.description,
            args: field
                .args
                .into_iter()
                .map(|v| (v.name.clone(), v.into()))
                .collect(),
            ty: field.ty.to_string().into(),
            deprecation: field.deprecated.into(),
            ..Default::default()
        }
    }
}

impl From<cynic_introspection::InputValue> for MetaInputValue {
    fn from(input: cynic_introspection::InputValue) -> Self {
        Self {
            name: input.name,
            description: input.description,
            ty: input.ty.to_string(),
            default_value: input.default_value.map(Into::into),
            visible: None,
            validators: None,
            is_secret: false,
            rename: None,
        }
    }
}

impl From<cynic_introspection::DirectiveLocation> for __DirectiveLocation {
    fn from(location: cynic_introspection::DirectiveLocation) -> Self {
        use __DirectiveLocation::*;
        use cynic_introspection::DirectiveLocation::*;

        match location {
            Query => QUERY,
            Mutation => MUTATION,
            Subscription => SUBSCRIPTION,
            Field => FIELD,
            FragmentDefinition => FRAGMENT_DEFINITION,
            FragmentSpread => FRAGMENT_SPREAD,
            InlineFragment => INLINE_FRAGMENT,
            VariableDefinition => VARIABLE_DEFINITION,
            Schema => SCHEMA,
            Scalar => SCALAR,
            Object => OBJECT,
            FieldDefinition => FIELD_DEFINITION,
            ArgumentDefinition => ARGUMENT_DEFINITION,
            Interface => INTERFACE,
            Union => UNION,
            Enum => ENUM,
            EnumValue => ENUM_VALUE,
            InputObject => INPUT_OBJECT,
            InputFieldDefinition => INPUT_FIELD_DEFINITION,
        }
    }
}

impl From<cynic_introspection::Type> for MetaType {
    fn from(ty: cynic_introspection::Type) -> Self {
        use cynic_introspection::Type::*;

        match ty {
            Object(v) => v.into(),
            InputObject(v) => v.into(),
            Enum(v) => v.into(),
            Interface(v) => v.into(),
            Union(v) => v.into(),
            Scalar(v) => v.into(),
        }
    }
}

impl From<cynic_introspection::ObjectType> for MetaType {
    fn from(object: cynic_introspection::ObjectType) -> Self {
        ObjectType::new(object.name, object.fields.into_iter().map(Into::into))
            .with_description(object.description)
            .into()
    }
}

impl From<cynic_introspection::Deprecated> for Deprecation {
    fn from(deprecated: cynic_introspection::Deprecated) -> Self {
        use cynic_introspection::Deprecated::*;
        use Deprecation::*;

        match deprecated {
            No => NoDeprecated,
            Yes(reason) => Deprecated { reason },
        }
    }
}

impl From<cynic_introspection::InputObjectType> for MetaType {
    fn from(input: cynic_introspection::InputObjectType) -> Self {
        super::InputObjectType::new(input.name, input.fields.into_iter().map(Into::into))
            .with_description(input.description)
            .into()
    }
}

impl From<cynic_introspection::EnumType> for MetaType {
    fn from(enum_type: cynic_introspection::EnumType) -> Self {
        super::EnumType::new(enum_type.name, enum_type.values.into_iter().map(Into::into))
            .with_description(enum_type.description)
            .into()
    }
}

impl From<cynic_introspection::EnumValue> for MetaEnumValue {
    fn from(enum_value: cynic_introspection::EnumValue) -> Self {
        Self {
            name: enum_value.name,
            description: enum_value.description,
            deprecation: enum_value.deprecated.into(),
            visible: None,
            value: None,
        }
    }
}

impl From<cynic_introspection::InterfaceType> for MetaType {
    fn from(interface: cynic_introspection::InterfaceType) -> Self {
        Self::Interface(super::InterfaceType {
            name: interface.name.clone(),
            description: interface.description,
            fields: interface
                .fields
                .into_iter()
                .map(|v| (v.name.clone(), v.into()))
                .collect(),
            possible_types: interface.possible_types.into_iter().collect(),
            extends: false,
            keys: None,
            visible: None,
            rust_typename: interface.name,
        })
    }
}

impl From<cynic_introspection::UnionType> for MetaType {
    fn from(union: cynic_introspection::UnionType) -> Self {
        Self::Union(super::UnionType {
            name: union.name.clone(),
            description: union.description,
            possible_types: union.possible_types.into_iter().collect(),
            visible: None,
            rust_typename: union.name,
            discriminators: None,
        })
    }
}

impl From<cynic_introspection::ScalarType> for MetaType {
    fn from(scalar: cynic_introspection::ScalarType) -> Self {
        Self::Scalar(super::ScalarType {
            name: scalar.name,
            description: scalar.description,
            is_valid: None,
            visible: None,
            specified_by_url: None,
            parser: super::ScalarParser::PassThrough,
        })
    }
}

#[cfg(test)]
mod tests {
    use cynic_introspection::query::IntrospectionQuery;

    use super::*;

    #[test]
    fn conversion() {
        let data = include_str!("../../tests/swapi_introspection.json");
        let schema = serde_json::from_str::<IntrospectionQuery>(data)
            .unwrap()
            .into_schema()
            .unwrap();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(Registry::from(schema))
        })
    }

    #[test]
    fn array_input_value() {
        let data = include_str!("../../tests/countries_introspection.json");
        let schema = serde_json::from_str::<IntrospectionQuery>(data)
            .unwrap()
            .into_schema()
            .unwrap();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(Registry::from(schema))
        })
    }
}
