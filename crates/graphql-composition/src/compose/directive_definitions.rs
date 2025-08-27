use crate::federated_graph::DirectiveLocations;

use super::*;
use std::fmt::Write as _;

pub(super) fn compose_directive_definitions(ctx: &mut Context<'_>) {
    // Filtered definitions. Sort by name, dedup.
    let mut definitions: Vec<&subgraphs::DirectiveDefinition> = ctx
        .subgraphs
        .directive_definitions()
        .iter()
        .filter(|definition| {
            ctx.subgraphs
                .is_composed_directive(definition.subgraph_id, definition.name)
        })
        .collect();

    definitions.sort_unstable_by_key(|definition| definition.name);

    let mut chunk = Vec::new();
    'directives: for (name, definitions) in definitions.into_iter().chunk_by(|def| def.name).into_iter() {
        chunk.clear();
        chunk.extend(definitions);

        let name = ctx.insert_string(name);

        let first_definition = chunk
            .first()
            .expect("There should be at least one definition for each name");

        // == Location ==
        for definition in chunk[1..].iter().copied() {
            if definition.locations != first_definition.locations {
                let mut diagnostic = format!(
                    "Directive `{}` is defined with different locations:\n",
                    ctx.subgraphs[first_definition.name].as_ref()
                );

                for def in [first_definition, definition] {
                    writeln!(
                        diagnostic,
                        " {} in {}",
                        def.locations,
                        ctx.subgraphs[ctx.subgraphs.at(def.subgraph_id).name].as_ref(),
                    )
                    .unwrap();
                }

                ctx.diagnostics.push_warning(diagnostic);
            }
        }

        let locations = chunk.iter().fold(DirectiveLocations::empty(), |location, dir| {
            location.union(dir.locations)
        });

        // == Arguments ==
        let repeatable = first_definition.repeatable;
        let mut arguments = Vec::<ir::InputValueDefinitionIr>::with_capacity(first_definition.arguments.len());
        for (ix, definition) in chunk.iter().copied().enumerate() {
            for argument in &definition.arguments {
                if arguments
                    .iter()
                    .find(|arg| ctx[arg.name] == ctx[argument.name])
                    .map(|arg| arg.r#type != argument.r#type || arg.default != argument.default_value)
                    .unwrap_or(argument.r#type.wrapping.is_non_null() && ix != 0)
                {
                    let mut diagnostic = format!(
                        "Directive `{}` is defined with incompatible arguments:\n",
                        ctx.subgraphs[first_definition.name].as_ref(),
                    );

                    for def in [first_definition, definition] {
                        writeln!(
                            diagnostic,
                            "- ({}) in {}",
                            def.arguments
                                .iter()
                                .cloned()
                                .map(|arg| arg.display(ctx.subgraphs).to_string())
                                .join(", "),
                            ctx.subgraphs[ctx.subgraphs.at(def.subgraph_id).name].as_ref(),
                        )
                        .unwrap();
                    }

                    ctx.diagnostics.push_fatal(diagnostic);
                    continue 'directives;
                } else {
                    let input_value_definition = ir::InputValueDefinitionIr {
                        name: ctx.insert_string(argument.name),
                        r#type: argument.r#type,
                        // Directive argument definitions cannot have directives applied on them.
                        directives: argument
                            .directives
                            .iter()
                            .map(|directive| ir::Directive::Other {
                                provenance: ir::DirectiveProvenance::Builtin,
                                name: ctx.insert_string(directive.name),
                                arguments: directive
                                    .arguments
                                    .iter()
                                    .map(|(name, value)| (ctx.insert_string(*name), value.clone()))
                                    .collect(),
                            })
                            .collect(),
                        description: None,
                        default: argument.default_value.clone(),
                    };

                    arguments.push(input_value_definition);
                }
            }
            if definition.arguments != first_definition.arguments {
                let mut diagnostic = format!(
                    "Directive `{}` is defined with different arguments:\n",
                    ctx.subgraphs[first_definition.name].as_ref()
                );

                for def in [first_definition, definition] {
                    writeln!(
                        diagnostic,
                        "- ({}) in {}",
                        def.arguments
                            .iter()
                            .cloned()
                            .map(|arg| arg.display(ctx.subgraphs).to_string())
                            .join(", "),
                        ctx.subgraphs[ctx.subgraphs.at(def.subgraph_id).name].as_ref(),
                    )
                    .unwrap();
                }

                ctx.diagnostics.push_warning(diagnostic);
            }
            if definition.repeatable != first_definition.repeatable {
                ctx.diagnostics.push_fatal(format!(
                    "Directive `{}` is defined as repeatable in {} but not in {}.",
                    ctx.subgraphs[first_definition.name].as_ref(),
                    ctx.subgraphs[ctx.subgraphs.at(definition.subgraph_id).name].as_ref(),
                    ctx.subgraphs[ctx.subgraphs.at(first_definition.subgraph_id).name].as_ref(),
                ));
                continue 'directives;
            }
        }

        ctx.insert_directive_definition(ir::DirectiveDefinitionIr {
            name,
            locations,
            arguments,
            repeatable,
        });
    }
}
