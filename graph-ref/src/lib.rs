use std::{fmt, str};

/// Parsed graph reference. A graph reference is a string of the form `project@branch`.
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
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (graph, branch) = match s.split_once('@') {
            Some((graph, branch)) => (graph, Some(branch)),
            None => (s, None),
        };

        if graph.is_empty() {
            return Err("The graph name is missing.");
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
}
