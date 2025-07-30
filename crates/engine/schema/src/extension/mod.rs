mod argument;
mod input_value;
mod ty;

pub use argument::*;
pub use input_value::*;
pub use ty::*;

use crate::ExtensionDirective;

impl<'a> ExtensionDirective<'a> {
    pub fn arguments_with_stage<'f, 'i>(
        &self,
        predicate: impl Fn(InjectionStage) -> bool + 'f,
    ) -> impl Iterator<Item = (&'a str, &'a ExtensionInputValueRecord)> + 'i
    where
        'a: 'i,
        'f: 'i,
    {
        self.argument_records()
            .iter()
            .filter(move |arg| predicate(arg.injection_stage))
            .map(|arg| (self.schema[arg.name_id].as_str(), &arg.value))
    }

    pub fn max_arguments_stage(&self) -> InjectionStage {
        self.argument_records()
            .iter()
            .map(|arg| arg.injection_stage)
            .max()
            .unwrap_or_default()
    }

    pub fn argument_records(&self) -> &'a [ExtensionDirectiveArgumentRecord] {
        &self.schema[self.as_ref().argument_ids]
    }

    pub fn static_arguments(self) -> ExtensionDirectiveArgumentsStaticView<'a> {
        ExtensionDirectiveArgumentsStaticView { directive: self }
    }
}
