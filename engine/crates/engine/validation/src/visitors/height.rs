use std::collections::HashSet;

use engine_parser::{types::Field, Positioned};

use crate::{
    registries::ValidationRegistry,
    visitor::{VisitMode, Visitor, VisitorContext},
};

pub struct HeightCalculate<'a> {
    height: &'a mut usize,
    variable_stack: Vec<HashSet<String>>,
}

impl<'a> HeightCalculate<'a> {
    pub fn new(height: &'a mut usize) -> Self {
        Self {
            height,
            variable_stack: vec![HashSet::new()],
        }
    }
}

impl<'ctx, 'a, Registry> Visitor<'ctx, Registry> for HeightCalculate<'a>
where
    Registry: ValidationRegistry,
{
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx, Registry>, field: &'ctx Positioned<Field>) {
        {
            let field_name = field.name.node.as_str();
            let last_stack = self.variable_stack.last_mut().expect("must exist");
            if !last_stack.contains(field_name) {
                last_stack.insert(field_name.to_owned());
                *self.height += 1;
            }
        }
        self.variable_stack.push(HashSet::new());
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'ctx, Registry>, _field: &'ctx Positioned<Field>) {
        self.variable_stack.pop();
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

    fn check_height(query: &str, expect_height: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let registry = registry_upgrade::convert_v1_to_v2(registry);

        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut height = 0;
        let mut height_calculate = HeightCalculate::new(&mut height);
        visit(&mut height_calculate, &mut ctx, &doc);
        assert_eq!(height, expect_height);
    }

    #[test]
    fn height() {
        check_height(
            r"{
            value1 #1
        }",
            1,
        );

        check_height(
            r"
        {
            obj { #1
                a #2
                b #3
            }
        }",
            3,
        );

        check_height(
            r"
        {
            value1 #1
            value2 #2
        }",
            2,
        );

        check_height(
            r"
        {
            value1 #1
            alias: value1
            obj { #2
                a #3
            }
        }",
            3,
        );
    }
}
