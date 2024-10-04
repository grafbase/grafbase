use super::{EnumValueId, FederatedGraph};
use std::fmt;

struct DebugFn<F>(F)
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<F> fmt::Debug for DebugFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

impl fmt::Debug for FederatedGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field(
                "type_definitions",
                &DebugFn(|f| f.debug_list().entries(self.iter_type_definitions()).finish()),
            )
            .field(
                "enum_values",
                &DebugFn(|f| {
                    f.debug_list()
                        .entries((0..self.enum_values.len()).map(|idx| self.at(EnumValueId::from(idx))))
                        .finish()
                }),
            )
            .finish_non_exhaustive()
    }
}
