use std::{
    fmt::{self, Write},
    ops::Deref,
    str::FromStr,
    sync::OnceLock,
};

use itertools::Itertools;
use regex::Regex;
use serde::Serialize;
use serde_with::DeserializeFromStr;

/// A wrapper type for Serde structures that can be (de-)serialized from a string.
/// If wrapping a type with this wrapper, one can pass values through env vars with the syntax
/// "{{ env.FOO }}".
#[derive(Debug, Serialize, DeserializeFromStr, Clone)]
pub struct DynamicString<T>(T)
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone;

impl<T> FromStr for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone,
{
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        /// Matches any "{{ something }}"
        fn re() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| Regex::new(r"\{\{\s*([[[:alnum:]]_.]+)\s*\}\}").expect("must be valid"))
        }

        let mut errors = Vec::new();

        // result is concatenated to one value of type T
        let mut result = T::default();

        let last_end = re().captures_iter(string).fold(0, |last_end, captures| {
            let overall_match = captures.get(0).unwrap();
            let key = captures.get(1).unwrap().as_str();
            let path = key.split('.');

            if let Some(("env", variable_name)) = path.collect_tuple() {
                // this is true if we have data between the current and the last match
                // e.g. `{{ env.FOO }} {{ env.BAR }}`
                //                    ^ we get this string
                if overall_match.start() > last_end {
                    match T::from_str(&string[last_end..overall_match.start()]) {
                        Ok(value) => result.write_str(value.as_ref()).expect("must succeed"),
                        Err(e) => errors.push(e.to_string()),
                    }
                }

                // fetches the value from the environment
                match std::env::var(variable_name) {
                    Ok(ref value) => match T::from_str(value) {
                        Ok(value) => result.write_str(value.as_ref()).expect("must succeed"),
                        Err(e) => errors.push(e.to_string()),
                    },
                    Err(e) => errors.push(format!("{e}: `{variable_name}`")),
                }
            } else {
                errors.push(format!(
                    "right now only variables scoped with 'env.' are supported: `{key}`"
                ));
            }

            overall_match.end()
        });

        if last_end != string.len() || string.is_empty() {
            match T::from_str(&string[last_end..]) {
                Ok(value) => result.write_str(value.as_ref()).expect("must succeed"),
                Err(e) => errors.push(e.to_string()),
            }
        }

        if let Some(first_error) = errors.pop() {
            Err(first_error)
        } else {
            Ok(DynamicString(result))
        }
    }
}

impl<T> AsRef<str> for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone,
{
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T> fmt::Display for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}

impl<T> PartialEq for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> From<T> for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone + PartialEq,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for DynamicString<T>
where
    T::Err: std::error::Error,
    T: FromStr + AsRef<str> + Default + Write + Clone + PartialEq,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use ascii::AsciiString;

    use super::DynamicString;

    #[test]
    fn simple_string_no_whitespace() {
        let result: DynamicString<String> = "foobar".parse().unwrap();
        assert_eq!("foobar", result.as_ref());
    }

    #[test]
    fn simple_string_with_whitespace() {
        let result: DynamicString<String> = "foobar foobar".parse().unwrap();
        assert_eq!("foobar foobar", result.as_ref());
    }

    #[test]
    fn single_env_var_not_set() {
        temp_env::with_var_unset("FOOBAR", || {
            let error = "{{ env.FOOBAR }}".parse::<DynamicString<String>>().unwrap_err();
            insta::assert_snapshot!(&error, @"environment variable not found: `FOOBAR`");
        });
    }

    #[test]
    fn single_env_var_set() {
        temp_env::with_var("FOOBAR", Some("some_value"), || {
            let result: DynamicString<String> = "{{ env.FOOBAR }}".parse().unwrap();
            assert_eq!("some_value", result.as_ref());
        });
    }

    #[test]
    fn single_env_var_set_twice() {
        temp_env::with_var("FOOBAR", Some("some_value"), || {
            let result: DynamicString<String> = "{{ env.FOOBAR }} {{ env.FOOBAR }}".parse().unwrap();
            assert_eq!("some_value some_value", result.as_ref());
        });
    }

    #[test]
    fn single_env_var_set_with_static_content_in_the_end() {
        temp_env::with_var("FOOBAR", Some("some_value"), || {
            let result: DynamicString<String> = "{{ env.FOOBAR }} static content".parse().unwrap();
            assert_eq!("some_value static content", result.as_ref());
        });
    }

    #[test]
    fn single_env_var_set_with_static_content_in_the_beginning() {
        temp_env::with_var("FOOBAR", Some("some_value"), || {
            let result: DynamicString<String> = "static content {{ env.FOOBAR }}".parse().unwrap();
            assert_eq!("static content some_value", result.as_ref());
        });
    }

    #[test]
    fn two_env_vars_set() {
        let vars = [("FOO", Some("foo")), ("BAR", Some("bar"))];

        temp_env::with_vars(vars, || {
            let result: DynamicString<String> = "{{ env.FOO }} {{ env.BAR }}".parse().unwrap();
            assert_eq!("foo bar", result.as_ref());
        });
    }

    #[test]
    fn two_env_vars_set_no_whitespace() {
        let vars = [("FOO", Some("foo")), ("BAR", Some("bar"))];

        temp_env::with_vars(vars, || {
            let result: DynamicString<String> = "{{ env.FOO }}{{ env.BAR }}".parse().unwrap();
            assert_eq!("foobar", result.as_ref());
        });
    }

    #[test]
    fn two_env_vars_one_not_set() {
        temp_env::with_var("FOO", Some("value"), || {
            let error = "{{ env.FOO }} {{ env.BAR}}"
                .parse::<DynamicString<String>>()
                .unwrap_err();

            insta::assert_snapshot!(&error, @"environment variable not found: `BAR`");
        });
    }

    #[test]
    fn ascii_static_value() {
        let result: DynamicString<AsciiString> = "foobar".parse().unwrap();
        assert_eq!("foobar", result.as_ref());
    }

    #[test]
    fn ascii_env_var_set() {
        temp_env::with_var("FOOBAR", Some("some_value"), || {
            let result: DynamicString<AsciiString> = "{{ env.FOOBAR }}".parse().unwrap();
            assert_eq!("some_value", result.as_ref());
        });
    }

    #[test]
    fn non_env_scope() {
        let error = "{{ meow.FOO }}".parse::<DynamicString<String>>().unwrap_err();

        insta::assert_snapshot!(&error, @"right now only variables scoped with 'env.' are supported: `meow.FOO`");
    }
}
