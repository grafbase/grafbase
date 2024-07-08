use config::latest::OperationLimits;
use engine::Positioned;

use crate::operation::parse::ParsedOperation;

use super::{BindError, BindResult};

pub(super) fn validate_parsed_operation(operation: &ParsedOperation, limits: &OperationLimits) -> BindResult<()> {
    Visitor {
        operation,
        current_fragments_stack: Vec::new(),
        root_fields: 0,
        max_root_fields: limits.root_fields.map(Into::into).unwrap_or(usize::MAX),
        current_depth: 0,
        max_depth: limits.depth.map(Into::into).unwrap_or(usize::MAX),
        aliases_count: 0,
        max_aliases_count: limits.aliases.map(Into::into).unwrap_or(usize::MAX),
        complexity: 0,
        max_complexity: limits.complexity.map(Into::into).unwrap_or(usize::MAX),
    }
    .visit_selection_set(&operation.definition.selection_set)
}

struct Visitor<'p> {
    operation: &'p ParsedOperation,
    current_fragments_stack: Vec<&'p str>,
    root_fields: usize,
    max_root_fields: usize,
    current_depth: usize,
    max_depth: usize,
    aliases_count: usize,
    max_aliases_count: usize,
    complexity: usize,
    max_complexity: usize,
}

impl<'p> Visitor<'p> {
    fn visit_selection_set(
        &mut self,
        selection_set: &'p Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<()> {
        for item in &selection_set.items {
            match &item.node {
                engine_parser::types::Selection::Field(field) => {
                    self.root_fields += (self.current_depth == 0) as usize;
                    if self.root_fields > self.max_root_fields {
                        return Err(BindError::QueryContainsTooManyRootFields {
                            count: self.root_fields,
                            location: selection_set.pos.try_into()?,
                        });
                    }
                    self.complexity += 1;
                    if self.complexity > self.max_complexity {
                        return Err(BindError::QueryTooComplex {
                            complexity: self.complexity,
                            location: selection_set.pos.try_into()?,
                        });
                    }
                    self.visit_field(field)?;
                }
                engine_parser::types::Selection::FragmentSpread(fragment_spread) => {
                    self.visit_fragment_spread(fragment_spread)?;
                }
                engine_parser::types::Selection::InlineFragment(inline_fragment) => {
                    self.visit_inline_fragment(inline_fragment)?;
                }
            }
        }

        Ok(())
    }

    fn visit_field(&mut self, field: &'p Positioned<engine_parser::types::Field>) -> BindResult<()> {
        self.aliases_count += field.alias.is_some() as usize;
        if self.aliases_count > self.max_aliases_count {
            return Err(BindError::QueryContainsTooManyAliases {
                count: self.aliases_count,
                location: field.pos.try_into()?,
            });
        }
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(BindError::QueryTooDeep {
                depth: self.current_depth,
                location: field.selection_set.pos.try_into()?,
            });
        }

        self.visit_selection_set(&field.selection_set)?;
        self.current_depth -= 1;

        Ok(())
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'p Positioned<engine_parser::types::FragmentSpread>,
    ) -> BindResult<()> {
        let fragment_name = &fragment_spread.fragment_name.node;
        if self.current_fragments_stack.contains(&fragment_name.as_str()) {
            self.current_fragments_stack.push(fragment_name.as_str());
            return Err(BindError::FragmentCycle {
                cycle: std::mem::take(&mut self.current_fragments_stack)
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                location: fragment_spread.pos.try_into()?,
            });
        }
        let Some(fragment) = self.operation.fragments.get(fragment_name) else {
            return Err(BindError::UnknownFragment {
                name: fragment_name.to_string(),
                location: fragment_spread.pos.try_into()?,
            });
        };

        self.current_fragments_stack.push(fragment_name.as_str());
        self.visit_selection_set(&fragment.selection_set)?;
        self.current_fragments_stack.pop();

        Ok(())
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'p Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<()> {
        self.visit_selection_set(&inline_fragment.selection_set)
    }
}
