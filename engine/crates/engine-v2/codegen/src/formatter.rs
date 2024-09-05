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
            doc_re: Regex::new(r#"(?<spaces>\s+)#\[doc\s*=\s*"(?<doc>.*)"\]"#)?,
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
                caps.name("doc")
                    .unwrap()
                    .as_str()
                    .split(r"\n")
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .format_with("", |s, f| f(&format_args!(r"{spaces}/// {s}")))
                    .to_string()
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
