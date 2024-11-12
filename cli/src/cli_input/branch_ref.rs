use std::{fmt, str};

/// Parsed graph reference. A graph reference is a string of the form `account/graph@branch`.
#[derive(Clone, Debug)]
pub struct BranchRef {
    account: String,
    graph: String,
    branch: String,
}

impl BranchRef {
    pub(crate) const ARG_DESCRIPTION: &'static str = r#"Branch reference following the format "account/graph@branch""#;

    pub(crate) fn account(&self) -> &str {
        self.account.as_ref()
    }

    pub(crate) fn graph(&self) -> &str {
        self.graph.as_ref()
    }

    pub(crate) fn branch(&self) -> &str {
        self.branch.as_ref()
    }
}

impl str::FromStr for BranchRef {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const GENERIC_ERR: &str =
            r#"Invalid branch reference. The branch reference argument must follow the format: "account/graph@branch""#;

        let Some((account, rest)) = s.split_once('/') else {
            return Err(GENERIC_ERR);
        };

        if account.is_empty() {
            return Err("The account name is missing before '/'.");
        }

        let Some((graph, branch)) = rest.split_once('@') else {
            return Err(GENERIC_ERR);
        };

        if graph.is_empty() {
            return Err("The graph name is missing.");
        }

        if branch.is_empty() {
            return Err("The branch name is missing.");
        }

        Ok(BranchRef {
            account: account.to_owned(),
            graph: graph.to_owned(),
            branch: branch.to_owned(),
        })
    }
}

impl fmt::Display for BranchRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.account)?;
        f.write_str("/")?;
        f.write_str(&self.graph)?;
        f.write_str("@")?;
        f.write_str(&self.branch)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_ref_ok() {
        let cases = [
            "microsoft/windows@main",
            "test/graph@master",
            "__my__/_____graph-with-things@branch-here",
            "1/2@3",
        ];

        for case in cases {
            assert_eq!(case, case.parse::<BranchRef>().unwrap().to_string());
        }
    }
}
