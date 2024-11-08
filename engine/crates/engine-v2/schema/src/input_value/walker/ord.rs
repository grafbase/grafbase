use std::cmp::Ordering;

use walker::Walk;

use crate::SchemaInputValueRecord;

use super::SchemaInputValue;

/// Ordering is used to avoid duplciates with a BTreeMap, making RequiredFielSet merging fast and
/// efficient.
impl Ord for SchemaInputValue<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ref_
            .discriminant()
            .cmp(&other.ref_.discriminant())
            .then_with(|| match (self.ref_, other.ref_) {
                (SchemaInputValueRecord::Null, SchemaInputValueRecord::Null) => Ordering::Equal,
                (SchemaInputValueRecord::String(l), SchemaInputValueRecord::String(r)) => l.cmp(r),
                (SchemaInputValueRecord::EnumValue(l), SchemaInputValueRecord::EnumValue(r)) => l.cmp(r),
                (SchemaInputValueRecord::Int(l), SchemaInputValueRecord::Int(r)) => l.cmp(r),
                (SchemaInputValueRecord::BigInt(l), SchemaInputValueRecord::BigInt(r)) => l.cmp(r),
                (SchemaInputValueRecord::U64(l), SchemaInputValueRecord::U64(r)) => l.cmp(r),
                (SchemaInputValueRecord::Float(l), SchemaInputValueRecord::Float(r)) => l.total_cmp(r),
                (SchemaInputValueRecord::Boolean(l), SchemaInputValueRecord::Boolean(r)) => l.cmp(r),
                (SchemaInputValueRecord::InputObject(lids), SchemaInputValueRecord::InputObject(rids)) => {
                    let left = lids.walk(self.schema);
                    let right = rids.walk(self.schema);
                    left.len().cmp(&right.len()).then_with(|| {
                        for ((left_def, left_value), (right_def, right_value)) in left.zip(right) {
                            match left_def
                                .id()
                                .cmp(&right_def.id())
                                .then_with(|| left_value.cmp(&right_value))
                            {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        Ordering::Equal
                    })
                }
                (SchemaInputValueRecord::List(lids), SchemaInputValueRecord::List(rids)) => {
                    lids.walk(self.schema).cmp(rids.walk(self.schema))
                }
                (SchemaInputValueRecord::Map(lids), SchemaInputValueRecord::Map(rids)) => {
                    lids.walk(self.schema).cmp(rids.walk(self.schema))
                }
                _ => unreachable!(),
            })
    }
}

impl PartialEq for SchemaInputValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for SchemaInputValue<'_> {}

impl PartialOrd for SchemaInputValue<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
