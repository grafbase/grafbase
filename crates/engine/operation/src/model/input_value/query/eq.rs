use schema::SchemaInputValueRecord;
use walker::Walk;

use crate::OperationContext;

use super::QueryInputValueRecord;

pub fn is_query_value_equivalent_schema_value(
    ctx: OperationContext<'_>,
    left: &QueryInputValueRecord,
    right: &SchemaInputValueRecord,
) -> bool {
    match (left, right) {
        (QueryInputValueRecord::Null, SchemaInputValueRecord::Null) => true,
        (QueryInputValueRecord::Null, _) => false,
        (QueryInputValueRecord::String(l), SchemaInputValueRecord::String(r)) => l == &ctx.schema[*r],
        (QueryInputValueRecord::String(_), _) => false,
        (QueryInputValueRecord::EnumValue(l), SchemaInputValueRecord::EnumValue(r)) => l == r,
        (QueryInputValueRecord::EnumValue(_), _) => false,
        (QueryInputValueRecord::UnboundEnumValue(l), SchemaInputValueRecord::UnboundEnumValue(r)) => {
            l == &ctx.schema[*r]
        }
        (QueryInputValueRecord::UnboundEnumValue(_), _) => false,
        (QueryInputValueRecord::Int(l), SchemaInputValueRecord::Int(r)) => l == r,
        (QueryInputValueRecord::Int(_), _) => false,
        (QueryInputValueRecord::BigInt(l), SchemaInputValueRecord::BigInt(r)) => l == r,
        (QueryInputValueRecord::BigInt(_), _) => false,
        (QueryInputValueRecord::U64(l), SchemaInputValueRecord::U64(r)) => l == r,
        (QueryInputValueRecord::U64(_), _) => false,
        (QueryInputValueRecord::Float(l), SchemaInputValueRecord::Float(r)) => l == r,
        (QueryInputValueRecord::Float(_), _) => false,
        (QueryInputValueRecord::Boolean(l), SchemaInputValueRecord::Boolean(r)) => l == r,
        (QueryInputValueRecord::Boolean(_), _) => false,
        (QueryInputValueRecord::InputObject(lids), SchemaInputValueRecord::InputObject(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let mut left = ctx.operation.query_input_values[*lids].iter();
            let mut right = ctx.schema[*rids].iter();
            while let Some(((left_id, left_value), (right_id, right_value))) = left.next().zip(right.next()) {
                if left_id != right_id || !is_query_value_equivalent_schema_value(ctx, left_value, right_value) {
                    return false;
                }
            }

            true
        }
        (QueryInputValueRecord::InputObject(_), _) => false,
        (QueryInputValueRecord::List(lids), SchemaInputValueRecord::List(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let left = &ctx.operation.query_input_values[*lids];
            let right = &ctx.schema[*rids];
            for (left_value, right_value) in left.iter().zip(right) {
                if !is_query_value_equivalent_schema_value(ctx, left_value, right_value) {
                    return false;
                }
            }
            true
        }
        (QueryInputValueRecord::List(_), _) => false,
        (QueryInputValueRecord::Map(lids), SchemaInputValueRecord::Map(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let right_kv = &ctx.schema[*rids];
            for (left_key, left_value) in &ctx.operation.query_input_values[*lids] {
                if let Some((_, right_value)) = right_kv
                    .iter()
                    .find(|(right_key_id, _)| &ctx.schema[*right_key_id] == left_key)
                {
                    if !is_query_value_equivalent_schema_value(ctx, left_value, right_value) {
                        return false;
                    }
                } else {
                    return false;
                };
            }

            true
        }
        (QueryInputValueRecord::Map(_), _) => false,
        (QueryInputValueRecord::DefaultValue(id), value) => id.walk(ctx.schema).eq(&value.walk(ctx.schema)),
        (QueryInputValueRecord::Variable(_), _) => false,
    }
}

pub fn are_query_value_equivalent(
    ctx: OperationContext<'_>,
    left: &QueryInputValueRecord,
    right: &QueryInputValueRecord,
) -> bool {
    match (left, right) {
        (QueryInputValueRecord::Null, QueryInputValueRecord::Null) => true,
        (QueryInputValueRecord::Null, _) => false,
        (QueryInputValueRecord::String(l), QueryInputValueRecord::String(r)) => l == r,
        (QueryInputValueRecord::String(_), _) => false,
        (QueryInputValueRecord::EnumValue(l), QueryInputValueRecord::EnumValue(r)) => l == r,
        (QueryInputValueRecord::EnumValue(_), _) => false,
        (QueryInputValueRecord::UnboundEnumValue(l), QueryInputValueRecord::UnboundEnumValue(r)) => l == r,
        (QueryInputValueRecord::UnboundEnumValue(_), _) => false,
        (QueryInputValueRecord::Int(l), QueryInputValueRecord::Int(r)) => l == r,
        (QueryInputValueRecord::Int(_), _) => false,
        (QueryInputValueRecord::BigInt(l), QueryInputValueRecord::BigInt(r)) => l == r,
        (QueryInputValueRecord::BigInt(_), _) => false,
        (QueryInputValueRecord::U64(l), QueryInputValueRecord::U64(r)) => l == r,
        (QueryInputValueRecord::U64(_), _) => false,
        (QueryInputValueRecord::Float(l), QueryInputValueRecord::Float(r)) => l == r,
        (QueryInputValueRecord::Float(_), _) => false,
        (QueryInputValueRecord::Boolean(l), QueryInputValueRecord::Boolean(r)) => l == r,
        (QueryInputValueRecord::Boolean(_), _) => false,
        (QueryInputValueRecord::InputObject(lids), QueryInputValueRecord::InputObject(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let input_values = &ctx.operation.query_input_values;
            let mut left = input_values[*lids].iter();
            let mut right = input_values[*rids].iter();
            while let Some(((left_id, left_value), (right_id, right_value))) = left.next().zip(right.next()) {
                if left_id != right_id || !are_query_value_equivalent(ctx, left_value, right_value) {
                    return false;
                }
            }

            true
        }
        (QueryInputValueRecord::InputObject(_), _) => false,
        (QueryInputValueRecord::List(lids), QueryInputValueRecord::List(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let input_values = &ctx.operation.query_input_values;
            let left = &input_values[*lids];
            let right = &input_values[*rids];
            for (left_value, right_value) in left.iter().zip(right) {
                if !are_query_value_equivalent(ctx, left_value, right_value) {
                    return false;
                }
            }
            true
        }
        (QueryInputValueRecord::List(_), _) => false,
        (QueryInputValueRecord::Map(lids), QueryInputValueRecord::Map(rids)) => {
            if lids.len() != rids.len() {
                return false;
            }

            let input_values = &ctx.operation.query_input_values;
            let right_kv = &input_values[*rids];
            for (left_key, left_value) in &input_values[*lids] {
                if let Some((_, right_value)) = right_kv.iter().find(|(right_key, _)| right_key == left_key) {
                    if !are_query_value_equivalent(ctx, left_value, right_value) {
                        return false;
                    }
                } else {
                    return false;
                };
            }

            true
        }
        (QueryInputValueRecord::Map(_), _) => false,
        (QueryInputValueRecord::DefaultValue(left_id), QueryInputValueRecord::DefaultValue(right_id)) => {
            left_id.walk(ctx.schema).eq(&right_id.walk(ctx.schema))
        }
        (QueryInputValueRecord::DefaultValue(_), _) => false,
        (QueryInputValueRecord::Variable(left_id), QueryInputValueRecord::Variable(right_id)) => left_id == right_id,
        (QueryInputValueRecord::Variable(_), _) => false,
    }
}
