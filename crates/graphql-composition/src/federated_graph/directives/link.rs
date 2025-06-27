use std::collections::HashMap;

use cynic_parser_deser::{ConstDeserializer, ValueDeserialize};
use serde::Deserialize;

/// directive @link(
///   url: String!,
///   as: String,
///   import: [Import],
///   for: Purpose)
///   repeatable on SCHEMA
///
/// Source: https://specs.apollo.dev/link/v1.0/
#[derive(Debug)]
pub struct LinkDirective<'a> {
    pub url: &'a str,
    pub r#as: Option<&'a str>,
    pub import: Option<Vec<Import<'a>>>,
    #[expect(unused)]
    pub r#for: Option<Purpose>,
}

impl<'a> ValueDeserialize<'a> for LinkDirective<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let fields = input
            .as_object()
            .ok_or_else(|| cynic_parser_deser::Error::custom("Bad link directive", input.span()))?;

        let mut url = None;
        let mut r#as = None;
        let mut import = None;
        let mut r#for = None;

        for field in fields {
            match field.name() {
                "url" => {
                    url = Some(field.value().as_str().ok_or_else(|| {
                        cynic_parser_deser::Error::custom("Bad `url` argument in `@link` directive", field.name_span())
                    })?)
                }
                "as" => {
                    r#as = Some(field.value().as_str().ok_or_else(|| {
                        cynic_parser_deser::Error::custom("Bad `as` argument in `@link` directive", field.name_span())
                    })?)
                }
                "for" => r#for = Some(field.value().deserialize()?),
                "import" => import = Some(field.value().deserialize()?),
                other => {
                    return Err(cynic_parser_deser::Error::custom(
                        format!("Unknown argument `{other}` in `@link` directive"),
                        field.name_span(),
                    ));
                }
            }
        }

        let Some(url) = url else {
            return Err(cynic_parser_deser::Error::custom(
                "Missing `url` argument in `@link` directive",
                input.span(),
            ));
        };

        Ok(LinkDirective {
            url,
            r#as,
            import,
            r#for,
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum Purpose {
    Security,
    Execution,
}

impl<'a> ValueDeserialize<'a> for Purpose {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let str: &str = input.deserialize()?;

        match str {
            "SECURITY" => Ok(Purpose::Security),
            "EXECUTION" => Ok(Purpose::Execution),
            _ => Err(cynic_parser_deser::Error::custom("Bad purpose", input.span())),
        }
    }
}

#[derive(Debug)]
pub enum Import<'a> {
    String(&'a str),
    Qualified(QualifiedImport<'a>),
}

#[derive(Debug)]
pub struct QualifiedImport<'a> {
    pub name: &'a str,
    pub r#as: Option<&'a str>,
}

impl<'a> ValueDeserialize<'a> for QualifiedImport<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let Some(object) = input.as_object() else {
            return Err(cynic_parser_deser::Error::Custom {
                text: "Bad import".to_owned(),
                span: input.span(),
            });
        };

        let mut fields: HashMap<&str, _> = object.fields().map(|field| (field.name(), field)).collect();

        if fields.len() > 2 {
            return Err(cynic_parser_deser::Error::Custom {
                text: "Bad import".to_owned(),
                span: input.span(),
            });
        }

        let Some(name) = fields.remove("name").and_then(|field| field.value().as_str()) else {
            return Err(cynic_parser_deser::Error::Custom {
                text: "Bad import".to_owned(),
                span: input.span(),
            });
        };

        let r#as = fields
            .remove("as")
            .map(|alias| {
                alias
                    .value()
                    .as_str()
                    .ok_or_else(|| cynic_parser_deser::Error::custom("Bad import", input.span()))
            })
            .transpose()?;

        Ok(QualifiedImport { name, r#as })
    }
}

impl<'a> ValueDeserialize<'a> for Import<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        if let Some(string) = input.as_str() {
            return Ok(Import::String(string));
        }

        if input.as_object().is_some() {
            return Ok(Import::Qualified(input.deserialize()?));
        }

        Err(cynic_parser_deser::Error::custom("Bad import", input.span()))
    }
}
