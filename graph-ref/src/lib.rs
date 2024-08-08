use std::{borrow::Cow, fmt, str};

/// Parsed graph reference. A graph reference is a string of the form `graph@branch`.
#[derive(Clone, Debug)]
pub struct GraphRef {
    graph: String,
    branch: Option<String>,
}

impl GraphRef {
    pub const ARG_DESCRIPTION: &'static str = r#"Graph reference following the format "graph@branch""#;

    pub fn graph(&self) -> &str {
        self.graph.as_ref()
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

impl str::FromStr for GraphRef {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (graph, branch) = match s.split_once('@') {
            Some((graph, branch)) => (graph, Some(branch)),
            None => (s, None),
        };

        if graph.is_empty() {
            return Err(Cow::Borrowed("The graph name is missing."));
        }

        if graph.contains('/') {
            let did_you_mean = 'split: {
                let Some((_, graph_name)) = graph.split_once('/') else {
                    break 'split String::new();
                };

                if graph_name.is_empty() {
                    break 'split String::new();
                }

                let branch = branch.map(|branch| format!("@{branch}")).unwrap_or_default();

                format!(" Did you mean: \"{graph_name}{branch}\"")
            };

            let message = format!("Graph ref should not contain an account name.{did_you_mean}",);

            return Err(Cow::Owned(message));
        }

        Ok(GraphRef {
            graph: graph.to_owned(),
            branch: branch.filter(|s| !s.is_empty()).map(String::from),
        })
    }
}

impl fmt::Display for GraphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.graph)?;

        if let Some(branch) = &self.branch {
            f.write_str("@")?;
            f.write_str(branch)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_ref_ok() {
        let cases = [
            "windows@main",
            "project@master",
            "_____project-with-things@branch-here",
            "2@3",
            "prjct", // no branch
        ];

        for case in cases {
            assert_eq!(case, case.parse::<GraphRef>().unwrap().to_string());
        }
    }

    #[test]
    fn account_name_not_allowed() {
        let err = "cow/bell@main".parse::<GraphRef>().unwrap_err();

        assert_eq!(
            err,
            "Graph ref should not contain an account name. Did you mean: \"bell@main\""
        )
    }
}
