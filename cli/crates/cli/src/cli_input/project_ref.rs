use std::{borrow::Cow, fmt, str};

use graph_ref::GraphRef;

/// Parsed project reference. A project reference is a string of the form `account/project@branch`.
#[derive(Clone, Debug)]
pub struct ProjectRef {
    account: String,
    graph: String,
    branch: Option<String>,
}

impl ProjectRef {
    pub(crate) const ARG_DESCRIPTION: &'static str = r#"Graph reference following the format "account/graph@branch""#;

    pub(crate) fn account(&self) -> &str {
        self.account.as_ref()
    }

    pub(crate) fn graph(&self) -> &str {
        self.graph.as_ref()
    }

    pub(crate) fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub(crate) fn into_parts(self) -> (String, String, Option<String>) {
        (self.account, self.graph, self.branch)
    }
}

impl str::FromStr for ProjectRef {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const GENERIC_ERR: &str =
            r#"Invalid graph reference. The graph reference argument must follow the format: "account/graph@branch""#;

        let Some((account, rest)) = s.split_once('/') else {
            return Err(GENERIC_ERR);
        };

        if account.is_empty() {
            return Err("The account name is missing before '/'.");
        }

        let (project, branch) = match rest.split_once('@') {
            Some((project, branch)) => (project, Some(branch)),
            None => (rest, None),
        };

        if project.is_empty() {
            return Err("The project name is missing.");
        }

        Ok(ProjectRef {
            account: account.to_owned(),
            graph: project.to_owned(),
            branch: branch.filter(|s| !s.is_empty()).map(String::from),
        })
    }
}

impl fmt::Display for ProjectRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.account)?;
        f.write_str("/")?;
        f.write_str(&self.graph)?;

        if let Some(branch) = &self.branch {
            f.write_str("@")?;
            f.write_str(branch)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ProjectRefOrGraphRef {
    ProjectRef(ProjectRef),
    GraphRef(GraphRef),
}

impl ProjectRefOrGraphRef {
    pub(crate) fn branch(&self) -> Option<&str> {
        match self {
            ProjectRefOrGraphRef::ProjectRef(pr) => pr.branch(),
            ProjectRefOrGraphRef::GraphRef(gr) => gr.branch(),
        }
    }

    pub(crate) fn account(&self) -> Option<&str> {
        match self {
            ProjectRefOrGraphRef::ProjectRef(pr) => Some(pr.account()),
            ProjectRefOrGraphRef::GraphRef(_) => None,
        }
    }

    pub(crate) fn project(&self) -> &str {
        match self {
            ProjectRefOrGraphRef::ProjectRef(pr) => pr.graph(),
            ProjectRefOrGraphRef::GraphRef(gr) => gr.graph(),
        }
    }
}

impl str::FromStr for ProjectRefOrGraphRef {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ProjectRef::from_str(s)
            .map(ProjectRefOrGraphRef::ProjectRef)
            .map_err(Cow::Borrowed)
            .or_else(|_| GraphRef::from_str(s).map(ProjectRefOrGraphRef::GraphRef))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_ref_ok() {
        let cases = [
            "microsoft/windows@main",
            "test/project@master",
            "__my__/_____project-with-things@branch-here",
            "1/2@3",
            "accnt/prjct", // no branch
        ];

        for case in cases {
            assert_eq!(case, case.parse::<ProjectRef>().unwrap().to_string());
        }
    }

    #[test]
    fn project_ref_or_graph_ref() {
        assert!(matches!(
            "microsoft/windows@main".parse(),
            Ok(ProjectRefOrGraphRef::ProjectRef(_))
        ));
        assert!(matches!(
            "test/project@master".parse(),
            Ok(ProjectRefOrGraphRef::ProjectRef(_))
        ));
        assert!(matches!(
            "__my__/_____project-with-things@branch-here".parse(),
            Ok(ProjectRefOrGraphRef::ProjectRef(_))
        ));
        assert!(matches!("1/2@3".parse(), Ok(ProjectRefOrGraphRef::ProjectRef(_))));
        assert!(matches!("accnt/prjct".parse(), Ok(ProjectRefOrGraphRef::ProjectRef(_))));

        assert!(matches!("windows@main".parse(), Ok(ProjectRefOrGraphRef::GraphRef(_))));
        assert!(matches!(
            "project@master".parse(),
            Ok(ProjectRefOrGraphRef::GraphRef(_))
        ));
        assert!(matches!(
            "_____project-with-things@branch-here".parse(),
            Ok(ProjectRefOrGraphRef::GraphRef(_))
        ));
        assert!(matches!("2@3".parse(), Ok(ProjectRefOrGraphRef::GraphRef(_))));
        assert!(matches!("prjct".parse(), Ok(ProjectRefOrGraphRef::GraphRef(_))));
    }
}
