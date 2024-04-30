use std::collections::HashMap;

use indexmap::IndexMap;
use registry_for_cache::{ids::*, storage::*, writer::RegistryWriter, IdRange};
use wrapping::Wrapping;

pub fn convert_v1_to_partial_cache_registry(v1: registry_v1::Registry) -> registry_for_cache::PartialCacheRegistry {
    let mut writer = RegistryWriter::new();

    let registry_v1::Registry {
        types,
        directives: _,
        implements: _,
        query_type,
        mutation_type,
        subscription_type,
        disable_introspection: _,
        enable_federation: _,
        federation_subscription: _,
        auth: _,
        mongodb_configurations: _,
        http_headers: _,
        postgres_databases: _,
        enable_caching,
        enable_kv: _,
        federation_entities: _,
        enable_ai: _,
        enable_codegen: _,
        codegen: _,
        is_federated: _,
        operation_limits: _,
        trusted_documents: _,
        cors_config: _,
    } = v1;

    let types = {
        let mut types = types.into_values().collect::<Vec<_>>();

        // Comes out of a BTreeMap so should be sorted, but it's important
        // so lets sort incase the type changes.
        types.sort_by(|lhs, rhs| lhs.name().cmp(rhs.name()));
        types
    };

    // Build a map of type name -> the ID it'll have when we insert it.
    let preallocated_ids = writer.preallocate_type_ids(types.len()).collect::<Vec<_>>();
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

    writer.enable_caching = enable_caching;

    writer.finish().unwrap()
}

fn insert_type(
    ty: registry_v1::MetaType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    match ty {
        registry_v1::MetaType::Object(inner) => insert_object(inner, writer, type_ids),
        registry_v1::MetaType::Interface(inner) => insert_interface(inner, writer, type_ids),
        other => insert_other(other, writer, type_ids),
    }
}

fn insert_object(
    inner: registry_v1::ObjectType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::ObjectType {
        name,
        description: _,
        fields,
        cache_control,
        extends: _,
        is_subscription: _,
        is_node: _,
        rust_typename: _,
        constraints: _,
        external: _,
        shareable: _,
    } = inner;

    let name = writer.intern_string(name);

    let fields = insert_fields(fields, writer, type_ids);

    writer.insert_object(ObjectTypeRecord {
        name,
        fields,
        cache_control,
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
                args: _,
                ty,
                deprecation: _,
                cache_control,
                requires: _,
                federation: _,
                resolver: _,
                required_operation: _,
                auth: _,
                mapped_name: _,
                description: _,
            } = field;

            let name = writer.intern_string(name);
            let ty = convert_meta_field_type(ty, type_ids);

            MetaFieldRecord {
                name,
                ty,
                cache_control,
            }
        })
        .collect();

    writer.insert_fields(fields)
}

fn insert_interface(
    inner: registry_v1::InterfaceType,
    writer: &mut RegistryWriter,
    type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let registry_v1::InterfaceType {
        name,
        description: _,
        fields,
        cache_control,
        possible_types,
        extends: _,
        rust_typename: _,
    } = inner;

    let name = writer.intern_string(name);

    let fields = insert_fields(fields, writer, type_ids);
    let possible_types = possible_types
        .into_iter()
        .map(|ty| lookup_type_id(type_ids, &ty))
        .collect();

    writer.insert_interface(InterfaceTypeRecord {
        name,
        fields,
        cache_control,
        possible_types,
    })
}

fn insert_other(
    inner: registry_v1::MetaType,
    writer: &mut RegistryWriter,
    _type_ids: &HashMap<String, MetaTypeId>,
) -> MetaTypeRecord {
    let name = writer.intern_str(inner.name());

    writer.insert_other(OtherTypeRecord { name })
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

fn lookup_type_id(type_ids: &HashMap<String, MetaTypeId>, name: &str) -> MetaTypeId {
    *type_ids
        .get(name)
        .unwrap_or_else(|| panic!("Couldn't find type {name}"))
}

fn wrappers_from_string(str: &str) -> Wrapping {
    let wrapping_chars = str
        .chars()
        .rev()
        .take_while(|c| matches!(c, '!' | ']'))
        .collect::<Vec<_>>();

    let mut iter = wrapping_chars.into_iter().rev().peekable();

    let mut rv: Wrapping;
    if matches!(iter.peek(), Some('!')) {
        rv = Wrapping::new(true);
        iter.next();
    } else {
        rv = Wrapping::new(false);
    }

    while let Some(char) = iter.next() {
        assert_eq!(char, ']');
        match iter.peek() {
            Some('!') => {
                rv = rv.wrapped_by_required_list();
                iter.next();
            }
            _ => rv = rv.wrapped_by_nullable_list(),
        }
    }

    rv
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
