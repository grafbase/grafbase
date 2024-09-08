use std::collections::HashSet;
use std::fmt::Write;

use indoc::formatdoc;
use itertools::Itertools;

use crate::domain::Domain;

pub fn generate_module_base_content<S>(domain: &Domain, submodules: &[S]) -> String
where
    S: std::fmt::Display,
{
    let submodules = submodules.iter().map(|s| s.to_string()).collect::<HashSet<_>>();
    let mod_definitions = submodules
        .iter()
        .format_with("\n", |submodule, f| f(&format_args!("mod {submodule};")));
    let pub_use_mod = submodules
        .iter()
        .format_with("\n", |submodule, f| f(&format_args!("pub use {submodule}::*;")));

    let mut contents = formatdoc!(
        r#"
        //! ===================
        //! !!! DO NOT EDIT !!!
        //! ===================
        //! Generated with: `cargo run -p {pkg}`
        //! Source file: <{pkg} dir>/{source}
        "#,
        pkg = env!("CARGO_PKG_NAME"),
        source = domain.source.to_string_lossy()
    );
    if !submodules.is_empty() {
        write!(&mut contents, "{mod_definitions}\n\n{pub_use_mod}\n").unwrap();
    }
    contents
}
