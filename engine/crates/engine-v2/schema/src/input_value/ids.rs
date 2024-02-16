use crate::{InputValue, InputValueDefinitionId, InputValues, MAX_ID};

macro_rules! input_ids {
    ($($field:ident[$name:ident] => $out:ty | unless $msg:literal,)*) => {
        $(
            #[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $name<Str> {
                index: std::num::NonZeroU32,
                _phantom: std::marker::PhantomData<Str>,
            }

            impl<Str> Clone for $name<Str> {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<Str> Copy for $name<Str> {}

            impl<Str> std::ops::Index<$name<Str>> for InputValues<Str> {
                type Output = $out;

                fn index(&self, id: $name<Str>) -> &$out {
                    &self.$field[(id.index.get() - 1) as usize]
                }
            }

            impl<Str> std::ops::IndexMut<$name<Str>> for InputValues<Str> {
                fn index_mut(&mut self, id: $name<Str>) -> &mut $out {
                    &mut self.$field[(id.index.get() - 1) as usize]
                }
            }

            impl<Str> std::ops::Index<crate::ids::IdRange<$name<Str>>> for InputValues<Str> {
                type Output = [$out];

                fn index(&self, range: crate::ids::IdRange<$name<Str>>) -> &Self::Output {
                    let crate::ids::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &self.$field[start..end]
                }
            }

            impl<Str> std::ops::IndexMut<crate::ids::IdRange<$name<Str>>> for InputValues<Str> {
                fn index_mut(&mut self, range: crate::ids::IdRange<$name<Str>>) -> &mut Self::Output {
                    let crate::ids::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &mut self.$field[start..end]
                }
            }

            impl<Str> From<usize> for $name<Str> {
                fn from(index: usize) -> Self {
                    assert!(index <= MAX_ID, "{}", $msg);
                    Self {
                        index: std::num::NonZeroU32::new((index + 1) as u32).unwrap(),
                        _phantom: std::marker::PhantomData,
                    }
                }
            }

            impl<Str> From<$name<Str>> for usize {
                fn from(id: $name<Str>) -> Self {
                    (id.index.get() - 1) as usize
                }
            }

            impl<Str> From<$name<Str>> for u32 {
                fn from(id: $name<Str>) -> Self {
                    id.index.get() - 1
                }
            }
        )*
    };
}

input_ids!(
    values[InputValueId] => InputValue<Str> | unless "Too many input values",
    input_fields[InputObjectFieldValueId] => (InputValueDefinitionId, InputValue<Str>) | unless "Too many input object fields",
    key_values[InputKeyValueId] => (Str, InputValue<Str>) | unless "Too many input fields",
);
