use bitflags::bitflags;

bitflags! {
    pub(crate) struct UsedDirectives: u8 {
        const COST = 1;
        const LIST_SIZE = 1 << 1;
        const COMPOSITE_LOOKUP = 1 << 2;
        const COMPOSITE_REQUIRE = 1 << 3;
    }
}
