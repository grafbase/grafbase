mod range;

pub use range::*;

#[macro_export]
macro_rules! make_id {
    ($name:ident, $output:ident, $field:ident, $container:ident) => {
        #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name(std::num::NonZeroU32);

        impl $name {
            pub(crate) fn new(index: usize) -> Self {
                Self(
                    std::num::NonZeroU32::new(u32::try_from(index + 1).expect("too many indices"))
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
                Some(Self(std::num::NonZeroU32::new(self.0.get() + 1)?))
            }
            fn back(self) -> Option<Self> {
                Some(Self(std::num::NonZeroU32::new(self.0.get() - 1)?))
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

#[cfg(test)]
mod tests {
    use crate::IdRange;

    #[allow(dead_code)]
    pub trait RecordLookup<T> {
        type Output;

        fn lookup(&self, _index: T) -> &Self::Output {
            unimplemented!()
        }
    }

    struct Thing;

    #[allow(dead_code)]
    struct Container {
        things: Vec<Thing>,
    }

    super::make_id!(ThingId, Thing, things, Container);
    super::impl_id_range!(ThingId);

    #[test]
    fn test_some_things() {
        assert_eq!(ThingId::new(0).to_index(), 0);
        assert_eq!(IdRange::<ThingId>::default().len(), 0);
        assert_eq!(IdRange::<ThingId>::default().next(), None);

        let a_range = IdRange::new(ThingId::new(0), ThingId::new(1));
        assert_eq!(a_range.len(), 1);
        assert_eq!(a_range.collect::<Vec<_>>(), vec![ThingId::new(0)]);

        let a_range = IdRange::new(ThingId::new(5), ThingId::new(7));
        assert_eq!(a_range.len(), 2);
        assert_eq!(a_range.collect::<Vec<_>>(), vec![ThingId::new(5), ThingId::new(6)]);
    }
}
