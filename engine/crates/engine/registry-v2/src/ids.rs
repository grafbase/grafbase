use std::num::NonZeroU32;

use engine_id_newtypes::{impl_id_range, make_id};

use crate::{
    generated::{
        enums::{EnumTypeRecord, MetaEnumValueRecord},
        inputs::{InputObjectTypeRecord, InputValidatorRecord},
    },
    storage::*,
    RecordLookup, Registry,
};

make_id!(MetaTypeId, MetaTypeRecord, types, Registry);
impl_id_range!(MetaTypeId);

make_id!(ObjectTypeId, ObjectTypeRecord, objects, Registry);
make_id!(MetaFieldId, MetaFieldRecord, object_fields, Registry);
impl_id_range!(MetaFieldId);

make_id!(MetaInputValueId, MetaInputValueRecord, input_values, Registry);
impl_id_range!(MetaInputValueId);

make_id!(InputValidatorId, InputValidatorRecord, input_validators, Registry);
impl_id_range!(InputValidatorId);

make_id!(InputObjectTypeId, InputObjectTypeRecord, input_objects, Registry);

make_id!(EnumTypeId, EnumTypeRecord, enums, Registry);
make_id!(MetaEnumValueId, MetaEnumValueRecord, enum_values, Registry);
impl_id_range!(MetaEnumValueId);

make_id!(InterfaceTypeId, InterfaceTypeRecord, interfaces, Registry);

make_id!(ScalarTypeId, ScalarTypeRecord, scalars, Registry);

make_id!(UnionTypeId, UnionTypeRecord, unions, Registry);

make_id!(MetaDirectiveId, MetaDirectiveRecord, directives, Registry);
impl_id_range!(MetaDirectiveId);

make_id!(StringId, str, strings, Registry);
