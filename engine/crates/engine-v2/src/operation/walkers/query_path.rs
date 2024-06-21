use itertools::Itertools;

use crate::operation::QueryPath;

use super::OperationWalker;

pub type QueryPathWalker<'a> = OperationWalker<'a, &'a QueryPath, ()>;

impl<'a> QueryPathWalker<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &'a str> + 'a {
        let keys = &self.operation.response_keys;
        self.item.into_iter().map(|key| &keys[*key])
    }
}

impl<'a> std::fmt::Display for QueryPathWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys = &self.operation.response_keys;
        write!(
            f,
            "{}",
            self.item
                .into_iter()
                .format_with(".", |key, f| f(&format_args!("{}", &keys[*key])))
        )
    }
}
