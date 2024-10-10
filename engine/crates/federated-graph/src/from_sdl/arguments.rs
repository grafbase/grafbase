use super::ast;

pub(super) trait GetArgumentsExt<'a> {
    fn get_argument(&self, argument_name: &str) -> Option<cynic_parser::values::ConstValue<'a>>;
}

impl<'a> GetArgumentsExt<'a> for ast::Directive<'a> {
    fn get_argument(&self, argument_name: &str) -> Option<cynic_parser::values::ConstValue<'a>> {
        self.arguments()
            .find(|arg| arg.name() == argument_name)
            .map(|arg| arg.value())
    }
}
