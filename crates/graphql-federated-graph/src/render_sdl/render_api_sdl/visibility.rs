use crate::Directive;

pub(super) fn is_inaccessible(directives: &[Directive]) -> bool {
    if directives
        .iter()
        .any(|directive| matches!(directive, Directive::Inaccessible))
    {
        return true;
    }

    let at_least_one_internal = directives
        .iter()
        .any(|d| matches!(d, Directive::CompositeInternal { .. }));

    at_least_one_internal
        && directives.iter().all(|directive| {
            let subgraph_id = match directive {
                Directive::JoinField(join_field_directive) => {
                    let Some(subgraph_id) = join_field_directive.subgraph_id else {
                        return true;
                    };

                    subgraph_id
                }
                Directive::JoinType(join_type_directive) => join_type_directive.subgraph_id,
                _ => return true,
            };

            directives.iter().any(|directive| {
                if let Directive::CompositeInternal {
                    graph: source_schema_id,
                } = directive
                {
                    subgraph_id == *source_schema_id
                } else {
                    false
                }
            })
        })
}
