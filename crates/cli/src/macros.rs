#[macro_export]
/// behaves like [indoc::indoc!], but also trims leading and trailing whitespace
macro_rules! trimdoc {
    ($doc: literal) => {{
        indoc::indoc! {$doc}.trim()
    }};
}

#[test]
fn trimdoc_test() {
    assert_eq!(
        "test",
        trimdoc! {"
            test   
        "}
    )
}
