use config::ComplexityControl;
use grafbase_telemetry::graphql::OperationType;
use schema::{InputObjectDefinition, StringId};
use serde::Deserialize;

use crate::{DataField, Field, FieldArgument, OperationContext, SelectionSet, Variables};

#[derive(Debug, thiserror::Error)]
pub enum ComplexityError {
    #[error("Query exceeded complexity limit")]
    LimitReached,
    #[error("Expected exactly one slicing argument on {0}")]
    ExpectedOneSlicingArgument(String),
}

pub fn compute_and_validate_complexity(
    ctx: OperationContext<'_>,
    variables: &Variables,
) -> Result<Option<ComplexityCost>, ComplexityError> {
    match ctx.schema.settings.complexity_control {
        ComplexityControl::Disabled => Ok(None),
        ComplexityControl::Enforce { limit, .. } => {
            let complexity = calculate_complexity(ctx, variables)?;
            if complexity.0 > limit {
                return Err(ComplexityError::LimitReached);
            }
            Ok(Some(complexity))
        }
        ComplexityControl::Measure { .. } => calculate_complexity(ctx, variables).map(Some),
    }
}

pub fn calculate_complexity(
    ctx: OperationContext<'_>,
    variables: &Variables,
) -> Result<ComplexityCost, ComplexityError> {
    let base_cost = match ctx.operation.attributes.ty {
        OperationType::Query | OperationType::Subscription => 0,
        OperationType::Mutation => 10,
    };

    let selection_set = ctx.root_selection_set();

    let context = ComplexityContext {
        default_list_size: ctx
            .schema
            .settings
            .complexity_control
            .list_size()
            .expect("should be some unless disabled"),
        variables,
    };

    let cost = base_cost + selection_set_complexity(&context, selection_set, None)?;

    tracing::debug!("Complexity was {cost}");

    Ok(ComplexityCost(cost))
}

#[derive(Clone, Copy)]
pub struct ComplexityCost(pub usize);

struct ComplexityContext<'a> {
    default_list_size: usize,
    variables: &'a Variables,
}

fn selection_set_complexity(
    context: &ComplexityContext<'_>,
    selection_set: SelectionSet<'_>,
    field_multipliers: Option<FieldMultipliers>,
) -> Result<usize, ComplexityError> {
    Ok(selection_set
        .fields()
        .map(|field| match field {
            Field::Data(field) => {
                let preset_list_size = field_multipliers.as_ref().and_then(|child_fields| {
                    child_fields.fields.iter().find_map(|child_field| {
                        if field.definition().name_id == *child_field {
                            Some(child_fields.multiplier)
                        } else {
                            None
                        }
                    })
                });
                field_complexity(context, field, preset_list_size)
            }
            Field::Typename(_) => Ok(1),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum())
}

fn field_complexity(
    context: &ComplexityContext<'_>,
    field: DataField<'_>,
    preset_list_size: Option<usize>,
) -> Result<usize, ComplexityError> {
    let type_cost = field
        .definition()
        .cost()
        .map(|cost| cost.weight)
        .unwrap_or_else(|| cost_for_type(field.definition().ty().definition())) as usize;

    let list_size_directive = field.definition().list_size();

    let child_count = calculate_child_count(context, field, list_size_directive, preset_list_size)?;

    let argument_cost = field
        .arguments()
        .map(|argument| cost_for_argument(argument, context.variables))
        .sum::<usize>();

    let this_field_count = child_count.this_field_count();
    let child_field_count = child_count.child_field_count();

    let child_cost = selection_set_complexity(context, field.selection_set(), child_field_count)?;

    Ok(this_field_count * (type_cost + argument_cost + child_cost))
}

fn cost_for_argument(argument: FieldArgument<'_>, variables: &Variables) -> usize {
    let def = argument.definition();
    let argument_type = def.ty().definition();
    let argument_cost = def.cost().unwrap_or_else(|| cost_for_type(argument_type)) as usize;

    let Ok(value) = serde_json::Value::deserialize(argument.value(variables)) else {
        tracing::warn!("Could not deserialize value when calculating cost");
        return argument_cost;
    };

    match argument_type {
        schema::Definition::InputObject(obj) => cost_for_object_value(&value, obj, argument_cost),
        _ => cost_for_input_scalar(&value, argument_cost),
    }
}

fn cost_for_object_value(
    value: &serde_json::Value,
    object: InputObjectDefinition<'_>,
    base_object_cost: usize,
) -> usize {
    match value {
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| cost_for_object_value(value, object, base_object_cost))
            .sum(),
        serde_json::Value::Object(map) => {
            let mut overall_cost = base_object_cost;
            for (name, value) in map {
                let Some(field) = object.input_fields().find(|field| field.name() == name) else {
                    continue;
                };
                let field_cost = field.cost().unwrap_or(cost_for_type(field.ty().definition())) as usize;

                if let schema::Definition::InputObject(object) = field.ty().definition() {
                    overall_cost += cost_for_object_value(value, object, field_cost);
                } else {
                    overall_cost += cost_for_input_scalar(value, field_cost);
                }
            }
            overall_cost
        }
        _ => 0,
    }
}

fn cost_for_input_scalar(value: &serde_json::Value, item_cost: usize) -> usize {
    match value {
        serde_json::Value::Array(values) => values.iter().map(|value| cost_for_input_scalar(value, item_cost)).sum(),
        _ => item_cost,
    }
}

#[derive(Debug)]
enum ListSizeHandling {
    ThisFieldIsTheList(usize),
    ChildFieldsAreTheList(FieldMultipliers),
}

impl ListSizeHandling {
    pub fn this_field_count(&self) -> usize {
        match self {
            ListSizeHandling::ThisFieldIsTheList(count) => *count,
            ListSizeHandling::ChildFieldsAreTheList(_) => 1,
        }
    }

    pub fn child_field_count(self) -> Option<FieldMultipliers> {
        match self {
            ListSizeHandling::ChildFieldsAreTheList(count) => Some(count),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct FieldMultipliers {
    multiplier: usize,
    fields: Vec<StringId>,
}

fn calculate_child_count<'a>(
    context: &ComplexityContext<'_>,
    field: DataField<'a>,
    list_size_directive: Option<schema::ListSizeDirective<'a>>,
    preset_list_size: Option<usize>,
) -> Result<ListSizeHandling, ComplexityError> {
    let field_is_list = field.definition().ty().wrapping.is_list();

    if let Some(size) = preset_list_size {
        return Ok(ListSizeHandling::ThisFieldIsTheList(size));
    }

    let default_multiplier = if field_is_list { context.default_list_size } else { 1 };

    let Some(directive) = list_size_directive else {
        return Ok(ListSizeHandling::ThisFieldIsTheList(default_multiplier));
    };

    let mut multiplier = directive.assumed_size.unwrap_or(context.default_list_size as u32) as usize;

    let mut slicing_arguments = directive.slicing_arguments().peekable();
    if slicing_arguments.peek().is_some() {
        let slicing_arguments = slicing_arguments
            .filter_map(|def| {
                field
                    .arguments()
                    .find(|arg| arg.definition().id == def.id)?
                    .value(context.variables)
                    .as_usize()
            })
            .collect::<Vec<_>>();

        if directive.require_one_slicing_argument && slicing_arguments.len() != 1 {
            let container_name = field.definition().parent_entity().name();
            let field_name = field.definition().name();

            return Err(ComplexityError::ExpectedOneSlicingArgument(format!(
                "{}.{}",
                container_name, field_name
            )));
        }

        multiplier = slicing_arguments.into_iter().max().unwrap_or(context.default_list_size);
    }

    let mut sized_fields = directive.sized_fields().peekable();
    if sized_fields.peek().is_none() {
        return Ok(ListSizeHandling::ThisFieldIsTheList(multiplier));
    }

    Ok(ListSizeHandling::ChildFieldsAreTheList(FieldMultipliers {
        multiplier,
        fields: sized_fields.map(|sized_field| sized_field.name_id).collect(),
    }))
}

fn cost_for_type(definition: schema::Definition<'_>) -> i32 {
    if let Some(cost) = definition.cost() {
        return cost;
    }

    match definition {
        schema::Definition::Enum(_) | schema::Definition::Scalar(_) => 0,
        schema::Definition::Interface(_) | schema::Definition::Object(_) | schema::Definition::Union(_) => 1,
        schema::Definition::InputObject(_) => 1,
    }
}
