use engine_parser::types::Field;

use crate::{
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    Positioned,
};

pub struct AliasCountCalculate<'a> {
    alias_count: &'a mut usize,
}

impl<'a> AliasCountCalculate<'a> {
    pub fn new(alias_count: &'a mut usize) -> Self {
        Self { alias_count }
    }
}

impl<'ctx, 'a> Visitor<'ctx> for AliasCountCalculate<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx>, field: &'ctx Positioned<Field>) {
        if field.node.alias.is_some() {
            *self.alias_count += 1;
        }
    }
}

#[cfg(fixme)]
#[cfg(test)]
mod tests {
    #![allow(clippy::diverging_sub_expression)]

    use super::*;
    use crate::{
        parser::parse_query,
        validation::{visit, VisitorContext},
        EmptyMutation, EmptySubscription, Object, Schema,
    };

    struct Query;

    struct MyObj;

    #[Object(internal)]
    #[allow(unreachable_code)]
    impl MyObj {
        async fn a(&self) -> i32 {
            todo!()
        }

        async fn b(&self) -> i32 {
            todo!()
        }

        async fn c(&self) -> MyObj {
            todo!()
        }
    }

    #[Object(internal)]
    #[allow(unreachable_code)]
    impl Query {
        async fn value1(&self) -> i32 {
            todo!()
        }

        async fn value2(&self) -> i32 {
            todo!()
        }

        async fn obj(&self) -> MyObj {
            todo!()
        }
    }

    fn check_alias_count(query: &str, expect_alias_count: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut alias_count = 0;
        let mut alias_count_calculate = AliasCountCalculate::new(&mut alias_count);
        visit(&mut alias_count_calculate, &mut ctx, &doc);
        assert_eq!(alias_count, expect_alias_count);
    }

    #[test]
    fn alias_count() {
        check_alias_count(
            r"{
                value1
            }",
            0,
        );

        check_alias_count(
            r"
            {
                obj {
                    a
                    alias: b #1
                }
            }",
            1,
        );

        check_alias_count(
            r"
            {
                value1
                alias1: value2 #1
                alias2: value2 #2
            }",
            2,
        );

        check_alias_count(
            r"
            {
                value1
                alias: value2 #1
                obj {
                    a
                    b: alias2 #2
                }
            }",
            2,
        );
    }
}
