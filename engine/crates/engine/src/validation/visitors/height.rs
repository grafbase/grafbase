use std::collections::HashSet;

use engine_parser::types::Field;

use crate::{
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    Positioned,
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

impl<'ctx, 'a> Visitor<'ctx> for HeightCalculate<'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx>, field: &'ctx Positioned<Field>) {
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

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'ctx>, _field: &'ctx Positioned<Field>) {
        self.variable_stack.pop();
    }
}

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

    fn check_height(query: &str, expect_height: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut height = 0;
        let mut height_calculate = HeightCalculate::new(&mut height);
        visit(&mut height_calculate, &mut ctx, &doc);
        assert_eq!(height, expect_height);
    }

    #[test]
    fn depth() {
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
