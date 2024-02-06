macro_rules! id_newtypes {
    ($($ty:ident.$field:ident[$name:ident] => $out:ident unless $msg:literal,)*) => {
        $(
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
            pub(crate) struct $name(std::num::NonZeroU16);

            impl std::ops::Index<$name> for $ty {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$field[(index.0.get() - 1) as usize]
                }
            }

            impl std::ops::IndexMut<$name> for $ty {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$field[(index.0.get() - 1) as usize]
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    usize::from(*self).fmt(f)
                }
            }

            impl std::fmt::Debug for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    usize::from(*self).fmt(f)
                }
            }

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    Self(
                        u16::try_from(value)
                            .ok()
                            .and_then(|value| std::num::NonZeroU16::new(value + 1))
                            .expect($msg)
                    )
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as usize
                }
            }

            impl std::ops::Index<crate::utils::IdRange<$name>> for $ty {
                type Output = [$out];

                fn index(&self, range: crate::utils::IdRange<$name>) -> &Self::Output {
                    let crate::utils::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &self.$field[start..end]
                }
            }
        )*
    }
}

// Not necessary anymore when Rust stabilize std::iter::Step
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IdRange<Id: Copy> {
    pub start: Id,
    pub end: Id,
}

impl<Id> IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    pub fn empty() -> Self {
        Self {
            start: Id::from(0),
            end: Id::from(0),
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Id> {
        (usize::from(self.start)..usize::from(self.end)).map(Id::from)
    }

    pub fn get(&self, i: usize) -> Option<Id> {
        let i = i + usize::from(self.start);
        if i < usize::from(self.end) {
            Some(Id::from(i))
        } else {
            None
        }
    }
}

pub(crate) use id_newtypes;
