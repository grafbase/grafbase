use std::{borrow::Cow, str::FromStr, sync::OnceLock};

use http::{header, HeaderName};
use schema::{HeaderRuleWalker, NameOrPatternRef};

use crate::engine::RequestContext;

pub(super) fn create_subgraph_headers_with_rules<'ctx, C>(
    request_context: &'ctx RequestContext<C>,
    rules: impl Iterator<Item = HeaderRuleWalker<'ctx>>,
    default: http::HeaderMap,
) -> http::HeaderMap {
    let mut headers = default;

    for header in rules {
        match header.rule() {
            schema::HeaderRuleRef::Forward { name, default, rename } => match name {
                NameOrPatternRef::Pattern(regex) => {
                    let filtered = request_context
                        .headers
                        .iter()
                        .filter(|(name, _)| !is_header_denied(name))
                        .filter(|(name, _)| regex.is_match(name.as_str()));

                    for (name, value) in filtered {
                        // if a previous rule added a header with the same name, remove the old one.
                        headers.remove(name);

                        match rename.and_then(|s| http::HeaderName::from_str(s).ok()) {
                            Some(rename) => {
                                headers.insert(rename, value.clone());
                            }
                            None => {
                                headers.insert(name.clone(), value.clone());
                            }
                        }
                    }
                }
                NameOrPatternRef::Name(name) => {
                    // if a previous rule added a header with the same name, remove the old one.
                    headers.remove(name);

                    let header = request_context.headers.get(name);
                    let default = default.and_then(|d| http::HeaderValue::from_str(d).ok());

                    let name = match rename {
                        Some(rename) => rename,
                        None => name,
                    };

                    let Ok(name) = http::HeaderName::from_str(name) else {
                        continue;
                    };

                    if is_header_denied(&name) {
                        continue;
                    }

                    match (header, default) {
                        (None, Some(default)) => {
                            headers.insert(name, default);
                        }
                        (Some(value), _) => {
                            headers.insert(name, value.clone());
                        }
                        _ => (),
                    };
                }
            },
            schema::HeaderRuleRef::Insert { name, value } => {
                let name = http::HeaderName::from_bytes(name.as_bytes()).ok();
                let value = http::HeaderValue::from_str(value).ok();

                if let Some((name, value)) = name.zip(value) {
                    if is_header_denied(&name) {
                        continue;
                    }
                    headers.insert(name, value);
                }
            }
            schema::HeaderRuleRef::Remove { name } => match name {
                schema::NameOrPatternRef::Pattern(regex) => {
                    // https://github.com/hyperium/http/issues/632
                    let delete_list: Vec<_> = headers
                        .keys()
                        .filter(|key| regex.is_match(key.as_str()))
                        .map(Clone::clone)
                        .collect();

                    for key in delete_list {
                        headers.remove(key);
                    }
                }
                schema::NameOrPatternRef::Name(name) => {
                    headers.remove(name);
                }
            },
            schema::HeaderRuleRef::RenameDuplicate { name, default, rename } => {
                let Ok(name) = http::HeaderName::from_str(name) else {
                    continue;
                };

                let Ok(rename) = http::HeaderName::from_str(rename) else {
                    continue;
                };
                if is_header_denied(&rename) {
                    continue;
                }

                let value = request_context.headers.get(&name).map(Cow::Borrowed).or_else(|| {
                    default
                        .and_then(|d| http::HeaderValue::from_str(d).ok())
                        .map(Cow::Owned)
                });

                if let Some(value) = value {
                    headers.insert(name, value.clone().into_owned());
                    headers.insert(rename, value.into_owned());
                }
            }
        }
    }

    headers
}

fn is_header_denied(name: &HeaderName) -> bool {
    static DENY_LIST: OnceLock<[&str; 14]> = OnceLock::new();
    let blacklist = DENY_LIST.get_or_init(|| {
        let mut blacklist = [
            header::ACCEPT.as_str(),
            header::ACCEPT_CHARSET.as_str(),
            header::ACCEPT_ENCODING.as_str(),
            header::ACCEPT_RANGES.as_str(),
            header::CONTENT_LENGTH.as_str(),
            header::CONTENT_TYPE.as_str(),
            // hop-by-hop headers
            header::CONNECTION.as_str(),
            "keep-alive",
            header::PROXY_AUTHENTICATE.as_str(),
            header::PROXY_AUTHORIZATION.as_str(),
            header::TE.as_str(),
            header::TRAILER.as_str(),
            header::TRANSFER_ENCODING.as_str(),
            header::UPGRADE.as_str(),
        ];
        blacklist.sort_unstable();
        blacklist
    });
    blacklist.binary_search(&name.as_str()).is_ok()
}
