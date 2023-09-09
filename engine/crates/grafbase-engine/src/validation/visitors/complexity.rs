use grafbase_engine_parser::types::{ExecutableDocument, OperationDefinition, VariableDefinition};
use grafbase_engine_value::Name;

use crate::{
    parser::types::Field,
    registry::{ComplexityType, MetaType, MetaTypeName},
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    Positioned,
};

pub struct ComplexityCalculate<'ctx, 'a> {
    pub complexity: &'a mut usize,
    pub complexity_stack: Vec<usize>,
    pub variable_definition: Option<&'ctx [Positioned<VariableDefinition>]>,
}

impl<'ctx, 'a> ComplexityCalculate<'ctx, 'a> {
    pub fn new(complexity: &'a mut usize) -> Self {
        Self {
            complexity,
            complexity_stack: Default::default(),
            variable_definition: None,
        }
    }
}

impl<'ctx, 'a> Visitor<'ctx> for ComplexityCalculate<'ctx, 'a> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_document(&mut self, _ctx: &mut VisitorContext<'ctx>, _doc: &'ctx ExecutableDocument) {
        self.complexity_stack.push(0);
    }

    fn exit_document(&mut self, _ctx: &mut VisitorContext<'ctx>, _doc: &'ctx ExecutableDocument) {
        *self.complexity = self.complexity_stack.pop().unwrap();
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _name: Option<&'ctx Name>,
        operation_definition: &'ctx Positioned<OperationDefinition>,
    ) {
        self.variable_definition = Some(&operation_definition.node.variable_definitions);
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'_>, _field: &Positioned<Field>) {
        self.complexity_stack.push(0);
    }

    fn exit_field(&mut self, ctx: &mut VisitorContext<'ctx>, field: &'ctx Positioned<Field>) {
        let children_complex = self.complexity_stack.pop().unwrap();

        if let Some(MetaType::Object(object)) = ctx.parent_type() {
            if let Some(meta_field) = object
                .fields
                .get(MetaTypeName::concrete_typename(field.node.name.node.as_str()))
            {
                if let Some(compute_complexity) = &meta_field.compute_complexity {
                    match compute_complexity {
                        ComplexityType::Const(n) => {
                            *self.complexity_stack.last_mut().unwrap() += n;
                        }
                        ComplexityType::Fn(f) => {
                            if meta_field.ty.is_list() {
                                match f(ctx, self.variable_definition.unwrap(), &field.node, children_complex) {
                                    Ok(n) => {
                                        *self.complexity_stack.last_mut().unwrap() += n;
                                    }
                                    Err(err) => ctx.report_error(vec![field.pos], err.to_string()),
                                }
                            }
                        }
                    }

                    return;
                }
            }
        }

        *self.complexity_stack.last_mut().unwrap() += 1 + children_complex;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        parser::parse_query,
        validation::{visit, VisitorContext},
        EmptyMutation, EmptySubscription, Object, Schema,
    };

    struct Query;

    #[derive(Copy, Clone)]
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
        async fn value(&self) -> i32 {
            todo!()
        }

        async fn obj(&self) -> MyObj {
            todo!()
        }

        #[graphql(complexity = "count * child_complexity")]
        #[allow(unused_variables)]
        async fn objs(&self, #[graphql(default_with = "5")] count: usize) -> Vec<MyObj> {
            todo!()
        }

        #[graphql(complexity = 3)]
        async fn d(&self) -> MyObj {
            todo!()
        }
    }

    fn check_complex(query: &str, expect_complex: usize) {
        let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut complex = 0;
        let mut complex_calculate = ComplexityCalculate::new(&mut complex);
        visit(&mut complex_calculate, &mut ctx, &doc);
        assert_eq!(complex, expect_complex);
    }

    #[test]
    fn complex_object() {
        check_complex(
            r#"
        {
            value #1
        }"#,
            1,
        );

        check_complex(
            r#"
        {
            value #1
            d #3
        }"#,
            4,
        );

        check_complex(
            r#"
        {
            value obj { #2
                a b #2
            }
        }"#,
            4,
        );

        check_complex(
            r#"
        {
            value obj { #2
                a b obj { #3
                    a b obj { #3
                        a #1
                    }
                }
            }
        }"#,
            9,
        );

        check_complex(
            r#"
        fragment A on MyObj {
            a b ... A2 #2
        }

        fragment A2 on MyObj {
            obj { # 1
                a # 1
            }
        }

        query {
            obj { # 1
                ... A
            }
        }"#,
            5,
        );

        check_complex(
            r#"
        {
            obj { # 1
                ... on MyObj {
                    a b #2
                    ... on MyObj {
                        obj { #1
                            a #1
                        }
                    }
                }
            }
        }"#,
            5,
        );

        check_complex(
            r#"
        {
            objs(count: 10) {
                a b
            }
        }"#,
            20,
        );

        check_complex(
            r#"
        {
            objs {
                a b
            }
        }"#,
            10,
        );

        check_complex(
            r#"
        fragment A on MyObj {
            a b
        }

        query {
            objs(count: 10) {
                ... A
            }
        }"#,
            20,
        );
    }
}
