mod abstract_types;
mod basic;
mod cycle;
mod entities;
mod flatten;
mod inaccessible;
mod interface;
mod interface_object;
mod introspection;
mod mutation;
mod provides;
mod shared_root;
mod sibling_dependencies;
mod tea_shop;
mod typename;

use itertools::Itertools;

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("engine_query_planning=debug".to_string()))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .without_time()
        .init();
}

#[macro_export]
macro_rules! assert_solving_snapshots {
    ($name: expr, $schema: expr, $query: expr) => {
        let schema = ::schema::Schema::from_sdl_or_panic($schema);
        let name = $name;
        let query = $query;
        let mut operation = ::operation::Operation::parse(&schema, None, query).unwrap();

        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };

        let query_solution_space = $crate::Query::generate_solution_space(&schema, &operation).unwrap();
        insta::assert_snapshot!(
            format!("{name}-graph"),
            query_solution_space.to_dot_graph(ctx),
            &query_solution_space.to_pretty_dot_graph(ctx)
        );

        let mut solver = $crate::solve::Solver::initialize(&schema, &operation, &query_solution_space).unwrap();
        insta::assert_snapshot!(
            format!("{name}-solver"),
            solver.to_dot_graph(),
            &solver.to_pretty_dot_graph()
        );

        solver.execute().unwrap();
        insta::assert_snapshot!(
            format!("{name}-solved"),
            solver.to_dot_graph(),
            &solver.to_pretty_dot_graph()
        );

        let solution = solver.into_solution();
        let crude_solved_query =
            $crate::solve::generate_crude_solved_query(&schema, &operation, query_solution_space, solution).unwrap();
        insta::assert_snapshot!(
            format!("{name}-partial-solution"),
            crude_solved_query.to_dot_graph(ctx),
            &crude_solved_query.to_pretty_dot_graph(ctx)
        );

        let solved_query = $crate::post_process::post_process(&schema, &mut operation, crude_solved_query);
        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };
        insta::assert_snapshot!(
            format!("{name}-finalized-solution"),
            solved_query.to_dot_graph(ctx),
            &solved_query.to_pretty_dot_graph(ctx)
        );
    };
}

#[allow(unused)]
fn strdiff(before: &str, after: &str) -> String {
    similar::TextDiff::from_lines(before, after)
        .iter_all_changes()
        .filter_map(|change| match change.tag() {
            similar::ChangeTag::Equal => None,
            similar::ChangeTag::Delete => Some(('-', change)),
            similar::ChangeTag::Insert => Some(('+', change)),
        })
        .format_with("", |(tag, change), f| f(&format_args!("{}{}", tag, change)))
        .to_string()
}
