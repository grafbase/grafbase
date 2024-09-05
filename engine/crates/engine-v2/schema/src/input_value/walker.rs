use std::cmp::Ordering;

use crate::{InputValue, SchemaInputValueRecord, SchemaWalker};

pub type SchemaInputValueWalker<'a> = SchemaWalker<'a, &'a SchemaInputValueRecord>;

impl<'a> From<SchemaInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: SchemaInputValueWalker<'a>) -> Self {
        match walker.item {
            SchemaInputValueRecord::Null => InputValue::Null,
            SchemaInputValueRecord::String(id) => InputValue::String(&walker.schema[*id]),
            SchemaInputValueRecord::EnumValue(id) => InputValue::EnumValue(*id),
            SchemaInputValueRecord::Int(n) => InputValue::Int(*n),
            SchemaInputValueRecord::BigInt(n) => InputValue::BigInt(*n),
            SchemaInputValueRecord::Float(f) => InputValue::Float(*f),
            SchemaInputValueRecord::Boolean(b) => InputValue::Boolean(*b),
            SchemaInputValueRecord::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.schema[*ids] {
                    fields.push((*input_value_definition_id, walker.walk(value).into()));
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            SchemaInputValueRecord::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for value in &walker.schema[*ids] {
                    values.push(walker.walk(value).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            SchemaInputValueRecord::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.schema[*ids] {
                    key_values.push((walker.schema[*key].as_str(), Self::from(walker.walk(value))));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            SchemaInputValueRecord::U64(n) => InputValue::U64(*n),
        }
    }
}

/// Ordering is used to avoid duplciates with a BTreeMap, making RequiredFielSet merging fast and
/// efficient.
impl Ord for SchemaInputValueWalker<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item.discriminant().cmp(&other.item.discriminant()).then_with(|| {
            match (self.item, other.item) {
                (SchemaInputValueRecord::Null, SchemaInputValueRecord::Null) => Ordering::Equal,
                (SchemaInputValueRecord::String(l), SchemaInputValueRecord::String(r)) => l.cmp(r),
                (SchemaInputValueRecord::EnumValue(l), SchemaInputValueRecord::EnumValue(r)) => l.cmp(r),
                (SchemaInputValueRecord::Int(l), SchemaInputValueRecord::Int(r)) => l.cmp(r),
                (SchemaInputValueRecord::BigInt(l), SchemaInputValueRecord::BigInt(r)) => l.cmp(r),
                (SchemaInputValueRecord::U64(l), SchemaInputValueRecord::U64(r)) => l.cmp(r),
                (SchemaInputValueRecord::Float(l), SchemaInputValueRecord::Float(r)) => l.total_cmp(r),
                (SchemaInputValueRecord::Boolean(l), SchemaInputValueRecord::Boolean(r)) => l.cmp(r),
                (SchemaInputValueRecord::InputObject(lids), SchemaInputValueRecord::InputObject(rids)) => {
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
                (SchemaInputValueRecord::List(lids), SchemaInputValueRecord::List(rids)) => {
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
                (SchemaInputValueRecord::Map(lids), SchemaInputValueRecord::Map(rids)) => {
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
            SchemaInputValueRecord::Null => write!(f, "Null"),
            SchemaInputValueRecord::String(s) => s.fmt(f),
            SchemaInputValueRecord::EnumValue(id) => f.debug_tuple("EnumValue").field(&self.walk(*id).name()).finish(),
            SchemaInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            SchemaInputValueRecord::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            SchemaInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            SchemaInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            SchemaInputValueRecord::Boolean(b) => b.fmt(f),
            SchemaInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.schema[*ids] {
                    map.field(self.walk(*input_value_definition_id).name(), &self.walk(value));
                }
                map.finish()
            }
            SchemaInputValueRecord::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.schema[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            SchemaInputValueRecord::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.schema[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
        }
    }
}
