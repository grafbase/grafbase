#![allow(unused_crate_dependencies)]

use async_graphql_parser::types as ast;
use indexmap::IndexSet;
use std::collections::HashMap;

type Paths = Box<[usize]>;

#[derive(Debug, Default)]
struct Item<T> {
    added: T,
    removed: T,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct Presence(u8);

impl Presence {
    const SOURCE: Presence = Presence(0b1);
    const TARGET: Presence = Presence(0b10);
    const BOTH: Presence = Presence(0b11);

    fn add_target(&mut self) {
        *self = Presence(self.0 | Self::TARGET.0)
    }
}

#[derive(Debug, Default)]
pub struct Diff {
    objects: Item<Paths>,
    fields: Item<Paths>,
    arguments: Item<Paths>,
    enum_variants: Item<Paths>,
    union_members: Item<Paths>,
    path_segments: IndexSet<Box<str>>,
}

#[derive(Debug, Default)]
struct DefinitionDiff {
    added: Paths,
    removed: Paths,
}

struct DiffState<'a> {
    source: &'a ast::ServiceDocument,
    target: &'a ast::ServiceDocument,
    definitions: Item<Definitions<'a>>,
    fields: Item<Vec<(&'a str, &'a str)>>,
    enum_variants: Item<Vec<(&'a str, &'a str)>>,
    union_members: Item<Vec<(&'a str, &'a str)>>,
    arguments: Item<Vec<(&'a str, &'a str, &'a str)>>,
}

macro_rules! definition_kinds {
    ($($camel:ident, $snake:ident);*) => {
            #[derive(Debug, PartialEq, Eq)]
            #[repr(u8)]
            enum DefinitionKind {
                $(
                    $camel,
                )*
            }

            #[derive(Default)]
            struct Definitions<'a> {
                $(
                    $snake: Vec<&'a str>,
                )*
            }

            impl<'a> DiffState<'a> {
                fn push_added_type(&mut self, name: &'a str, kind: DefinitionKind) {
                    match kind {
                        $(
                            DefinitionKind::$camel => self.definitions.added.$snake.push(name),
                        )*
                    }
                }

                fn push_removed_type(&mut self, name: &'a str, kind: DefinitionKind) {
                    match kind {
                        $(
                            DefinitionKind::$camel => self.definitions.removed.$snake.push(name),
                        )*
                    }
                }
            }
    }
}

definition_kinds! {
    Directive, directive;
    Enum, r#enum;
    InputObject, input_object;
    Interface, interface;
    Object, object;
    Scalar, scalar;
    Schema, schema;
    Union, union
}

impl DiffState<'_> {
    fn into_diff(self) -> Diff {
        let mut path_segments = IndexSet::new();
        let mut insert_segment = |segment: &str| match path_segments.get_full(segment) {
            Some((idx, _)) => idx,
            None => path_segments.insert_full(segment.to_owned().into_boxed_str()).0,
        };

        Diff {
            objects: Item {
                added: self
                    .definitions
                    .added
                    .object
                    .into_iter()
                    .map(|name| insert_segment(name))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                removed: self
                    .definitions
                    .removed
                    .object
                    .into_iter()
                    .map(|name| insert_segment(name))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            fields: Default::default(),
            arguments: Default::default(),
            enum_variants: Default::default(),
            union_members: Default::default(),
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
        definitions: Default::default(),
        fields: Default::default(),
        enum_variants: Default::default(),
        union_members: Default::default(),
        arguments: Default::default(),
    };

    let schema_size_approx = source.definitions.len().max(target.definitions.len());

    let mut types_map: HashMap<&str, (Option<DefinitionKind>, Option<DefinitionKind>)> =
        HashMap::with_capacity(schema_size_approx);
    let mut fields_map: HashMap<(&str, &str), Presence> = HashMap::with_capacity(schema_size_approx);
    let mut arguments_map: HashMap<(&str, &str, &str), Presence> = HashMap::with_capacity(schema_size_approx);

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
                            fields_map.insert((type_name, field.node.name.node.as_str()), Presence::SOURCE);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            fields_map.insert((type_name, field.node.name.node.as_str()), Presence::SOURCE);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in &union.members {
                            fields_map.insert((type_name, member.node.as_str()), Presence::SOURCE);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in &enm.values {
                            fields_map.insert((type_name, value.node.value.node.as_str()), Presence::SOURCE);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        for field in &input.fields {
                            fields_map.insert((type_name, field.node.name.node.as_str()), Presence::SOURCE);
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
                                .add_target();
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);

                        for field in &iface.fields {
                            fields_map
                                .entry((type_name, field.node.name.node.as_str()))
                                .or_default()
                                .add_target();
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in &union.members {
                            fields_map
                                .entry((type_name, member.node.as_str()))
                                .or_default()
                                .add_target();
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in &enm.values {
                            fields_map
                                .entry((type_name, value.node.value.node.as_str()))
                                .or_default()
                                .add_target();
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in &input.fields {
                            fields_map
                                .entry((type_name, field.node.name.node.as_str()))
                                .or_default()
                                .add_target();
                        }
                    }
                }
            }
        }
    }

    for (name, entries) in types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(kind)) => state.push_added_type(name, kind),
            (Some(kind), None) => state.push_removed_type(name, kind),
            (Some(a), Some(b)) if a != b => {
                state.push_removed_type(name, a);
                state.push_added_type(name, b);
            }
            (Some(_), Some(_)) => (),
        }
    }

    for (path @ (type_name, field_name), presence) in fields_map {
        let item: &mut Item<Vec<(_, _)>> = match types_map[type_name] {
            (None, None) => unreachable!(),
            (None, Some(_)) => todo!(),
            (Some(_), None) => todo!(),
            (Some(_), Some(_)) => todo!(),
        };

        match presence {
            Presence::SOURCE => state.fields.removed.push(path),
            Presence::TARGET => state.fields.added.push(path),
            _ => (),
        }
    }

    Ok(state.into_diff())
}
