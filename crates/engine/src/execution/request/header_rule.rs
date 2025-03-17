use std::{borrow::Cow, str::FromStr, sync::OnceLock};

use http::{HeaderName, header};
use schema::{
    ForwardHeaderRule, HeaderRule, HeaderRuleVariant, InsertHeaderRule, NameOrPattern, RemoveHeaderRule,
    RenameDuplicateHeaderRule,
};

pub(crate) fn apply_header_rules<'ctx>(
    gateway_headers: &http::HeaderMap,
    rules: impl Iterator<Item = HeaderRule<'ctx>>,
    subgraph_headers: &mut http::HeaderMap,
) {
    for rule in rules {
        match rule.variant() {
            HeaderRuleVariant::Forward(rule) => {
                handle_forward(gateway_headers, rule, subgraph_headers);
            }
            HeaderRuleVariant::Insert(rule) => {
                handle_insert(rule, subgraph_headers);
            }
            HeaderRuleVariant::Remove(rule) => handle_remove(rule, subgraph_headers),
            HeaderRuleVariant::RenameDuplicate(rule) => {
                handle_rename_duplicate(gateway_headers, rule, subgraph_headers);
            }
        }
    }
}

fn handle_rename_duplicate(
    gateway_headers: &http::HeaderMap,
    rule: RenameDuplicateHeaderRule<'_>,
    subgraph_headers: &mut http::HeaderMap,
) {
    let Ok(name) = http::HeaderName::from_str(rule.name()) else {
        return;
    };

    let Ok(rename) = http::HeaderName::from_str(rule.rename()) else {
        return;
    };

    if is_header_denied(&rename) {
        return;
    }

    let value = gateway_headers.get(&name).map(Cow::Borrowed).or_else(|| {
        rule.default()
            .and_then(|d| http::HeaderValue::from_str(d).ok())
            .map(Cow::Owned)
    });

    if let Some(value) = value {
        subgraph_headers.append(name, value.clone().into_owned());
        subgraph_headers.append(rename, value.into_owned());
    }
}

fn handle_remove(rule: RemoveHeaderRule<'_>, subgraph_headers: &mut http::HeaderMap) {
    match rule.name() {
        NameOrPattern::Pattern(regex) => {
            // https://github.com/hyperium/http/issues/632
            let delete_list: Vec<_> = subgraph_headers
                .keys()
                .filter(|key| regex.is_match(key.as_str()))
                .map(Clone::clone)
                .collect();

            for key in delete_list {
                subgraph_headers.remove(key);
            }
        }
        NameOrPattern::Name(name) => {
            subgraph_headers.remove(name);
        }
    }
}

fn handle_insert(rule: InsertHeaderRule<'_>, subgraph_headers: &mut http::HeaderMap) {
    let name = http::HeaderName::from_bytes(rule.name().as_bytes()).ok();
    let value = http::HeaderValue::from_str(rule.value()).ok();

    if let Some((name, value)) = name.zip(value) {
        if is_header_denied(&name) {
            return;
        }

        subgraph_headers.append(name, value);
    }
}

fn handle_forward(
    gateway_headers: &http::HeaderMap,
    rule: ForwardHeaderRule<'_>,
    subgraph_headers: &mut http::HeaderMap,
) {
    match rule.name() {
        NameOrPattern::Pattern(regex) => {
            let filtered = gateway_headers
                .iter()
                .filter(|(name, _)| !is_header_denied(name))
                .filter(|(name, _)| regex.is_match(name.as_str()));

            for (name, value) in filtered {
                match rule.rename().and_then(|s| http::HeaderName::from_str(s).ok()) {
                    Some(rename) => {
                        subgraph_headers.append(rename, value.clone());
                    }
                    None => {
                        subgraph_headers.append(name.clone(), value.clone());
                    }
                }
            }
        }
        NameOrPattern::Name(name) => {
            let Ok(name) = http::HeaderName::from_str(name) else {
                return;
            };

            // if a previous rule added a header with the same name, remove the old one.
            subgraph_headers.remove(&name);

            let found = gateway_headers.get_all(&name);

            let name = match rule.rename() {
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

            let default = rule.default().and_then(|d| http::HeaderValue::from_str(d).ok());
            let mut inserted = false;

            for header in found {
                inserted = true;
                subgraph_headers.append(name.clone(), header.clone());
            }

            match default {
                Some(value) if !inserted => {
                    subgraph_headers.insert(name, value);
                }
                _ => (),
            }
        }
    }
}

fn is_header_denied(name: &HeaderName) -> bool {
    find_matching_denied_header(name).is_some()
}

pub(crate) fn find_matching_denied_header(name: &HeaderName) -> Option<&'static str> {
    static DENY_LIST: OnceLock<[&'static str; 21]> = OnceLock::new();
    let blacklist = DENY_LIST.get_or_init(|| {
        [
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
            header::ORIGIN.as_str(),
            header::HOST.as_str(),
            header::SEC_WEBSOCKET_VERSION.as_str(),
            header::SEC_WEBSOCKET_KEY.as_str(),
            header::SEC_WEBSOCKET_ACCEPT.as_str(),
            header::SEC_WEBSOCKET_PROTOCOL.as_str(),
            header::SEC_WEBSOCKET_EXTENSIONS.as_str(),
        ]
    });
    blacklist.iter().find(|denied| **denied == name.as_str()).copied()
}
