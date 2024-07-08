use cynic_parser::executable::{iter::Iter, Directive, FieldSelection, FragmentSpread, InlineFragment, Value};

pub trait FieldExt<'a> {
    fn response_key(&self) -> &'a str;
}

impl<'a> FieldExt<'a> for FieldSelection<'a> {
    fn response_key(&self) -> &'a str {
        self.alias().unwrap_or(self.name())
    }
}

pub struct DeferDirective<'a> {
    pub label: Option<&'a str>,
}

pub trait DeferExt<'a> {
    fn defer_directive(&self) -> Option<DeferDirective<'a>>;
}

impl<'a> DeferExt<'a> for FragmentSpread<'a> {
    fn defer_directive(&self) -> Option<DeferDirective<'a>> {
        find_defer(self.directives())
    }
}

impl<'a> DeferExt<'a> for InlineFragment<'a> {
    fn defer_directive(&self) -> Option<DeferDirective<'a>> {
        find_defer(self.directives())
    }
}

fn find_defer<'a>(mut directives: Iter<'a, Directive<'a>>) -> Option<DeferDirective<'a>> {
    directives
        .find(|directive| directive.name() == "defer")
        .map(|directive| {
            let label = directive
                .arguments()
                .find(|arg| arg.name() == "label")
                .and_then(|argument| {
                    let value = argument.value();
                    let Value::String(label) = value else { return None };
                    Some(label)
                });

            DeferDirective { label }
        })
}
