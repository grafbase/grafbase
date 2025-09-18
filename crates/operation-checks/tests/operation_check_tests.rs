use operation_checks::CheckParams;
use std::{fs, path::Path};

fn run_test(case_path: &Path) {
    let case = fs::read_to_string(case_path).unwrap();
    let mut sections = case.split("# --- #");

    let source_schema_string = sections.next().expect("source schema section missing");
    let source_schema: operation_checks::Schema =
        async_graphql_parser::parse_schema(source_schema_string).unwrap().into();

    let target_schema_string = sections.next().expect("target schema section missing");
    let target_schema: operation_checks::Schema =
        async_graphql_parser::parse_schema(target_schema_string).unwrap().into();

    let mut field_usage = operation_checks::FieldUsage::default();

    for operation in sections {
        let parsed_query = async_graphql_parser::parse_query(operation).unwrap();
        let operation = operation_checks::Operation::from(parsed_query);
        operation_checks::aggregate_field_usage(&operation, &source_schema, &mut field_usage);
    }

    let [result_forward, result_backward] = [
        (
            &source_schema_string,
            &source_schema,
            &target_schema_string,
            &target_schema,
        ),
        (
            &target_schema_string,
            &target_schema,
            &source_schema_string,
            &source_schema,
        ),
    ]
    .map(|(source_str, source, target_str, target)| {
        let diff = graphql_schema_diff::diff(source_str, target_str).unwrap();
        let params = CheckParams {
            source,
            target,
            diff: &diff,
            field_usage: &field_usage,
        };
        operation_checks::check(&params)
    });

    let rendered = format!("Forward:\n{:#?}\n\nBackward:\n{:#?}\n", result_forward, result_backward);

    insta::assert_snapshot!("result", rendered);
}

#[test]
fn operation_check_tests() {
    insta::glob!("cases/*.graphql", |graphql_file_path| {
        let test_name = graphql_file_path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({
            snapshot_path => "cases",
            prepend_module_to_snapshot => false,
            snapshot_suffix => test_name,
        }, {
            run_test(graphql_file_path);
        });
    });
}
