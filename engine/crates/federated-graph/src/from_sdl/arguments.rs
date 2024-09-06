use super::ast;

pub(super) trait GetArgumentsExt<'a> {
    fn get_argument(&self, argument_name: &str) -> Option<ast::Argument<'a>>;
}

impl<'a> GetArgumentsExt<'a> for ast::Directive<'a> {
    fn get_argument(&self, argument_name: &str) -> Option<ast::Argument<'a>> {
        self.arguments().find(|arg| arg.name() == argument_name)
    }
}
