use std::{borrow::Cow, fmt, str};

use graph_ref::GraphRef;
/// Parsed graph reference. A graph reference is a string of the form `account/project@branch`.
#[derive(Clone, Debug)]
pub struct FullGraphRef {
    account: String,
    graph: String,
    branch: Option<String>,
}

impl FullGraphRef {
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
}

impl str::FromStr for FullGraphRef {
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

        let (graph, branch) = match rest.split_once('@') {
            Some((project, branch)) => (project, Some(branch)),
            None => (rest, None),
        };

        if graph.is_empty() {
            return Err("The graph name is missing.");
        }

        Ok(FullGraphRef {
            account: account.to_owned(),
            graph: graph.to_owned(),
            branch: branch.filter(|s| !s.is_empty()).map(String::from),
        })
    }
}

impl fmt::Display for FullGraphRef {
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
pub(crate) enum FullOrPartialGraphRef {
    Full(FullGraphRef),
    Partial(GraphRef),
}

impl FullOrPartialGraphRef {
    pub(crate) fn branch(&self) -> Option<&str> {
        match self {
            FullOrPartialGraphRef::Full(pr) => pr.branch(),
            FullOrPartialGraphRef::Partial(gr) => gr.branch(),
        }
    }

    pub(crate) fn account(&self) -> Option<&str> {
        match self {
            FullOrPartialGraphRef::Full(pr) => Some(pr.account()),
            FullOrPartialGraphRef::Partial(_) => None,
        }
    }

    pub(crate) fn graph(&self) -> &str {
        match self {
            FullOrPartialGraphRef::Full(pr) => pr.graph(),
            FullOrPartialGraphRef::Partial(gr) => gr.slug(),
        }
    }
}

impl str::FromStr for FullOrPartialGraphRef {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FullGraphRef::from_str(s)
            .map(FullOrPartialGraphRef::Full)
            .map_err(Cow::Borrowed)
            .or_else(|_| GraphRef::from_str(s).map(FullOrPartialGraphRef::Partial))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_graph_ref_ok() {
        let cases = [
            "microsoft/windows@main",
            "test/project@master",
            "__my__/_____project-with-things@branch-here",
            "1/2@3",
            "accnt/prjct", // no branch
        ];

        for case in cases {
            assert_eq!(case, case.parse::<FullGraphRef>().unwrap().to_string());
        }
    }

    #[test]
    fn full_or_partial_graph_ref() {
        assert!(matches!(
            "microsoft/windows@main".parse(),
            Ok(FullOrPartialGraphRef::Full(_))
        ));
        assert!(matches!(
            "test/project@master".parse(),
            Ok(FullOrPartialGraphRef::Full(_))
        ));
        assert!(matches!(
            "__my__/_____project-with-things@branch-here".parse(),
            Ok(FullOrPartialGraphRef::Full(_))
        ));
        assert!(matches!("1/2@3".parse(), Ok(FullOrPartialGraphRef::Full(_))));
        assert!(matches!("accnt/prjct".parse(), Ok(FullOrPartialGraphRef::Full(_))));

        assert!(matches!("windows@main".parse(), Ok(FullOrPartialGraphRef::Partial(_))));
        assert!(matches!(
            "project@master".parse(),
            Ok(FullOrPartialGraphRef::Partial(_))
        ));
        assert!(matches!(
            "_____project-with-things@branch-here".parse(),
            Ok(FullOrPartialGraphRef::Partial(_))
        ));
        assert!(matches!("2@3".parse(), Ok(FullOrPartialGraphRef::Partial(_))));
        assert!(matches!("prjct".parse(), Ok(FullOrPartialGraphRef::Partial(_))));
    }
}
