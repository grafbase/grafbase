pub(crate) struct LinkUrl {
    #[expect(unused)]
    pub(crate) url: url::Url,
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
}

/// https://specs.apollo.dev/link/v1.0/#@link.url
/// With support for extension manifests
pub(crate) fn parse_link_url(url: &str) -> Option<LinkUrl> {
    // Must be a url, or treated as an opaque identifier (which is valid).
    let url = url::Url::parse(url).ok()?;

    let segments = url.path_segments()?;

    let mut reversed_segments = segments.rev();

    let Some(maybe_version_or_name) = reversed_segments.next() else {
        return Some(LinkUrl {
            url,
            name: None,
            version: None,
        });
    };

    if is_valid_version(maybe_version_or_name) {
        let name = reversed_segments
            .next()
            .filter(|s| is_valid_graphql_name(s))
            .map(String::from);

        let version = Some(maybe_version_or_name.to_owned());

        Some(LinkUrl { url, name, version })
    } else if is_valid_graphql_name(maybe_version_or_name) {
        let name = Some(maybe_version_or_name.to_owned());

        Some(LinkUrl {
            url,
            name,
            version: None,
        })
    } else {
        Some(LinkUrl {
            url,
            name: None,
            version: None,
        })
    }
}

fn is_valid_version(s: &str) -> bool {
    let mut chars = s.chars();

    let Some('v') = chars.next() else { return false };

    let Some(digit) = chars.next() else { return false };

    if !digit.is_ascii_digit() {
        return false;
    };

    chars.all(|char| char.is_ascii_digit() || char == '.')
}

fn is_valid_graphql_name(s: &str) -> bool {
    let mut chars = s.chars();

    let Some(first_char) = chars.next() else {
        return false;
    };

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}
