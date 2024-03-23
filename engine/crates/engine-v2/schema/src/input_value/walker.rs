use std::cmp::Ordering;

use crate::{InputValue, SchemaInputValue, SchemaWalker};

pub type SchemaInputValueWalker<'a> = SchemaWalker<'a, &'a SchemaInputValue>;

impl<'a> From<SchemaInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: SchemaInputValueWalker<'a>) -> Self {
        match walker.item {
            SchemaInputValue::Null => InputValue::Null,
            SchemaInputValue::String(id) => InputValue::String(&walker.schema[*id]),
            SchemaInputValue::EnumValue(id) => InputValue::EnumValue(*id),
            SchemaInputValue::Int(n) => InputValue::Int(*n),
            SchemaInputValue::BigInt(n) => InputValue::BigInt(*n),
            SchemaInputValue::Float(f) => InputValue::Float(*f),
            SchemaInputValue::Boolean(b) => InputValue::Boolean(*b),
            SchemaInputValue::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.schema[*ids] {
                    fields.push((*input_value_definition_id, walker.walk(value).into()));
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            SchemaInputValue::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for value in &walker.schema[*ids] {
                    values.push(walker.walk(value).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            SchemaInputValue::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.schema[*ids] {
                    key_values.push((walker.schema[*key].as_str(), Self::from(walker.walk(value))));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            SchemaInputValue::U64(n) => InputValue::U64(*n),
        }
    }
}

/// Ordering is used to avoid duplciates with a BTreeMap, making RequiredFielSet merging fast and
/// efficient.
impl Ord for SchemaInputValueWalker<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item.discriminant().cmp(&other.item.discriminant()).then_with(|| {
            match (self.item, other.item) {
                (SchemaInputValue::Null, SchemaInputValue::Null) => Ordering::Equal,
                (SchemaInputValue::String(l), SchemaInputValue::String(r)) => l.cmp(r),
                (SchemaInputValue::EnumValue(l), SchemaInputValue::EnumValue(r)) => l.cmp(r),
                (SchemaInputValue::Int(l), SchemaInputValue::Int(r)) => l.cmp(r),
                (SchemaInputValue::BigInt(l), SchemaInputValue::BigInt(r)) => l.cmp(r),
                (SchemaInputValue::U64(l), SchemaInputValue::U64(r)) => l.cmp(r),
                (SchemaInputValue::Float(l), SchemaInputValue::Float(r)) => l.total_cmp(r),
                (SchemaInputValue::Boolean(l), SchemaInputValue::Boolean(r)) => l.cmp(r),
                (SchemaInputValue::InputObject(lids), SchemaInputValue::InputObject(rids)) => {
                    let left = &self.schema[*lids];
                    let right = &self.schema[*rids];
                    left.len().cmp(&right.len()).then_with(|| {
                        for ((lid, left_value), (rid, right_value)) in left.iter().zip(right) {
                            match lid
                                .cmp(rid)
                                .then_with(|| self.walk(left_value).cmp(&self.walk(right_value)))
                            {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        Ordering::Equal
                    })
                }
                (SchemaInputValue::List(lids), SchemaInputValue::List(rids)) => {
                    let left = &self.schema[*lids];
                    let right = &self.schema[*rids];
                    left.len().cmp(&right.len()).then_with(|| {
                        for (lv, rv) in left.iter().zip(right) {
                            match self.walk(lv).cmp(&self.walk(rv)) {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        Ordering::Equal
                    })
                }
                (SchemaInputValue::Map(lids), SchemaInputValue::Map(rids)) => {
                    let left = &self.schema[*lids];
                    let right = &self.schema[*rids];
                    left.len().cmp(&right.len()).then_with(|| {
                        for ((lid, left_value), (rid, right_value)) in left.iter().zip(right) {
                            // StringId are deduplicated so it's safe to compare the ids directly
                            match lid
                                .cmp(rid)
                                .then_with(|| self.walk(left_value).cmp(&self.walk(right_value)))
                            {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        Ordering::Equal
                    })
                }
                _ => unreachable!(),
            }
        })
    }
}

impl PartialEq for SchemaInputValueWalker<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for SchemaInputValueWalker<'_> {}

impl PartialOrd for SchemaInputValueWalker<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for SchemaInputValueWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            SchemaInputValue::Null => write!(f, "Null"),
            SchemaInputValue::String(s) => s.fmt(f),
            SchemaInputValue::EnumValue(id) => f.debug_tuple("EnumValue").field(&self.walk(*id).name()).finish(),
            SchemaInputValue::Int(n) => f.debug_tuple("Int").field(n).finish(),
            SchemaInputValue::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            SchemaInputValue::U64(n) => f.debug_tuple("U64").field(n).finish(),
            SchemaInputValue::Float(n) => f.debug_tuple("Float").field(n).finish(),
            SchemaInputValue::Boolean(b) => b.fmt(f),
            SchemaInputValue::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.schema[*ids] {
                    map.field(self.walk(*input_value_definition_id).name(), &self.walk(value));
                }
                map.finish()
            }
            SchemaInputValue::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.schema[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            SchemaInputValue::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.schema[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
        }
    }
}
