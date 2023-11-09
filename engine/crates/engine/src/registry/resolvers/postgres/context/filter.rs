mod complex;
mod simple;

pub(super) use complex::ComplexFilterIterator;
use grafbase_sql_ast::ast::ConditionTree;
pub(super) use simple::ByFilterIterator;

#[derive(Clone)]
pub enum FilterIterator<'a> {
    By(ByFilterIterator<'a>),
    Complex(ComplexFilterIterator<'a>),
}

impl<'a> Iterator for FilterIterator<'a> {
    type Item = ConditionTree<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FilterIterator::By(iterator) => iterator.next().map(ConditionTree::from),
            FilterIterator::Complex(iterator) => iterator.next(),
        }
    }
}
