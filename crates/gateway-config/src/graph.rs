use serde::de;

#[derive(Debug, Clone)]
pub struct SchemaFetchUrl {
    template: String,
    graph_idx: Option<(usize, usize)>,
    branch_idx: Option<(usize, usize)>,
}

impl SchemaFetchUrl {
    pub fn render(&self, graph: &str, branch: Option<&str>) -> Result<String, String> {
        let mut url = self.template.clone();

        let mut substitutions = [
            ("graph-ref.graph", Some(graph), self.graph_idx),
            ("graph-ref.branch", branch, self.branch_idx),
        ];

        substitutions.sort_unstable_by_key(|(_, _, idx)| idx.map(|(start, _)| start));

        // How much the indexes have shifted due to replacements so far.
        let mut offset = 0i32;

        for (name, value, idx) in substitutions {
            match (value, idx) {
                (_, None) => (),
                (None, Some(_)) => {
                    return Err(format!(
                        "Expected a value for {name} to construct the schema fetch URL, but it is not available."
                    ));
                }
                (Some(value), Some((start, end))) => {
                    let new_offset = value.len() as i32 - (end as i32 - start as i32);
                    let [start, end] = [start, end].map(|idx| (idx as i32 + offset) as usize);
                    url.replace_range(start..end, value);
                    offset += new_offset;
                }
            }
        }

        Ok(url)
    }
}

impl<'de> de::Deserialize<'de> for SchemaFetchUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct V;

        impl de::Visitor<'_> for V {
            type Value = SchemaFetchUrl;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a template string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let [mut graph_idx, mut branch_idx] = [None; 2];

                for (start, end) in serde_dynamic_string::iter_variables(v) {
                    let variable = v[start..end].trim_start_matches('{').trim_end_matches('}').trim();

                    match variable {
                        "graph-ref.graph" => graph_idx = Some((start, end)),
                        "graph-ref.branch" => branch_idx = Some((start, end)),
                        other => {
                            return Err(de::Error::unknown_field(
                                other,
                                &["graph-ref.graph", "graph-ref.branch", "access-token.account_id"],
                            ));
                        }
                    }
                }

                Ok(SchemaFetchUrl {
                    template: v.to_owned(),
                    graph_idx,
                    branch_idx,
                })
            }
        }

        deserializer.deserialize_str(V)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    fn schema_fetch_url_from_str(s: &str) -> Result<SchemaFetchUrl, toml::de::Error> {
        #[derive(Deserialize)]
        struct SchemaFetch {
            url: SchemaFetchUrl,
        }

        let config: SchemaFetch = toml::from_str(s)?;

        Ok(config.url)
    }

    #[test]
    fn test_deserialize_with_all_variables() {
        let toml = "url = \"https://api.example.com/graph/{{graph-ref.graph}}/branch/{{graph-ref.branch}}/schema\"";
        let url = schema_fetch_url_from_str(toml).unwrap();

        insta::assert_debug_snapshot!(&url, @r#"
        SchemaFetchUrl {
            template: "https://api.example.com/graph/{{graph-ref.graph}}/branch/{{graph-ref.branch}}/schema",
            graph_idx: Some(
                (
                    30,
                    49,
                ),
            ),
            branch_idx: Some(
                (
                    57,
                    77,
                ),
            ),
        }
        "#);
    }

    #[test]
    fn test_deserialize_with_partial_variables() {
        let toml = "url = \"https://api.example.com/graph/{{graph-ref.graph}}/schema\"";
        let url = schema_fetch_url_from_str(toml).unwrap();

        insta::assert_debug_snapshot!(&url, @r#"
        SchemaFetchUrl {
            template: "https://api.example.com/graph/{{graph-ref.graph}}/schema",
            graph_idx: Some(
                (
                    30,
                    49,
                ),
            ),
            branch_idx: None,
        }
        "#);
    }

    #[test]
    fn test_deserialize_with_no_variables() {
        let toml = "url = \"https://api.example.com/schema\"";
        let url = schema_fetch_url_from_str(toml).unwrap();

        insta::assert_debug_snapshot!(&url, @r#"
        SchemaFetchUrl {
            template: "https://api.example.com/schema",
            graph_idx: None,
            branch_idx: None,
        }
        "#);
    }

    #[test]
    fn test_deserialize_with_invalid_variable() {
        let toml = "url = \"https://api.example.com/graph/{{invalid-variable}}/schema\"";
        let result = schema_fetch_url_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_with_all_values() {
        let toml = "url = \"https://api.example.com/graph/{{graph-ref.graph}}/branch/{{graph-ref.branch}}/schema\"";
        let url = schema_fetch_url_from_str(toml).unwrap();

        let result = url.render("my-graph", Some("main")).unwrap();
        assert_eq!(result, "https://api.example.com/graph/my-graph/branch/main/schema");
    }

    #[test]
    fn test_render_with_partial_values() {
        let toml = "url = \"https://api.example.com/graph/{{graph-ref.graph}}/schema\"";

        let url = schema_fetch_url_from_str(toml).unwrap();

        let result = url.render("my-graph", None).unwrap();
        assert_eq!(result, "https://api.example.com/graph/my-graph/schema");
    }

    #[test]
    fn test_render_with_missing_required_value() {
        let toml = "url = \"https://api.example.com/graph/{{graph-ref.branch}}/schema\"";
        let url = schema_fetch_url_from_str(toml).unwrap();

        let result = url.render("test", None);
        assert!(result.is_err());
    }
}
