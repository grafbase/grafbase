mod range;

pub use range::*;

#[macro_export]
macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident, $container:ident) => {
        #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            pub(crate) fn new(index: usize) -> Self {
                Self(
                    NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
                        .expect("also too many indices"),
                )
            }

            pub(crate) fn to_index(self) -> usize {
                (self.0.get() - 1) as usize
            }
        }

        impl RecordLookup<$name> for $container {
            type Output = $output;

            fn lookup(&self, index: $name) -> &Self::Output {
                &self.$field[index.to_index()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_id_range {
    ($name: ident) => {
        impl $crate::IdOperations for $name {
            fn empty_range() -> $crate::IdRange<Self> {
                $crate::IdRange::new($name::new(0), $name::new(0))
            }
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
