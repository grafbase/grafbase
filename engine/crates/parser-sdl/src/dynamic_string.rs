use engine::ServerError;
use itertools::Itertools;
use regex::Regex;
use serde_with::DeserializeFromStr;
use std::{collections::HashMap, sync::OnceLock};

/// A type representing a segment of a partially evaluated dynamic string.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DynamicStringSegment {
    Literal(String),
    EnvironmentVariable(String),
    // In the future:
    // RequestVariable(String), // req.{variable} etc.
}

#[derive(Debug, serde::Serialize, DeserializeFromStr)]
pub struct DynamicString(Vec<DynamicStringSegment>);

impl std::str::FromStr for DynamicString {
    type Err = ServerError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        fn re() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| Regex::new(r"\{\{\s*([[[:alnum:]]_.]+)\s*\}\}").unwrap())
        }

        let mut errors = vec![];
        let mut segments = vec![];
        let last_end = re().captures_iter(string).fold(0, |last_end, captures| {
            let overall_match = captures.get(0).unwrap();
            let key = captures.get(1).unwrap().as_str();
            let path = key.split('.');

            if let Some(("env", variable_name)) = path.collect_tuple() {
                if overall_match.start() > last_end {
                    segments.push(DynamicStringSegment::Literal(
                        string[last_end..overall_match.start()].to_string(),
                    ));
                }
                segments.push(DynamicStringSegment::EnvironmentVariable(variable_name.to_string()));
            } else {
                errors.push(format!(
                    "right now only variables scoped with 'env.' are supported: `{key}`"
                ));
            }
            overall_match.end()
        });

        if last_end != string.len() || string.is_empty() {
            segments.push(DynamicStringSegment::Literal(string[last_end..].to_string()));
        }

        if let Some(first_error) = errors.pop() {
            Err(ServerError::new(first_error, None))
        } else {
            Ok(DynamicString(segments))
        }
    }
}

impl DynamicString {
    pub fn partially_evaluate(&mut self, environment_variables: &HashMap<String, String>) -> Result<(), ServerError> {
        self.0 = self
            .0
            .drain(..)
            .map(|segment| match segment {
                DynamicStringSegment::EnvironmentVariable(variable_name) => environment_variables
                    .get(&variable_name)
                    .cloned()
                    .map(DynamicStringSegment::Literal)
                    .ok_or_else(|| ServerError::new(format!("undefined variable `{variable_name}`"), None)),
                other => Ok(other),
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .coalesce(|lhs, rhs| match (lhs, rhs) {
                (DynamicStringSegment::Literal(lhs), DynamicStringSegment::Literal(rhs)) => {
                    Ok(DynamicStringSegment::Literal(lhs + &rhs))
                }
                (lhs, rhs) => Err((lhs, rhs)),
            })
            .collect::<Vec<_>>();
        Ok(())
    }

    pub fn into_fully_evaluated_str(self) -> Option<String> {
        type UnaryArray = [DynamicStringSegment; 1];

        match UnaryArray::try_from(self.0).ok()? {
            [DynamicStringSegment::Literal(literal)] => Some(literal),
            _ => None,
        }
    }
}
