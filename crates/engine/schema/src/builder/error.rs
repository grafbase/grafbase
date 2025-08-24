use cynic_parser::Span;

use super::sdl;

pub(crate) struct Error {
    pub text: String,
    pub span: Option<Span>,
}

impl Error {
    pub fn new(text: impl std::fmt::Display) -> Self {
        Error {
            text: text.to_string(),
            span: None,
        }
    }

    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn display<'a>(self, translator: &'a sdl::SpanTranslator) -> ErrorDisplay<'a> {
        ErrorDisplay {
            error: self,
            translator,
        }
    }

    pub fn with_prefix(self, mut prefix: String) -> Self {
        prefix.push_str(&self.text);
        Error { text: prefix, ..self }
    }

    pub fn span_if_absent(self, span: Span) -> Self {
        Error {
            span: self.span.or(Some(span)),
            ..self
        }
    }
}

pub(crate) struct ErrorDisplay<'a> {
    error: Error,
    translator: &'a sdl::SpanTranslator<'a>,
}

impl<'a> std::fmt::Display for ErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Error { text, span } = &self.error;

        // If no span or default span, just write the text
        let Some(span) = span.filter(|span| *span != Span::default()) else {
            return writeln!(f, "{}", text);
        };

        writeln!(f, "* {}", text)?;
        writeln!(f, "{}", self.translator.display_span(span))
    }
}

impl From<String> for Error {
    fn from(text: String) -> Self {
        Error { text, span: None }
    }
}

impl From<&'static str> for Error {
    fn from(text: &'static str) -> Self {
        Error {
            text: text.to_string(),
            span: None,
        }
    }
}

impl From<(String, sdl::Span)> for Error {
    fn from((text, span): (String, sdl::Span)) -> Self {
        Error { text, span: Some(span) }
    }
}

impl From<(&'static str, sdl::Span)> for Error {
    fn from((text, span): (&'static str, sdl::Span)) -> Self {
        Error {
            text: text.to_string(),
            span: Some(span),
        }
    }
}
