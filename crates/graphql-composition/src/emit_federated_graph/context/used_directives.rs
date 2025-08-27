use bitflags::bitflags;

bitflags! {
    pub(crate) struct UsedDirectives: u8 {
        const COST = 1;
        const LIST_SIZE = 1 << 1;
        const COMPOSITE_LOOKUP = 1 << 2;
        const COMPOSITE_REQUIRE = 1 << 3;
        const COMPOSITE_IS = 1 << 4;
        const COMPOSITE_DERIVE = 1 << 5;
        const COMPOSITE_INTERNAL = 1 << 6;
        const JOIN_ENUM_VALUE = 1 << 7;
    }
}
