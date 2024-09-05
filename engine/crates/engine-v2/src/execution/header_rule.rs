use std::{borrow::Cow, str::FromStr, sync::OnceLock};

use http::{header, HeaderName};
use schema::{HeaderRule, NameOrPatternRef};

use crate::engine::RequestContext;

pub(super) fn create_subgraph_headers_with_rules<'ctx, C>(
    request_context: &'ctx RequestContext<C>,
    rules: impl Iterator<Item = HeaderRule<'ctx>>,
    default: http::HeaderMap,
) -> http::HeaderMap {
    let mut headers = default;

    for header in rules {
        match header.rule() {
            schema::HeaderRuleRef::Forward { name, default, rename } => {
                handle_forward(&mut headers, name, request_context, rename, default);
            }
            schema::HeaderRuleRef::Insert { name, value } => {
                handle_insert(&mut headers, name, value);
            }
            schema::HeaderRuleRef::Remove { name } => handle_remove(&mut headers, name),
            schema::HeaderRuleRef::RenameDuplicate { name, default, rename } => {
                handle_rename_duplicate(&mut headers, name, rename, request_context, default);
            }
        }
    }

    headers
}

fn handle_rename_duplicate<C>(
    headers: &mut http::HeaderMap,
    name: &str,
    rename: &str,
    request_context: &RequestContext<C>,
    default: Option<&str>,
) {
    let Ok(name) = http::HeaderName::from_str(name) else {
        return;
    };

    let Ok(rename) = http::HeaderName::from_str(rename) else {
        return;
    };

    if is_header_denied(&rename) {
        return;
    }

    let value = request_context.headers.get(&name).map(Cow::Borrowed).or_else(|| {
        default
            .and_then(|d| http::HeaderValue::from_str(d).ok())
            .map(Cow::Owned)
    });

    if let Some(value) = value {
        headers.append(name, value.clone().into_owned());
        headers.append(rename, value.into_owned());
    }
}

fn handle_remove(headers: &mut http::HeaderMap, name: NameOrPatternRef<'_>) {
    match name {
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
    }
}

fn handle_insert(headers: &mut http::HeaderMap, name: &str, value: &str) {
    let name = http::HeaderName::from_bytes(name.as_bytes()).ok();
    let value = http::HeaderValue::from_str(value).ok();

    if let Some((name, value)) = name.zip(value) {
        if is_header_denied(&name) {
            return;
        }

        headers.append(name, value);
    }
}

fn handle_forward<C>(
    headers: &mut http::HeaderMap,
    name: NameOrPatternRef<'_>,
    request_context: &RequestContext<C>,
    rename: Option<&str>,
    default: Option<&str>,
) {
    match name {
        NameOrPatternRef::Pattern(regex) => {
            let filtered = request_context
                .headers
                .iter()
                .filter(|(name, _)| !is_header_denied(name))
                .filter(|(name, _)| regex.is_match(name.as_str()));

            for (name, value) in filtered {
                match rename.and_then(|s| http::HeaderName::from_str(s).ok()) {
                    Some(rename) => {
                        headers.append(rename, value.clone());
                    }
                    None => {
                        headers.append(name.clone(), value.clone());
                    }
                }
            }
        }
        NameOrPatternRef::Name(name) => {
            let Ok(name) = http::HeaderName::from_str(name) else {
                return;
            };

            // if a previous rule added a header with the same name, remove the old one.
            headers.remove(&name);

            let found = request_context.headers.get_all(&name);

            let name = match rename {
                Some(rename) => match http::HeaderName::from_str(rename) {
                    Ok(name) => name,
                    Err(_) => {
                        return;
                    }
                },
                None => name,
            };

            if is_header_denied(&name) {
                return;
            }

            let default = default.and_then(|d| http::HeaderValue::from_str(d).ok());
            let mut inserted = false;

            for header in found {
                inserted = true;
                headers.append(name.clone(), header.clone());
            }

            match default {
                Some(value) if !inserted => {
                    headers.insert(name, value);
                }
                _ => (),
            }
        }
    }
}

fn is_header_denied(name: &HeaderName) -> bool {
    static DENY_LIST: OnceLock<[&str; 15]> = OnceLock::new();
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
            header::HOST.as_str(),
        ];
        blacklist.sort_unstable();
        blacklist
    });
    blacklist.binary_search(&name.as_str()).is_ok()
}
