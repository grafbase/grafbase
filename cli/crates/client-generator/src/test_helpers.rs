pub use expect_test::expect;
pub use indoc::indoc;

use expect_test::Expect;
use std::path::PathBuf;

#[track_caller]
pub fn expect_ts(result: impl ToString, expected: &Expect) {
    let config = crate::prettier_configuration();
    let result = dprint_plugin_typescript::format_text(&PathBuf::from("test.ts"), &result.to_string(), config)
        .unwrap()
        .unwrap();

    expect_raw_ts(result, expected);
}

pub fn expect_raw_ts(result: impl ToString, expected: &Expect) {
    expected.assert_eq(&result.to_string());
}
