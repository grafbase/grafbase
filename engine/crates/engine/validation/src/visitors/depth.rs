use engine_parser::{types::Field, Positioned};

use crate::{
    registries::AnyRegistry,
    visitor::{VisitMode, Visitor, VisitorContext},
};

pub struct DepthCalculate<'a> {
    max_depth: &'a mut usize,
    current_depth: usize,
}

impl<'a> DepthCalculate<'a> {
    pub fn new(max_depth: &'a mut usize) -> Self {
        Self {
            max_depth,
            current_depth: 0,
        }
    }
}

impl<'ctx, 'a, Registry> Visitor<'ctx, Registry> for DepthCalculate<'a>
where
    Registry: AnyRegistry,
{
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx, Registry>, _field: &'ctx Positioned<Field>) {
        self.current_depth += 1;
        *self.max_depth = (*self.max_depth).max(self.current_depth);
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
        async fn value(&self) -> i32 {
            todo!()
        }

        async fn obj(&self) -> MyObj {
            todo!()
        }
    }

    fn check_depth(query: &str, expect_depth: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let registry = registry_upgrade::convert_v1_to_v2(registry).unwrap();

        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut depth = 0;
        let mut depth_calculate = DepthCalculate::new(&mut depth);
        visit(&mut depth_calculate, &mut ctx, &doc);
        assert_eq!(depth, expect_depth);
    }

    #[test]
    fn depth() {
        check_depth(
            r"{
            value #1
        }",
            1,
        );

        check_depth(
            r"
        {
            obj { #1
                a b #2
            }
        }",
            2,
        );

        check_depth(
            r"
        {
            obj { # 1
                a b c { # 2
                    a b c { # 3
                        a b # 4
                    }
                }
            }
        }",
            4,
        );

        check_depth(
            r"
        fragment A on MyObj {
            a b ... A2 #2
        }

        fragment A2 on MyObj {
            obj {
                a #3
            }
        }

        query {
            obj { # 1
                ... A
            }
        }",
            3,
        );

        check_depth(
            r"
        {
            obj { # 1
                ... on MyObj {
                    a b #2
                    ... on MyObj {
                        obj {
                            a #3
                        }
                    }
                }
            }
        }",
            3,
        );
    }
}
