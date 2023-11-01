mod field_types_map;

use self::field_types_map::FieldTypesMap;
use crate::{
    composition_ir::{CompositionIr, FieldIr, StringsIr},
    subgraphs, Subgraphs, VecExt,
};
use grafbase_federated_graph as federated;
use itertools::Itertools;
use std::collections::BTreeSet;

/// This can't fail. All the relevant, correct information should already be in the CompositionIr.
pub(crate) fn emit_federated_graph(
    ir: CompositionIr,
    subgraphs: &Subgraphs,
) -> federated::FederatedGraph {
    let mut strings_ir = ir.strings;
    let mut field_types_map = FieldTypesMap::new(ir.definitions_by_name);
    let mut out = federated::FederatedGraph {
        enums: ir.enums,
        objects: ir.objects,
        interfaces: ir.interfaces,
        unions: ir.unions,
        scalars: ir.scalars,
        input_objects: ir.input_objects,
        ..Default::default()
    };

    emit_fields(
        &ir.fields,
        subgraphs,
        &mut strings_ir,
        &mut field_types_map,
        &mut out,
    );

    emit_union_members(&ir.union_members, &mut field_types_map, &mut out);

    out.field_types = field_types_map.field_types;
    out.strings = strings_ir.strings;

    out
}

fn emit_fields(
    ir_fields: &[FieldIr],
    subgraphs: &Subgraphs,
    strings: &mut StringsIr,
    field_types_map: &mut FieldTypesMap,
    out: &mut federated::FederatedGraph,
) {
    for FieldIr {
        parent_name,
        field_name,
        field_type,
        arguments,
    } in ir_fields
    {
        let field_type_id = field_types_map.insert(subgraphs.walk(*field_type));
        let field_name = strings.insert(subgraphs.walk(*field_name));
        let arguments = arguments
            .iter()
            .map(|(arg_name, arg_type)| federated::FieldArgument {
                name: strings.insert(subgraphs.walk(*arg_name)),
                type_id: field_types_map.insert(subgraphs.walk(*arg_type)),
            })
            .collect();

        let push_field = |out: &mut Vec<_>| {
            let field = federated::Field {
                name: field_name,
                field_type_id,
                arguments,

                provides: Vec::new(),
                requires: Vec::new(),
                resolvable_in: Vec::new(),
                composed_directives: Vec::new(),
            };

            federated::FieldId(out.push_return_idx(field))
        };

        match field_types_map.definitions[parent_name] {
            federated::Definition::Object(object_id) => {
                let field_id = push_field(&mut out.fields);
                out.object_fields.push(federated::ObjectField {
                    object_id,
                    field_id,
                })
            }
            federated::Definition::Interface(interface_id) => {
                let field_id = push_field(&mut out.fields);
                out.interface_fields.push(federated::InterfaceField {
                    interface_id,
                    field_id,
                })
            }
            federated::Definition::InputObject(input_object_id) => {
                out[input_object_id]
                    .fields
                    .push(federated::InputObjectField {
                        name: field_name,
                        field_type_id,
                    });
            }
            _ => unreachable!(),
        }
    }
}

fn emit_union_members(
    ir_members: &BTreeSet<(subgraphs::StringId, subgraphs::StringId)>,
    field_types_map: &mut FieldTypesMap,
    out: &mut federated::FederatedGraph,
) {
    for (union_name, members) in &ir_members.iter().group_by(|(union_name, _)| union_name) {
        let federated::Definition::Union(union_id) = field_types_map.definitions[union_name] else {
            continue;
        };
        let union = &mut out[union_id];

        for (_, member) in members {
            let federated::Definition::Object(object_id) = field_types_map.definitions[member]
            else {
                continue;
            };
            union.members.push(object_id);
        }
    }
}
