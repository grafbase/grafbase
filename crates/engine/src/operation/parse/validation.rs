use cynic_parser::{
    executable::{FieldSelection, FragmentSpread, InlineFragment, Iter, Selection},
    Span,
};
use schema::Schema;

use super::{ParseError, ParseResult, ParsedOperation};

pub(super) fn validate(schema: &Schema, operation: &ParsedOperation) -> ParseResult<()> {
    let limits = &schema.settings.operation_limits;
    let operation = operation.operation();

    Visitor {
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
    .visit_selection_set(operation.selection_set(), operation.selection_set_span())
}

struct Visitor<'p> {
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
    fn visit_selection_set(&mut self, selection_set: Iter<'p, Selection<'p>>, span: Span) -> ParseResult<()> {
        for item in selection_set {
            match item {
                Selection::Field(field) => {
                    self.root_fields += (self.current_depth == 0) as usize;
                    if self.root_fields > self.max_root_fields {
                        return Err(ParseError::QueryContainsTooManyRootFields {
                            count: self.root_fields,
                            span,
                        });
                    }
                    self.complexity += 1;
                    if self.complexity > self.max_complexity {
                        return Err(ParseError::QueryTooComplex {
                            complexity: self.complexity,
                            span: field.selection_set_span().map(Into::into).unwrap_or(span),
                        });
                    }
                    self.visit_field(field)?;
                }
                Selection::FragmentSpread(fragment_spread) => {
                    self.visit_fragment_spread(fragment_spread)?;
                }
                Selection::InlineFragment(inline_fragment) => {
                    self.visit_inline_fragment(inline_fragment)?;
                }
            }
        }

        Ok(())
    }

    fn visit_field(&mut self, field: FieldSelection<'p>) -> ParseResult<()> {
        self.aliases_count += field.alias().is_some() as usize;
        if self.aliases_count > self.max_aliases_count {
            return Err(ParseError::QueryContainsTooManyAliases {
                count: self.aliases_count,
                span: field.alias_span().unwrap(),
            });
        }
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(ParseError::QueryTooDeep {
                depth: self.current_depth,
                span: field.name_span(),
            });
        }

        self.visit_selection_set(
            field.selection_set(),
            field.selection_set_span().unwrap_or(field.name_span()),
        )?;
        self.current_depth -= 1;

        Ok(())
    }

    fn visit_fragment_spread(&mut self, fragment_spread: FragmentSpread<'p>) -> ParseResult<()> {
        let fragment_name = fragment_spread.fragment_name();

        if self.current_fragments_stack.contains(&fragment_name) {
            self.current_fragments_stack.push(fragment_name);
            return Err(ParseError::FragmentCycle {
                cycle: std::mem::take(&mut self.current_fragments_stack)
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                span: fragment_spread.fragment_name_span(),
            });
        }

        let Some(fragment) = fragment_spread.fragment() else {
            return Err(ParseError::UnknownFragment {
                name: fragment_name.to_string(),
                span: fragment_spread.fragment_name_span(),
            });
        };

        self.current_fragments_stack.push(fragment_name);
        self.visit_selection_set(fragment.selection_set(), fragment.selection_set_span())?;
        self.current_fragments_stack.pop();

        Ok(())
    }

    fn visit_inline_fragment(&mut self, inline_fragment: InlineFragment<'p>) -> ParseResult<()> {
        self.visit_selection_set(inline_fragment.selection_set(), inline_fragment.selection_set_span())
    }
}
