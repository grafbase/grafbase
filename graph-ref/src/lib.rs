use std::{borrow::Cow, fmt, str};

/// Parsed graph reference. A graph reference is a string of the form `graph@branch`.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct GraphRef {
    graph_slug: String,
    branch_name: Option<String>,
}

impl GraphRef {
    pub const ARG_DESCRIPTION: &'static str = r#"Graph reference following the format "graph@branch""#;

    pub fn new(graph_slug: String, branch_name: Option<String>) -> Self {
        GraphRef {
            graph_slug,
            branch_name,
        }
    }

    pub fn graph_slug(&self) -> &str {
        self.graph_slug.as_ref()
    }

    pub fn branch_name(&self) -> Option<&str> {
        self.branch_name.as_deref()
    }
}

impl str::FromStr for GraphRef {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (graph_slug, branch_name) = match s.split_once('@') {
            Some((graph_slug, branch_name)) => (graph_slug, Some(branch_name)),
            None => (s, None),
        };

        if graph_slug.is_empty() {
            return Err(Cow::Borrowed("The graph name is missing."));
        }

        if graph_slug.contains('/') {
            let did_you_mean = 'split: {
                let Some((_, graph_slug)) = graph_slug.split_once('/') else {
                    break 'split String::new();
                };

                if graph_slug.is_empty() {
                    break 'split String::new();
                }

                let branch_name = branch_name.map(|branch| format!("@{branch}")).unwrap_or_default();

                format!(" Did you mean: \"{graph_slug}{branch_name}\"")
            };

            let message = format!("Graph ref should not contain an account name.{did_you_mean}",);

            return Err(Cow::Owned(message));
        }

        Ok(GraphRef {
            graph_slug: graph_slug.to_owned(),
            branch_name: branch_name.filter(|s| !s.is_empty()).map(String::from),
        })
    }
}

impl fmt::Display for GraphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.graph_slug)?;

        if let Some(branch) = &self.branch_name {
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
