use std::{borrow::Cow, fmt};

#[derive(Clone)]
pub struct CommentBlock<'a> {
    content: Cow<'a, str>,
}

impl<'a> CommentBlock<'a> {
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<'a, T> From<T> for CommentBlock<'a>
where
    T: Into<Cow<'a, str>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<'a> fmt::Display for CommentBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("/**\n")?;

        // split per line. for windows files, the line separator is \r\n,
        // so strip the \r if there
        for line in self.content.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            writeln!(f, " * {line}")?;
        }

        f.write_str(" */")?;

        Ok(())
    }
}
