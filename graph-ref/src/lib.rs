use std::{borrow::Cow, fmt, str};

/// Parsed graph reference. A graph reference is a string of the form `graph@branch#version`.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum GraphRef {
    LatestProductionVersion {
        graph_slug: String,
    },
    LatestVersion {
        graph_slug: String,
        branch_name: String,
    },
    Id {
        graph_slug: String,
        branch_name: String,
        version: String,
    },
}

impl GraphRef {
    pub const ARG_DESCRIPTION: &'static str = r#"Graph reference following the format "graph@branch""#;

    pub fn new(graph_slug: String, branch_name: String, version: String) -> Self {
        GraphRef::Id {
            graph_slug,
            branch_name,
            version,
        }
    }

    pub fn latest_production_version(graph_slug: String) -> Self {
        GraphRef::LatestProductionVersion { graph_slug }
    }

    pub fn lastest_version(graph_slug: String, branch_name: String) -> Self {
        GraphRef::LatestVersion {
            graph_slug,
            branch_name,
        }
    }

    pub fn slug(&self) -> &str {
        match self {
            GraphRef::LatestProductionVersion { graph_slug } => graph_slug,
            GraphRef::LatestVersion { graph_slug, .. } => graph_slug,
            GraphRef::Id { graph_slug, .. } => graph_slug,
        }
    }

    pub fn branch(&self) -> Option<&str> {
        match self {
            GraphRef::LatestProductionVersion { .. } => None,
            GraphRef::LatestVersion { branch_name, .. } => Some(branch_name),
            GraphRef::Id { branch_name, .. } => Some(branch_name),
        }
    }

    pub fn version(&self) -> Option<&str> {
        match self {
            GraphRef::LatestProductionVersion { .. } => None,
            GraphRef::LatestVersion { .. } => None,
            GraphRef::Id { version, .. } => Some(version),
        }
    }
}

impl str::FromStr for GraphRef {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let graph = match s.split_once('@') {
            Some((graph_slug, rest)) => match rest.split_once('#') {
                Some((branch_name, version)) => GraphRef::Id {
                    graph_slug: graph_slug.to_string(),
                    branch_name: branch_name.to_string(),
                    version: version.to_string(),
                },
                None => GraphRef::LatestVersion {
                    graph_slug: graph_slug.to_string(),
                    branch_name: rest.to_string(),
                },
            },
            None => GraphRef::LatestProductionVersion {
                graph_slug: s.to_string(),
            },
        };

        if graph.slug().is_empty() {
            return Err(Cow::Borrowed("The graph name is missing."));
        }

        if graph.slug().contains('/') {
            let did_you_mean = 'split: {
                let Some((_, graph_slug)) = graph.slug().split_once('/') else {
                    break 'split String::new();
                };

                if graph_slug.is_empty() {
                    break 'split String::new();
                }

                let branch_name = graph.branch().map(|branch| format!("@{branch}")).unwrap_or_default();
                let version = graph.version().map(|version| format!("#{version}")).unwrap_or_default();

                format!(" Did you mean: \"{graph_slug}{branch_name}{version}\"")
            };

            let message = format!("Graph ref should not contain an account name.{did_you_mean}",);

            return Err(Cow::Owned(message));
        }

        Ok(graph)
    }
}

impl fmt::Display for GraphRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())?;

        if let Some(branch) = &self.branch() {
            f.write_str("@")?;
            f.write_str(branch)?;
        }

        if let Some(version) = &self.version() {
            f.write_str("#")?;
            f.write_str(version)?;
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
            "prjct",        // no branch
            "prjct@branch", // no version
            "prjct@branch#version",
            "windows@main",
            "project@master",
            "_____project-with-things@branch-here",
            "2@3",
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
