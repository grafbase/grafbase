//! A structured representation of the types of a GraphQL schema generated from protobuf definitions.:

mod frozen_sorted_vec;
mod ids;
mod records;
mod view;

pub(crate) use self::{ids::*, records::*, view::*};

use self::frozen_sorted_vec::FrozenSortedVec;

#[derive(Debug, Default)]
pub(crate) struct GrpcSchema {
    pub(crate) packages: FrozenSortedVec<ProtoPackage>,
    // Not sorted!
    pub(crate) messages: Vec<ProtoMessage>,
    pub(crate) fields: FrozenSortedVec<ProtoField>,
    pub(crate) services: FrozenSortedVec<ProtoService>,
    pub(crate) methods: FrozenSortedVec<ProtoMethod>,
    // Not sorted!
    pub(crate) enums: Vec<ProtoEnum>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord)]
pub(crate) enum Parent {
    Root,
    Message(ProtoMessageId),
    Package(ProtoPackageId),
}

impl Parent {
    pub(crate) fn child_name(&self, schema: &GrpcSchema, name: &str) -> String {
        match self {
            Parent::Root => format!(".{name}"),
            Parent::Message(message_id) => format!("{}.{name}", schema[*message_id].name),
            Parent::Package(package_id) => format!(".{}.{name}", schema[*package_id].name),
        }
    }
}
