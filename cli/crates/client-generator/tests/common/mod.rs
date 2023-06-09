pub use expect_test::expect;
pub use indoc::indoc;

use dprint_plugin_typescript::configuration::{
    Configuration, ConfigurationBuilder, QuoteStyle, SemiColons, TrailingCommas,
};
use expect_test::Expect;
use std::{path::PathBuf, sync::OnceLock};

#[track_caller]
pub fn expect_ts(result: impl ToString, expected: &Expect) {
    static TS_CONFIG: OnceLock<Configuration> = OnceLock::new();

    let config = TS_CONFIG.get_or_init(|| {
        ConfigurationBuilder::new()
            .line_width(80)
            .prefer_hanging(true)
            .prefer_single_line(false)
            .trailing_commas(TrailingCommas::Never)
            .quote_style(QuoteStyle::PreferSingle)
            .indent_width(2)
            .semi_colons(SemiColons::Asi)
            .build()
    });

    let result = dprint_plugin_typescript::format_text(&PathBuf::from("test.ts"), &result.to_string(), config)
        .unwrap()
        .unwrap();

    expect_raw_ts(result, expected);
}

pub fn expect_raw_ts(result: impl ToString, expected: &Expect) {
    expected.assert_eq(&result.to_string());
}
