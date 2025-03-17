mod frozen_sorted_vec;
mod ids;
mod records;
mod view;

pub(crate) use self::{ids::*, records::*, view::*};

use self::frozen_sorted_vec::FrozenSortedVec;

#[derive(Debug, Default)]
pub(crate) struct TranslatedSchema {
    pub(crate) packages: FrozenSortedVec<ProtoPackage>,
    pub(crate) messages: FrozenSortedVec<ProtoMessage>,
    pub(crate) fields: FrozenSortedVec<ProtoField>,
    pub(crate) services: FrozenSortedVec<ProtoService>,
    pub(crate) methods: FrozenSortedVec<ProtoMethod>,
    pub(crate) enums: FrozenSortedVec<ProtoEnum>,
}
