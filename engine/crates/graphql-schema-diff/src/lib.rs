#![allow(unused_crate_dependencies)]

use async_graphql_parser::types as ast;
use indexmap::IndexSet;
use std::collections::HashMap;

type Paths = Box<[usize]>;

#[derive(Debug, Default)]
pub struct Diff {
    added_types: Paths,
    removed_types: Paths,
    added_fields: Paths,
    removed_fields: Paths,
    added_enum_variants: Paths,
    removed_enum_variants: Paths,
    path_segments: IndexSet<Box<str>>,
}

struct DiffState<'a> {
    source: &'a ast::ServiceDocument,
    target: &'a ast::ServiceDocument,
    added_types: Vec<&'a str>,
    removed_types: Vec<&'a str>,
    added_fields: Vec<(&'a str, &'a str)>,
    removed_fields: Vec<(&'a str, &'a str)>,
    added_enum_variants: Vec<(&'a str, &'a str)>,
    removed_enum_variants: Vec<(&'a str, &'a str)>,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum DefinitionKind {
    Directive,
    Enum,
    InputObject,
    Interface,
    Object,
    Scalar,
    Schema,
    Union,
}

impl DiffState<'_> {
    fn into_diff(self) -> Diff {
        let mut path_segments = IndexSet::new();
        let mut insert_segment = |segment: &str| match path_segments.get_full(segment) {
            Some((idx, _)) => idx,
            None => path_segments.insert_full(segment.to_owned().into_boxed_str()).0,
        };

        Diff {
            added_types: self.added_types.into_iter().map(&mut insert_segment).collect(),
            removed_types: self.removed_types.into_iter().map(&mut insert_segment).collect(),
            added_fields: Vec::new().into_boxed_slice(),
            removed_fields: Vec::new().into_boxed_slice(),
            added_enum_variants: Vec::new().into_boxed_slice(),
            removed_enum_variants: Vec::new().into_boxed_slice(),
            path_segments,
        }
    }
}

pub fn diff(source: &str, target: &str) -> Result<Diff, async_graphql_parser::Error> {
    let source = async_graphql_parser::parse_schema(source)?;
    let target = async_graphql_parser::parse_schema(target)?;
    let mut state = DiffState {
        source: &source,
        target: &target,
        added_types: Vec::new(),
        removed_types: Vec::new(),
        added_fields: Vec::new(),
        removed_fields: Vec::new(),
        added_enum_variants: Vec::new(),
        removed_enum_variants: Vec::new(),
    };

    let schema_size_approx = source.definitions.len().max(target.definitions.len());

    let mut types_map: HashMap<&str, (Option<DefinitionKind>, Option<DefinitionKind>)> =
        HashMap::with_capacity(schema_size_approx);
    let mut fields_map: HashMap<(&str, &str), (Option<usize>, Option<usize>)> =
        HashMap::with_capacity(schema_size_approx);

    for (idx, tpe) in source.definitions.iter().enumerate() {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => todo!(),
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        types_map.insert(type_name, (Some(DefinitionKind::Scalar), None));
                    }
                    ast::TypeKind::Object(obj) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Object), None));

                        for field in &obj.fields {
                            fields_map.insert((type_name, field.node.name.node.as_str()), (Some(idx), None));
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            fields_map.insert((type_name, field.node.name.node.as_str()), (Some(idx), None));
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in &union.members {
                            fields_map.insert((type_name, member.node.as_str()), (Some(idx), None));
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in &enm.values {
                            fields_map.insert((type_name, value.node.value.node.as_str()), (Some(idx), None));
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        for field in &input.fields {
                            fields_map.insert((type_name, field.node.name.node.as_str()), (Some(idx), None));
                        }
                    }
                }
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(_) => todo!(),
        }
    }

    for (idx, tpe) in target.definitions.iter().enumerate() {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => todo!(),
            async_graphql_parser::types::TypeSystemDefinition::Directive(_) => todo!(),
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Scalar);
                    }
                    ast::TypeKind::Object(obj) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Object);

                        for field in &obj.fields {
                            fields_map
                                .entry((type_name, field.node.name.node.as_str()))
                                .or_default()
                                .1 = Some(idx);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);

                        for field in &iface.fields {
                            fields_map
                                .entry((type_name, field.node.name.node.as_str()))
                                .or_default()
                                .1 = Some(idx);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in &union.members {
                            fields_map.entry((type_name, member.node.as_str())).or_default().1 = Some(idx);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in &enm.values {
                            fields_map
                                .entry((type_name, value.node.value.node.as_str()))
                                .or_default()
                                .1 = Some(idx);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in &input.fields {
                            fields_map
                                .entry((type_name, field.node.name.node.as_str()))
                                .or_default()
                                .1 = Some(idx);
                        }
                    }
                }
            }
        }
    }

    for (name, entries) in types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(_)) => state.added_types.push(name),
            (Some(_), None) => state.removed_types.push(name),
            (Some(a), Some(b)) if a != b => {
                state.added_types.push(name);
                state.removed_types.push(name);
            }
            (Some(_), Some(_)) => (),
        }
    }

    for ((type_name, field_name), entries) in fields_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(_)) => state.added_fields.push((type_name, field_name)),
            (Some(_), None) => state.removed_fields.push((type_name, field_name)),
            (Some(_), Some(_)) => (),
        }
    }

    Ok(state.into_diff())
}
