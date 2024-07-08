use engine_id_newtypes::{impl_id_range, make_id};

use crate::{storage::*, PartialCacheRegistry, RecordLookup};

make_id!(MetaTypeId, MetaTypeRecord, types, PartialCacheRegistry);
impl_id_range!(MetaTypeId);

make_id!(ObjectTypeId, ObjectTypeRecord, objects, PartialCacheRegistry);
make_id!(MetaFieldId, MetaFieldRecord, object_fields, PartialCacheRegistry);
impl_id_range!(MetaFieldId);

make_id!(InterfaceTypeId, InterfaceTypeRecord, interfaces, PartialCacheRegistry);

make_id!(OtherTypeId, OtherTypeRecord, others, PartialCacheRegistry);

make_id!(SupertypeId, StringId, supertypes, PartialCacheRegistry);
impl_id_range!(SupertypeId);

make_id!(StringId, str, strings, PartialCacheRegistry);
