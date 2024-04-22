use std::num::NonZeroU32;

use crate::{
    common::IdRange,
    generated::{
        enums::{EnumTypeRecord, MetaEnumValueRecord},
        inputs::{InputObjectTypeRecord, InputValidatorRecord},
    },
    storage::*,
    RecordLookup, Registry,
};

macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident) => {
        #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            pub(super) fn new(index: usize) -> Self {
                Self(
                    NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                        .expect("also too many indices"),
                )
            }

            pub(super) fn to_index(self) -> usize {
                (self.0.get() - 1) as usize
            }
        }

        impl RecordLookup<$name> for Registry {
            type Output = $output;

            fn lookup(&self, index: $name) -> &Self::Output {
                &self.$field[index.to_index()]
            }

            // fn lookup_mut(&mut self, index: $name) -> &mut Self::Output {
            //     &mut self.$field[(index.0.get() - 1) as usize]
            // }
        }
    };
}

macro_rules! impl_id_range {
    ($name: ident) => {
        impl IdRange<$name> {
            pub fn len(&self) -> usize {
                (self.end.0.get() - self.start.0.get()) as usize
            }

            pub fn is_empty(&self) -> bool {
                (self.end.0.get() - self.start.0.get()) == 0
            }

            pub fn iter(&self) -> impl ExactSizeIterator<Item = $name> {
                (self.start.0.get()..self.end.0.get())
                    .map(|num| $name(NonZeroU32::new(num).expect("range is too large")))
            }
        }

        impl Default for IdRange<$name> {
            fn default() -> Self {
                Self::new($name::new(0), $name::new(0))
            }
        }

        impl crate::common::IdOperations for $name {
            fn forward(self) -> Option<Self> {
                Some(Self(NonZeroU32::new(self.0.get() + 1)?))
            }
            fn back(self) -> Option<Self> {
                Some(Self(NonZeroU32::new(self.0.get() - 1)?))
            }
            fn cmp(self, other: Self) -> std::cmp::Ordering {
                self.0.get().cmp(&other.0.get())
            }
            fn distance(lhs: Self, rhs: Self) -> usize {
                rhs.0.get().saturating_sub(lhs.0.get()) as usize
            }
        }
    };
}

make_id!(MetaTypeId, MetaTypeRecord, types);
impl_id_range!(MetaTypeId);

make_id!(ObjectTypeId, ObjectTypeRecord, objects);
make_id!(MetaFieldId, MetaFieldRecord, object_fields);
impl_id_range!(MetaFieldId);

make_id!(MetaInputValueId, MetaInputValueRecord, input_values);
impl_id_range!(MetaInputValueId);

make_id!(InputValidatorId, InputValidatorRecord, input_validators);
impl_id_range!(InputValidatorId);

make_id!(InputObjectTypeId, InputObjectTypeRecord, input_objects);

make_id!(EnumTypeId, EnumTypeRecord, enums);
make_id!(MetaEnumValueId, MetaEnumValueRecord, enum_values);
impl_id_range!(MetaEnumValueId);

make_id!(InterfaceTypeId, InterfaceTypeRecord, interfaces);

make_id!(ScalarTypeId, ScalarTypeRecord, scalars);

make_id!(UnionTypeId, UnionTypeRecord, unions);

make_id!(MetaDirectiveId, MetaDirectiveRecord, directives);
impl_id_range!(MetaDirectiveId);

make_id!(StringId, str, strings);
