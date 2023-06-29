mod block;
mod class;
mod comment;
mod common;
mod document;
mod error;
mod expression;
mod function;
mod import;
mod interface;
mod statement;
mod transpile;
mod r#type;

#[cfg(test)]
mod test_helpers;

use dprint_plugin_typescript::configuration::{
    Configuration, ConfigurationBuilder, QuoteStyle, SemiColons, TrailingCommas,
};
use std::sync::OnceLock;

#[cfg(test)]
use expect_test as _;
#[cfg(test)]
use indoc as _;

pub(crate) use block::Block;
pub(crate) use class::Class;
pub(crate) use common::{Identifier, Template};
pub(crate) use document::Document;
pub(crate) use error::GeneratorError;
pub(crate) use expression::Expression;
pub(crate) use function::{Function, FunctionBody};
pub(crate) use import::Import;
pub(crate) use interface::Interface;

pub type Result<T> = std::result::Result<T, GeneratorError>;

pub use transpile::generate;

pub fn typescript_configuration() -> &'static Configuration {
    static TS_CONFIG: OnceLock<Configuration> = OnceLock::new();

    &*TS_CONFIG.get_or_init(|| {
        ConfigurationBuilder::new()
            .line_width(80)
            .prefer_hanging(true)
            .prefer_single_line(false)
            .trailing_commas(TrailingCommas::Never)
            .quote_style(QuoteStyle::PreferSingle)
            .indent_width(2)
            .semi_colons(SemiColons::Asi)
            .build()
    })
}
