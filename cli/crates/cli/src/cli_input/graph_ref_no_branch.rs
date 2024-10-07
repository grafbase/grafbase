use std::{fmt, str};

/// Parsed graph reference. A project reference is a string of the form `account/graph`.
#[derive(Clone, Debug)]
pub struct GraphRefNoBranch {
    account: String,
    graph: String,
}

impl str::FromStr for GraphRefNoBranch {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const GENERIC_ERR: &str =
            r#"Invalid graph reference. The graph reference argument must follow the format: "account/graph""#;

        let Some((account, graph)) = s.split_once('/') else {
            return Err(GENERIC_ERR);
        };

        if account.is_empty() {
            return Err("The account name is missing before '/'.");
        }

        if graph.is_empty() {
            return Err("The graph name is missing.");
        }

        if graph.split_once('@').is_some() {
            return Err("The graph ref should not define branch.");
        }

        Ok(GraphRefNoBranch {
            account: account.to_owned(),
            graph: graph.to_owned(),
        })
    }
}

impl fmt::Display for GraphRefNoBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.account)?;
        f.write_str("/")?;
        f.write_str(&self.graph)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_ref_no_branch_ok() {
        let cases = [
            "microsoft/windows",
            "test/project",
            "__my__/_____project-with-things",
            "1/2",
        ];

        for case in cases {
            assert_eq!(case, case.parse::<GraphRefNoBranch>().unwrap().to_string());
        }
    }

    #[test]
    fn graph_ref_with_branch() {
        let err = "foo/bar@lol".parse::<GraphRefNoBranch>().unwrap_err();
        assert_eq!("The graph ref should not define branch.", err);
    }
}
