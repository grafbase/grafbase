use std::collections::HashMap;

use indexmap::IndexMap;
use registry_v2::{
    ids::{InputValidatorId, MetaFieldId, MetaTypeId},
    storage::*,
    writer::RegistryWriter,
    IdRange, TypeWrappers, UnionDiscriminators,
};

mod partial_cache_registry;

pub use partial_cache_registry::convert_v1_to_partial_cache_registry;

pub fn convert_v1_to_v2(v1: registry_v1::Registry) -> registry_v2::Registry {
    let mut writer = RegistryWriter::new();

    let registry_v1::Registry {
        types,
        directives,
        implements,
        query_type,
        mutation_type,
        subscription_type,
        disable_introspection,
        enable_federation,
        federation_subscription,
        auth,
        mongodb_configurations,
        http_headers,
        postgres_databases,
        enable_caching,
        enable_kv,
        federation_entities,
        enable_ai: _,
        enable_codegen,
        is_federated,
        operation_limits,
        trusted_documents,
        cors_config,
        codegen,
    } = v1;

    // First, copy over the fields that are the same.
    writer.disable_introspection = disable_introspection;
    writer.enable_federation = enable_federation;
    writer.federation_subscription = federation_subscription;
    writer.auth = auth;
    writer.mongodb_configurations = mongodb_configurations;
    writer.http_headers = http_headers;
    writer.postgres_databases = postgres_databases;
    writer.enable_caching = enable_caching;
    writer.enable_kv = enable_kv;
    writer.federation_entities = federation_entities;
    writer.enable_codegen = enable_codegen;
    writer.is_federated = is_federated;
    writer.operation_limits = operation_limits;
    writer.trusted_documents = trusted_documents;
    writer.codegen = codegen;
    writer.cors_config = cors_config;

    let types = {
        let mut types = types.into_values().collect::<Vec<_>>();
        // Comes out of a BTreeMap so should be sorted, but it's important
        // so lets sort incase the type changes.
        types.sort_by(|lhs, rhs| lhs.name().cmp(rhs.name()));
        types
    };

    // Build a map of type name -> the ID it'll have when we insert it.
    let preallocated_ids = writer.preallocate_type_ids(types.len()).collect::<Vec<_>>();
    assert_eq!(preallocated_ids.len(), types.len());

    let type_ids = types
        .iter()
        .zip(preallocated_ids.iter().cloned())
        .map(|(ty, id)| (ty.name().to_string(), id))
        .collect::<HashMap<_, _>>();

    for (ty, id) in types.into_iter().zip(preallocated_ids) {
        let record = insert_type(ty, &mut writer, &type_ids);
        writer.populate_preallocated_type(id, record);
    }

    writer.query_type = Some(lookup_type_id(&type_ids, &query_type));
    writer.mutation_type = mutation_type.map(|name| lookup_type_id(&type_ids, &name));
    writer.subscription_type = subscription_type.map(|name| lookup_type_id(&type_ids, &name));

    let directives = {
        let mut directives = directives.into_values().collect::<Vec<_>>();
        directives.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        directives
    };
    for directive in directives {
        insert_directive(directive, &mut writer, &type_ids);
    }

    writer.implements = implements
        .into_iter()
        .map(|(ty, implements)| (type_ids[&ty], implements.into_iter().map(|ty| type_ids[&ty]).collect()))
        .collect();

    writer.finish().unwrap()
}

fn insert_type(
    ty: registry_v1::MetaType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    match ty {
        registry_v1::MetaType::Scalar(inner) => insert_scalar(inner, writer, type_ids),
        registry_v1::MetaType::Object(inner) => insert_object(inner, writer, type_ids),
        registry_v1::MetaType::Interface(inner) => insert_interface(inner, writer, type_ids),
        registry_v1::MetaType::Union(inner) => insert_union(inner, writer, type_ids),
        registry_v1::MetaType::Enum(inner) => insert_enum(inner, writer, type_ids),
        registry_v1::MetaType::InputObject(inner) => insert_input_object(inner, writer, type_ids),
    }
}

fn insert_scalar(
    scalar: registry_v1::ScalarType,
    writer: &mut RegistryWriter,
    _type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::ScalarType {
        name,
        description,
        is_valid: _,
        specified_by_url,
        parser,
    } = scalar;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));
    let specified_by_url = specified_by_url.map(|url| writer.intern_string(url));

    writer.insert_scalar(ScalarTypeRecord {
        name,
        description,
        specified_by_url,
        parser,
    })
}

fn insert_object(
    inner: registry_v1::ObjectType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::ObjectType {
        name,
        description,
        fields,
        cache_control,
        extends: _,
        is_subscription: _,
        is_node: _,
        rust_typename: _,
        constraints: _,
        external,
        shareable,
    } = inner;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));

    let fields = insert_fields(fields, writer, type_ids);

    writer.insert_object(ObjectTypeRecord {
        name,
        description,
        fields,
        cache_control,
        external,
        shareable,
    })
}

fn insert_fields(
    fields: IndexMap<String, registry_v1::MetaField>,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> IdRange<MetaFieldId> {
    let fields = fields
        .into_values()
        .map(|field| {
            let registry_v1::MetaField {
                name,
                mapped_name,
                description,
                args,
                ty,
                deprecation,
                cache_control,
                requires,
                federation,
                resolver,
                required_operation,
                auth,
            } = field;

            let name = writer.intern_string(name);
            let mapped_name = mapped_name.map(|name| writer.intern_string(name));
            let description = description.map(|desc| writer.intern_string(desc));
            let args = insert_input_values(args, writer, type_ids);
            let ty = convert_meta_field_type(ty, type_ids);
            let deprecation = deprecation.is_deprecated().then(|| Box::new(deprecation));
            let requires = requires.map(Box::new);
            let required_operation = required_operation.map(Box::new);

            MetaFieldRecord {
                name,
                mapped_name,
                description,
                args,
                ty,
                deprecation,
                cache_control,
                requires,
                federation,
                resolver,
                required_operation,
                auth,
            }
        })
        .collect();

    writer.insert_fields(fields)
}

fn insert_input_values(
    values: IndexMap<String, registry_v1::MetaInputValue>,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> IdRange<registry_v2::ids::MetaInputValueId> {
    let values = values
        .into_values()
        .map(|field| {
            let registry_v1::MetaInputValue {
                name,
                description,
                ty,
                default_value,
                validators,
                is_secret: _,
                rename,
            } = field;

            let name = writer.intern_string(name);
            let description = description.map(|desc| writer.intern_string(desc));
            let ty = convert_input_value_type(ty, type_ids);
            let default_value = default_value.map(Box::new);
            let rename = rename.map(|rename| writer.intern_string(rename));
            let validators = validators
                .map(|validators| insert_validators(validators, writer))
                .unwrap_or_default();

            MetaInputValueRecord {
                name,
                description,
                ty,
                default_value,
                rename,
                validators,
            }
        })
        .collect();

    writer.insert_input_values(values)
}

fn insert_interface(
    inner: registry_v1::InterfaceType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::InterfaceType {
        name,
        description,
        fields,
        cache_control,
        possible_types,
        extends: _,
        rust_typename: _,
    } = inner;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));

    let fields = insert_fields(fields, writer, type_ids);
    let possible_types = possible_types
        .into_iter()
        .map(|ty| lookup_type_id(type_ids, &ty))
        .collect();

    writer.insert_interface(InterfaceTypeRecord {
        name,
        description,
        fields,
        cache_control,
        possible_types,
    })
}

fn insert_union(
    inner: registry_v1::UnionType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::UnionType {
        name,
        description,
        possible_types,
        rust_typename: _,
        discriminators,
    } = inner;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));
    let possible_types = possible_types
        .into_iter()
        .map(|ty| lookup_type_id(type_ids, &ty))
        .collect();
    let discriminators = UnionDiscriminators(discriminators.unwrap_or_default());

    writer.insert_union(UnionTypeRecord {
        name,
        description,
        possible_types,
        discriminators,
    })
}

fn insert_enum(
    inner: registry_v1::EnumType,
    writer: &mut RegistryWriter,
    _type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::EnumType {
        name,
        description,
        enum_values,
        rust_typename: _,
    } = inner;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));
    let values = insert_enum_values(enum_values, writer);

    writer.insert_enum(EnumTypeRecord {
        name,
        description,
        values,
    })
}

fn insert_enum_values(
    enum_values: IndexMap<String, registry_v1::MetaEnumValue>,
    writer: &mut RegistryWriter,
) -> IdRange<registry_v2::ids::MetaEnumValueId> {
    let values = enum_values
        .into_values()
        .map(|value| {
            let registry_v1::MetaEnumValue {
                name,
                description,
                deprecation,
                value,
            } = value;

            let name = writer.intern_string(name);
            let description = description.map(|desc| writer.intern_string(desc));
            let deprecation = deprecation.is_deprecated().then(|| Box::new(deprecation));
            let value = value.map(|val| writer.intern_string(val));

            MetaEnumValueRecord {
                name,
                description,
                deprecation,
                value,
            }
        })
        .collect();

    writer.insert_enum_values(values)
}

fn insert_input_object(
    inner: registry_v1::InputObjectType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::InputObjectType {
        name,
        description,
        input_fields,
        rust_typename: _,
        oneof,
    } = inner;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));

    let input_fields = insert_input_values(input_fields, writer, type_ids);

    writer.insert_input_object(InputObjectTypeRecord {
        name,
        description,
        input_fields,
        oneof,
    })
}

fn insert_directive(
    directive: registry_v1::MetaDirective,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) {
    let registry_v1::MetaDirective {
        name,
        description,
        locations,
        args,
        is_repeatable,
    } = directive;

    let name = writer.intern_string(name);
    let description = description.map(|desc| writer.intern_string(desc));
    let args = insert_input_values(args, writer, type_ids);

    writer.insert_directive(MetaDirectiveRecord {
        name,
        description,
        locations,
        args,
        is_repeatable,
    });
}

fn insert_validators(
    validators: Vec<registry_v2::validators::DynValidator>,
    writer: &mut RegistryWriter,
) -> IdRange<InputValidatorId> {
    let validators = validators
        .into_iter()
        .map(|validator| InputValidatorRecord { validator })
        .collect();

    writer.insert_input_validators(validators)
}

fn convert_meta_field_type(
    ty: registry_v1::MetaFieldType,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaFieldTypeRecord {
    MetaFieldTypeRecord {
        wrappers: wrappers_from_string(ty.as_str()),
        target: lookup_type_id(type_ids, ty.base_type_name()),
    }
}

fn convert_input_value_type(
    ty: registry_v1::InputValueType,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaInputValueTypeRecord {
    MetaInputValueTypeRecord {
        wrappers: wrappers_from_string(ty.as_str()),
        target: lookup_type_id(type_ids, ty.base_type_name()),
    }
}

fn lookup_type_id(type_ids: &HashMap<String, MetaTypeId>, name: &str) -> MetaTypeId {
    *type_ids
        .get(name)
        .unwrap_or_else(|| panic!("Couldn't find type {name}"))
}

fn wrappers_from_string(str: &str) -> TypeWrappers {
    str.chars()
        .rev()
        .take_while(|c| matches!(c, '!' | ']'))
        .map(|c| match c {
            '!' => registry_v2::WrappingType::NonNull,
            ']' => registry_v2::WrappingType::List,
            _ => unreachable!(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_conversions() {
        roundtrip_test("[String!]!", "String");
        roundtrip_test("[String!]", "String");
        roundtrip_test("[String]", "String");
        roundtrip_test("String!", "String");
        roundtrip_test("String", "String");
        roundtrip_test("[String!]", "String");
        roundtrip_test("[String]!", "String");
        roundtrip_test("[[String!]]!", "String");
        roundtrip_test("[[String]]!", "String");
        roundtrip_test("[[String!]]", "String");
    }

    fn roundtrip_test(input: &str, ty: &str) {
        let mut output = String::new();
        wrappers_from_string(input).write_type_string(ty, &mut output).unwrap();

        assert_eq!(input, output);
    }
}
