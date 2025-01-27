use super::ValidateContext;
use std::fmt::Write as _;

pub(crate) fn validate_compose_directive(ctx: &mut ValidateContext<'_>) {
    let directive_definitions = ctx.subgraphs.directive_definitions();
    let mut composed_directive_definitions_sorted_by_name: Vec<usize> = directive_definitions
        .iter()
        .enumerate()
        .filter(|(_, def)| ctx.subgraphs.is_composed_directive(def.name))
        .map(|(idx, _)| idx)
        .collect();

    composed_directive_definitions_sorted_by_name.sort_unstable_by_key(|&idx| directive_definitions[idx].name);

    for chunk in composed_directive_definitions_sorted_by_name
        .chunk_by(|idx_1, idx_2| directive_definitions[*idx_1].name == directive_definitions[*idx_2].name)
    {
        let first_definition = &directive_definitions[chunk[0]];

        for definition_idx in &chunk[1..] {
            let definition = &directive_definitions[*definition_idx];
            debug_assert_eq!(definition.name, first_definition.name);

            if definition.locations != first_definition.locations {
                let mut diagnostic = format!(
                    "Directive `{}` is defined with different locations:\n",
                    ctx.subgraphs.walk(first_definition.name).as_str()
                );

                for def in [first_definition, definition] {
                    writeln!(
                        diagnostic,
                        " {} in {}",
                        def.locations,
                        ctx.subgraphs.walk_subgraph(def.subgraph_id).name().as_str(),
                    )
                    .unwrap();
                }

                ctx.diagnostics.push_fatal(diagnostic);
            }

            if definition.arguments != first_definition.arguments {
                let mut diagnostic = format!(
                    "Directive `{}` is defined with different arguments:\n",
                    ctx.subgraphs.walk(first_definition.name).as_str()
                );

                for def in [first_definition, definition] {
                    writeln!(
                        diagnostic,
                        "- ({}) in {}",
                        def.arguments
                            .iter()
                            .cloned()
                            .map(|arg| ctx.subgraphs.walk(arg).to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        ctx.subgraphs.walk_subgraph(def.subgraph_id).name().as_str(),
                    )
                    .unwrap();
                }

                ctx.diagnostics.push_fatal(diagnostic);
            }
        }
    }
}
