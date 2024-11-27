use super::*;

impl<'a> Path<'a> {
    pub fn parse(s: &'a str) -> ParseResult<Path<'a>> {
        let mut segments = s.split('.');

        let first_segment = segments.next().filter(|s| !s.is_empty()).ok_or(ParseError)?;

        // Is it a schema definition or extension?
        if let Some(suffix) = first_segment.strip_prefix(":schema") {
            if segments.next().is_some() {
                return Err(ParseError);
            }

            if suffix.is_empty() {
                return Ok(Path::SchemaDefinition);
            }

            let index = parse_index(suffix)?;
            return Ok(Path::SchemaExtension(index));
        }

        // Is it a directive definition?
        if let Some(name) = first_segment.strip_prefix('@') {
            if !is_valid_graphql_name(name) {
                return Err(ParseError);
            }

            if segments.next().is_some() {
                return Err(ParseError);
            }

            return Ok(Path::DirectiveDefinition(name));
        }

        // The only remaining possibility is a type definition
        let (name, idx) = parse_name_and_optional_index(first_segment)?;

        if let Some(idx) = idx {
            Ok(Path::TypeExtension(name, idx, Self::parse_path_in_type(segments)?))
        } else {
            Ok(Path::TypeDefinition(name, Self::parse_path_in_type(segments)?))
        }
    }

    fn parse_path_in_type(mut segments: impl Iterator<Item = &'a str>) -> ParseResult<Option<PathInType<'a>>> {
        let segment = match segments.next() {
            Some("") => return Err(ParseError),
            Some(segment) => segment,
            None => return Ok(None),
        };

        // Is it a directive path?
        if let Some(suffix) = segment.strip_prefix('@') {
            let (name, idx) = parse_name_and_optional_index(suffix)?;

            if segments.next().is_some() {
                return Err(ParseError);
            }

            let Some(idx) = idx else {
                return Err(ParseError);
            };

            return Ok(Some(PathInType::InDirective(name, idx)));
        }

        // Is it an interface implementation?
        if let Some(suffix) = segment.strip_prefix('&') {
            if !is_valid_graphql_name(suffix) {
                return Err(ParseError);
            }

            if segments.next().is_some() {
                return Err(ParseError);
            }

            return Ok(Some(PathInType::InterfaceImplementation(suffix)));
        }

        // The only other possibility is that we have a field / union member / enum value name.
        if !is_valid_graphql_name(segment) {
            return Err(ParseError);
        }

        Ok(Some(PathInType::InField(segment, Self::parse_path_in_field(segments)?)))
    }

    fn parse_path_in_field(mut segments: impl Iterator<Item = &'a str>) -> ParseResult<Option<PathInField<'a>>> {
        let segment = match segments.next() {
            Some("") => return Err(ParseError),
            Some(segment) => segment,
            None => return Ok(None),
        };

        // To be implemented.
        if segments.next().is_some() {
            return Err(ParseError);
        }

        // Is it a directive path?
        if let Some(suffix) = segment.strip_prefix('@') {
            let (name, Some(idx)) = parse_name_and_optional_index(suffix)? else {
                return Err(ParseError);
            };

            if segments.next().is_some() {
                return Err(ParseError);
            }

            return Ok(Some(PathInField::InDirective(name, idx)));
        }

        // The only other possibility is that we have an argument name.
        if !is_valid_graphql_name(segment) {
            return Err(ParseError);
        }

        Ok(Some(PathInField::InArgument(segment)))
    }
}

fn parse_name_and_optional_index(s: &str) -> ParseResult<(&str, Option<usize>)> {
    let bracket_idx = s.chars().position(|c| c == '[');

    let name = bracket_idx.map(|idx| &s[..idx]).unwrap_or(s);

    if !is_valid_graphql_name(name) {
        return Err(ParseError);
    }

    let idx = if let Some(bracket_idx) = bracket_idx {
        Some(parse_index(&s[bracket_idx..])?)
    } else {
        None
    };

    Ok((name, idx))
}

fn parse_index(s: &str) -> ParseResult<usize> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(ParseError);
    }

    let idx = &s[1..s.len() - 1];

    if !idx.chars().any(|char| char.is_ascii_digit()) {
        return Err(ParseError);
    }

    idx.parse::<usize>().map_err(|_| ParseError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index_tests() {
        assert_eq!(parse_index("[0]").unwrap(), 0);
        assert_eq!(parse_index("[10]").unwrap(), 10);
        assert_eq!(parse_index("[1243247]").unwrap(), 1243247);
        assert_eq!(parse_index("[03]").unwrap(), 3);

        assert!(parse_index("3").is_err());
        assert!(parse_index("[3").is_err());
        assert!(parse_index("3]").is_err());
        assert!(parse_index("[[3]]").is_err());
        assert!(parse_index("[0x3]").is_err());
        assert!(parse_index("[-3]").is_err());
    }
}
