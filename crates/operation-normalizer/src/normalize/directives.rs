use graphql_parser::schema::Directive;

use super::arguments;

pub(super) fn normalize<'a>(directives: &mut [Directive<'a, &'a str>]) {
    directives.sort_by(|a, b| a.name.cmp(b.name));

    for directive in directives.iter_mut() {
        arguments::normalize(&mut directive.arguments);
    }
}
