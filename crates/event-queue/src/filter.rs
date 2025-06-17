use enumflags2::{BitFlags, bitflags};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventFilter {
    All,
    Types(BitFlags<EventFilterType>),
}

#[bitflags]
#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum EventFilterType {
    Operation = 1 << 0,
    SubgraphRequest = 1 << 1,
    HttpRequest = 1 << 2,
    Extension = 1 << 3,
}
