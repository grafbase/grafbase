#![allow(dead_code)]

use std::{borrow::Cow, fmt};

use super::common::{Identifier, Quoted};

#[derive(Debug)]
pub struct Import<'a> {
    items: ImportItems<'a>,
    import_location: Quoted<'a>,
}

impl<'a> Import<'a> {
    pub fn all_as(import_location: impl Into<Cow<'a, str>>, alias: impl Into<Cow<'a, str>>) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::All { alias: alias.into() },
        }
    }

    pub fn items(import_location: impl Into<Cow<'a, str>>, items: &[&'a str]) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::Set(items.iter().map(|i| Identifier::new(*i)).collect()),
        }
    }

    pub fn push_item(&mut self, identifier: Identifier<'a>) {
        match self.items {
            ImportItems::All { .. } => self.items = ImportItems::Set(vec![identifier]),
            ImportItems::Set(ref mut items) => items.push(identifier),
        }
    }
}

impl<'a> fmt::Display for Import<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "import {} from {}", self.items, self.import_location)
    }
}

#[derive(Debug)]
pub enum ImportItems<'a> {
    All { alias: Cow<'a, str> },
    Set(Vec<Identifier<'a>>),
}

impl<'a> fmt::Display for ImportItems<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportItems::All { alias } => write!(f, "* as {alias}"),
            ImportItems::Set(ref identifiers) => {
                if identifiers.len() > 1 {
                    f.write_str("{ ")?;
                }

                for (i, ident) in identifiers.iter().enumerate() {
                    ident.fmt(f)?;

                    if i < identifiers.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                if identifiers.len() > 1 {
                    f.write_str(" }")?;
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Import;
    use crate::test_helpers::{expect, expect_ts};

    #[test]
    fn import_all() {
        let import = Import::all_as("graphql-request", "gql");

        let expected = expect![[r#"
            import * as gql from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }

    #[test]
    fn import_one() {
        let import = Import::items("graphql-request", &["gql"]);

        let expected = expect![[r#"
            import gql from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }

    #[test]
    fn import_many() {
        let import = Import::items("graphql-request", &["gql", "GraphQLClient"]);

        let expected = expect![[r#"
            import { gql, GraphQLClient } from 'graphql-request'
        "#]];

        expect_ts(import, &expected);
    }
}
