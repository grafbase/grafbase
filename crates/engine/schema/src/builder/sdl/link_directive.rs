use std::collections::HashMap;

use cynic_parser_deser::{ConstDeserializer, ValueDeserialize};

/// directive @link(
///   url: String!,
///   as: String,
///   import: [Import],
///   for: Purpose)
///   repeatable on SCHEMA
///
/// Source: https://specs.apollo.dev/link/v1.0/
#[derive(Debug)]
pub(crate) struct LinkDirective<'a> {
    pub(crate) url: LinkUrl<'a>,
    pub(crate) namespace: Option<String>,
    pub(crate) r#as: Option<&'a str>,
    pub(crate) import: Option<Vec<Import<'a>>>,
}

#[derive(Debug)]
pub(crate) struct LinkUrl<'a> {
    raw: &'a str,
    pub name: Option<String>,
    pub version: Option<semver::VersionReq>,
}

impl std::fmt::Display for LinkUrl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'a> LinkUrl<'a> {
    pub fn as_str(&self) -> &'a str {
        self.raw
    }
}

// Name must be consistent with the graphql-composition
impl<'a> From<&'a str> for LinkUrl<'a> {
    fn from(raw: &'a str) -> Self {
        if let Ok(url) = raw.parse::<url::Url>()
            && let Some(mut path) = url.path_segments()
            && let Some(mut last) = path.next_back()
        {
            if url.scheme() == "file" && last == "build" {
                let Some(segment) = path.next_back() else {
                    return LinkUrl {
                        raw,
                        name: None,
                        version: None,
                    };
                };
                last = segment
            }
            if let Ok(version) = format!("^{}", last.strip_prefix("v").unwrap_or(last)).parse::<semver::VersionReq>() {
                if let Some(penultimate) = path.next_back() {
                    LinkUrl {
                        raw,
                        version: Some(version),
                        name: Some(penultimate.to_string()),
                    }
                } else {
                    LinkUrl {
                        raw,
                        version: Some(version),
                        name: None,
                    }
                }
            } else {
                LinkUrl {
                    raw,
                    version: None,
                    name: Some(last.to_string()),
                }
            }
        } else {
            LinkUrl {
                raw,
                name: None,
                version: None,
            }
        }
    }
}

impl AsRef<str> for LinkUrl<'_> {
    fn as_ref(&self) -> &str {
        self.raw
    }
}

impl<'a> ValueDeserialize<'a> for LinkDirective<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let fields = input
            .as_object()
            .ok_or_else(|| cynic_parser_deser::Error::custom("Bad link directive", input.span()))?;

        let mut url = None;
        let mut r#as = None;
        let mut import = None;

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
                "import" => import = Some(field.value().deserialize()?),
                "for" => {}
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

        let url = LinkUrl::from(url);
        let namespace = r#as.or(url.name.as_deref()).map(str::to_string);

        Ok(LinkDirective {
            url,
            namespace,
            r#as,
            import,
        })
    }
}

#[derive(Debug)]
pub(crate) enum Import<'a> {
    String(&'a str),
    Qualified(QualifiedImport<'a>),
}

#[derive(Debug)]
pub(crate) struct QualifiedImport<'a> {
    pub(crate) name: &'a str,
    pub(crate) r#as: Option<&'a str>,
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
