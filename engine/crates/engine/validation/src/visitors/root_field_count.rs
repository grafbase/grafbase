use engine_parser::{types::Field, Positioned};

use crate::{
    registries::ValidationRegistry,
    visitor::{VisitMode, Visitor, VisitorContext},
};

pub struct RootFieldCountCalculate<'a> {
    root_field_count: &'a mut usize,
    current_depth: usize,
}

impl<'a> RootFieldCountCalculate<'a> {
    pub fn new(root_field_count: &'a mut usize) -> Self {
        Self {
            root_field_count,
            current_depth: 0,
        }
    }
}

impl<'ctx, 'a, Registry> Visitor<'ctx, Registry> for RootFieldCountCalculate<'a>
where
    Registry: ValidationRegistry,
{
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx, Registry>, _field: &'ctx Positioned<Field>) {
        self.current_depth += 1;
        if self.current_depth == 1 {
            *self.root_field_count += 1;
        }
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'ctx, Registry>, _field: &'ctx Positioned<Field>) {
        self.current_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::diverging_sub_expression)]

    use super::*;

    use engine::{EmptyMutation, EmptySubscription, Object, Schema};
    use {
        crate::{visit, VisitorContext},
        engine_parser::parse_query,
    };

    struct Query;

    struct MyObj;

    #[Object]
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

    #[Object]
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

    fn check_root_field_count(query: &str, expect_root_field_count: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let registry = registry_upgrade::convert_v1_to_v2(registry);

        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut root_field_count = 0;
        let mut root_field_count_calculate = RootFieldCountCalculate::new(&mut root_field_count);
        visit(&mut root_field_count_calculate, &mut ctx, &doc);
        assert_eq!(root_field_count, expect_root_field_count);
    }

    #[test]
    fn root_field_count() {
        check_root_field_count(
            r"{
            value1 #1
        }",
            1,
        );

        check_root_field_count(
            r"
        {
            obj { #1
                a b
            }
        }",
            1,
        );

        check_root_field_count(
            r"
        {
            value1 #1
            value2 #2
        }",
            2,
        );

        check_root_field_count(
            r"
        {
            value1 #1
            alias: value1 #2
            obj { #3
                a
            }
        }",
            3,
        );
    }
}
