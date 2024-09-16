use std::{collections::HashMap, path::PathBuf};

use cynic_parser::type_system::{iter::Iter, Definition, Directive, TypeDefinition, Value};

use crate::domain::{self};

pub(super) fn load(path: PathBuf) -> anyhow::Result<domain::Domain> {
    let sdl = std::fs::read_to_string(&path)?;
    let document = match cynic_parser::parse_type_system_document(&sdl) {
        Ok(document) => document,
        Err(error) => {
            println!("Error parsing document");
            println!("{}", error.to_report(&sdl));
            return Err(anyhow::anyhow!(""));
        }
    };

    let mut domain: Option<domain::Domain> = None;
    let mut definitions_by_name = HashMap::new();

    for definition in document.definitions() {
        let Definition::Type(ty) = definition else {
            anyhow::bail!("unsupported definition");
        };

        if let Some(directive) = ty.directives().find(|d| d.name() == "graph") {
            assert!(domain.is_none(), "Only one scalar can have the directive @graph");
            let dir = env!("CARGO_MANIFEST_DIR");
            domain = Some(domain::Domain {
                sdl: sdl.clone(),
                source: path.strip_prefix(dir).unwrap().to_path_buf(),
                destination_path: {
                    let path = directive
                        .arguments()
                        .find(|arg| arg.name() == "destination")
                        .and_then(|arg| arg.value().as_str())
                        .expect("Missing destination in @graph")
                        .to_string();
                    assert!(!path.is_empty(), "Missing or empty destination in @graph");
                    path.into()
                },
                root_module: directive
                    .arguments()
                    .find(|arg| arg.name() == "root_module")
                    .and_then(|arg| arg.value().as_str())
                    .map(|value| value.split('/').map(str::to_string).collect::<Vec<_>>())
                    .unwrap_or_default(),
                graph_var_name: ty.name().to_lowercase(),
                graph_type_name: ty.name().to_string(),
                definitions_by_name: Default::default(),
            });
            continue;
        };

        let def: domain::Definition = match ty {
            TypeDefinition::Scalar(scalar) => domain::Scalar {
                indexed: parse_indexed(scalar.name(), scalar.directives()),
                span: scalar.span(),
                name: scalar.name().to_string(),
                struct_name: if is_record(scalar.directives()) {
                    format!("{}Record", scalar.name())
                } else {
                    scalar.name().to_string()
                },
                is_record: is_record(scalar.directives()),
                copy: is_copy(scalar.directives()),
            }
            .into(),
            TypeDefinition::Object(object) => domain::Object {
                meta: parse_meta(object.directives()).unwrap_or_default(),
                indexed: parse_indexed(object.name(), object.directives()),
                span: object.span(),
                name: object.name().to_string(),
                struct_name: format!("{}Record", object.name()),
                copy: is_copy(object.directives()),
                fields: object
                    .fields()
                    .map(|field| domain::Field {
                        meta: parse_field_meta(field.directives()).unwrap_or_default(),
                        name: field.name().to_string(),
                        // Add any explicitly defined field name or leave empty to be generated
                        // afterwards.
                        record_field_name: parse_record_field_name(field.directives()).unwrap_or_default(),
                        description: field.description().map(|s| s.description().to_cow().into_owned()),
                        type_name: field.ty().name().to_string(),
                        wrapping: field.ty().wrappers().collect(),
                    })
                    .collect(),
            }
            .into(),
            TypeDefinition::Union(union) => domain::Union {
                meta: parse_meta(union.directives()).unwrap_or_default(),
                span: union.span(),
                kind: parse_union_kind(union.name(), union.directives()),
                variants: {
                    let variant = parse_variants(union.directives()).unwrap_or_default();

                    let mut variants = Vec::new();
                    for (index, member) in union.members().enumerate() {
                        variants.push(domain::Variant {
                            index,
                            name: {
                                if let Some(name) = variant.names.as_ref().and_then(|names| names.get(index)) {
                                    name.to_string()
                                } else {
                                    let name = member.name();
                                    match &variant.remove_suffix {
                                        Ok(false) => name,
                                        Ok(true) => name.strip_suffix(union.name()).unwrap(),
                                        Err(suffix) => name.strip_suffix(suffix).unwrap(),
                                    }
                                    .to_string()
                                }
                            },
                            value_type_name: Some(member.name().to_string()),
                        });
                    }

                    for name in variant.empty {
                        variants.push(domain::Variant {
                            index: variants.len(),
                            name,
                            value_type_name: None,
                        });
                    }
                    variants.sort_by(|a, b| a.name.cmp(&b.name));
                    for (index, variant) in variants.iter_mut().enumerate() {
                        variant.index = index;
                    }
                    variants
                },
            }
            .into(),
            _ => anyhow::bail!("unsupported type {}", ty.name()),
        };
        definitions_by_name.insert(def.name().to_string(), def);
    }

    let mut domain = domain.expect("Missing scalar with @graph directive");
    domain.definitions_by_name = finalize_field_struct_names(definitions_by_name);

    Ok(domain)
}

fn finalize_field_struct_names(
    mut definitions_by_name: HashMap<String, domain::Definition>,
) -> HashMap<String, domain::Definition> {
    let suffixes = definitions_by_name
        .iter()
        .map(|(name, definition)| {
            let suffix = match definition {
                domain::Definition::Union(union) => match &union.kind {
                    domain::UnionKind::Record(union) => {
                        if union.indexed.is_some() {
                            Some("id")
                        } else {
                            Some("value")
                        }
                    }
                    domain::UnionKind::Id(_) | domain::UnionKind::BitpackedId(_) => Some("id"),
                },
                domain::Definition::Scalar(scalar) => {
                    if scalar.indexed.is_some() {
                        Some("id")
                    } else if scalar.is_record {
                        Some("record")
                    } else {
                        // We don't generate a walker for those fields. The Deref & as_ref()
                        // are enough.
                        None
                    }
                }
                domain::Definition::Object(object) => {
                    if object.indexed.is_some() {
                        Some("id")
                    } else {
                        Some("record")
                    }
                }
            };
            (name.to_string(), suffix)
        })
        .collect::<HashMap<_, _>>();

    for definition in definitions_by_name.values_mut() {
        let domain::Definition::Object(ref mut object) = definition else {
            continue;
        };
        for field in &mut object.fields {
            if !field.record_field_name.is_empty() {
                continue;
            }
            let Some(suffix) = suffixes.get(&field.type_name).cloned().flatten() else {
                field.record_field_name = field.name.clone();
                continue;
            };
            let record_field_name = if field.has_list_wrapping() {
                format!("{}_{suffix}s", field.name.strip_suffix("s").unwrap_or(&field.name))
            } else {
                format!("{}_{suffix}", field.name)
            };
            field.record_field_name = record_field_name;
        }
    }

    definitions_by_name
}

fn parse_union_kind(name: &str, directives: Iter<'_, Directive<'_>>) -> domain::UnionKind {
    if let Some(directive) = directives.clone().find(|directive| directive.name() == "id") {
        if let Some(bitpacked_size) = directive
            .arguments()
            .find(|arg| arg.name() == "bitpacked_size")
            .and_then(|arg| arg.value().as_str())
        {
            domain::UnionKind::BitpackedId(domain::BitPackedIdUnion {
                name: name.to_string(),
                size: bitpacked_size.to_string(),
                enum_name: format!("BitPacked{name}Id"),
            })
        } else {
            domain::UnionKind::Id(domain::IdUnion {
                name: name.to_string(),
                enum_name: format!("{name}Id"),
            })
        }
    } else {
        domain::UnionKind::Record(domain::RecordUnion {
            indexed: parse_indexed(name, directives),
            copy: is_copy(directives),
            name: name.to_string(),
            walker_enum_name: format!("{name}Variant"),
            enum_name: format!("{name}Record"),
        })
    }
}

struct VariantDirective {
    // Result used as a Either
    remove_suffix: Result<bool, String>,
    empty: Vec<String>,
    names: Option<Vec<String>>,
}

impl Default for VariantDirective {
    fn default() -> Self {
        Self {
            remove_suffix: Ok(false),
            empty: Default::default(),
            names: Default::default(),
        }
    }
}

fn parse_variants(mut directives: Iter<'_, Directive<'_>>) -> Option<VariantDirective> {
    let directive = directives.find(|directive| directive.name() == "variants")?;
    let remove_suffix = directive
        .arguments()
        .find(|arg| arg.name() == "remove_suffix")
        .and_then(|arg| match arg.value() {
            Value::Boolean(value) => Some(Ok(value)),
            Value::String(value) => Some(Err(value.to_string())),
            _ => None,
        })
        .unwrap_or(VariantDirective::default().remove_suffix);
    let empty = directive
        .arguments()
        .find(|arg| arg.name() == "empty")
        .and_then(|arg| match arg.value() {
            Value::List(values) => Some(
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();
    let names = directive
        .arguments()
        .find(|arg| arg.name() == "names")
        .and_then(|arg| match arg.value() {
            Value::List(values) => Some(
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect(),
            ),
            _ => None,
        });
    Some(VariantDirective {
        remove_suffix,
        empty,
        names,
    })
}

fn parse_record_field_name(mut directives: Iter<'_, Directive<'_>>) -> Option<String> {
    directives
        .find(|directive| directive.name() == "field")
        .and_then(|directive| {
            directive
                .arguments()
                .find(|arg| arg.name() == "record_field_name")
                .and_then(|arg| arg.value().as_str())
                .map(str::to_string)
        })
}

fn parse_field_meta(mut directives: Iter<'_, Directive<'_>>) -> Option<domain::FieldMeta> {
    let directive = directives.find(|directive| directive.name() == "meta")?;

    let debug = directive
        .arguments()
        .find(|arg| arg.name() == "debug")
        .and_then(|arg| match arg.value() {
            Value::Boolean(value) => Some(value),
            _ => None,
        })
        .unwrap_or(true);

    Some(domain::FieldMeta { debug })
}

fn parse_meta(mut directives: Iter<'_, Directive<'_>>) -> Option<domain::Meta> {
    let directive = directives.find(|directive| directive.name() == "meta")?;
    let derive = directive
        .arguments()
        .find(|arg| arg.name() == "derive")
        .and_then(|arg| match arg.value() {
            Value::List(values) => Some(
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();

    let debug = directive
        .arguments()
        .find(|arg| arg.name() == "debug")
        .and_then(|arg| match arg.value() {
            Value::Boolean(value) => Some(value),
            _ => None,
        })
        .unwrap_or(true);

    let module_path = directive
        .arguments()
        .find(|arg| arg.name() == "module")
        .and_then(|arg| arg.value().as_str())
        .map(|value| value.split('/').map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();

    assert!(!module_path.is_empty(), "Missing or empty module in @meta");

    Some(domain::Meta {
        derive,
        module_path,
        debug,
    })
}

fn parse_indexed(name: &str, mut directives: Iter<'_, Directive<'_>>) -> Option<domain::Indexed> {
    let directive = directives.find(|directive| directive.name() == "indexed")?;
    let id_size = directive
        .arguments()
        .find(|arg| arg.name() == "id_size")
        .and_then(|arg| arg.value().as_str().map(str::to_string));
    let max_id = directive
        .arguments()
        .find(|arg| arg.name() == "max_id")
        .and_then(|arg| arg.value().as_str().map(str::to_string));
    let deduplicated = directive
        .arguments()
        .find(|arg| arg.name() == "deduplicated")
        .and_then(|arg| match arg.value() {
            cynic_parser::type_system::Value::Boolean(b) => Some(b),
            _ => None,
        })
        .unwrap_or_default();
    Some(domain::Indexed {
        id_struct_name: format!("{name}Id"),
        id_size,
        max_id,
        deduplicated,
    })
}

fn is_copy(mut directives: Iter<'_, Directive<'_>>) -> bool {
    directives.any(|directive| directive.name() == "copy")
}

fn is_record(mut directives: Iter<'_, Directive<'_>>) -> bool {
    directives.any(|directive| directive.name() == "record")
}
