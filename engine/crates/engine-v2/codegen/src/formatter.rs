use itertools::Itertools;
use regex::{Captures, Regex};

pub(super) struct Formatter {
    shell: xshell::Shell,
    doc_re: Regex,
}

impl Formatter {
    pub(super) fn new() -> anyhow::Result<Self> {
        Ok(Self {
            shell: xshell::Shell::new()?,
            doc_re: Regex::new(r#"(?m)^(?<spaces>\s*)#\[doc\s*=\s*"(?<doc>.*)"\]$"#)?,
        })
    }

    pub(super) fn format(&self, code: String) -> anyhow::Result<String> {
        use xshell::cmd;

        let code = cmd!(self.shell, "rustfmt")
            .stdin(code.clone())
            .read()
            .inspect_err(|_| {
                tracing::error!("Failed to format file:\n{code}");
            })?;
        // Transform 'serde :: Serialize' to 'serde::Serialize' and other similar cases.
        let code = code.replace(" :: ", "::");
        let code = self
            .doc_re
            .replace_all(&code, |caps: &Captures| {
                let spaces = caps.name("spaces").unwrap().as_str();
                let doc_indent = spaces.replace("\n", "");
                let lines = caps
                    .name("doc")
                    .unwrap()
                    .as_str()
                    .split(r"\n")
                    .map(|line| line.replace(r#"\""#, r#"""#))
                    .collect::<Vec<_>>();

                let comment_indent = lines
                    .iter()
                    .map(|line| {
                        line.char_indices()
                            .find(|(_, ch)| !ch.is_whitespace())
                            .map(|(i, _)| i)
                            .unwrap_or_else(|| line.len())
                    })
                    .min()
                    .unwrap_or(0);
                format!(
                    "{}{}",
                    caps.name("spaces").unwrap().as_str(),
                    lines.into_iter().format_with("\n", |line, f| f(&format_args!(
                        r"{doc_indent}/// {}",
                        &line[comment_indent..]
                    )))
                )
            })
            .to_string();

        Ok(cmd!(self.shell, "rustfmt")
            .stdin(code.clone())
            .read()
            .inspect_err(|_| {
                tracing::error!("Failed to format file:\n{code}");
            })?)
    }
}
