use cynic_parser::Span;

use super::sdl;

pub(crate) struct Error {
    pub text: String,
    pub span: Option<Span>,
}

impl Error {
    pub fn into_string(self, translator: &sdl::SpanTranslator) -> String {
        let Self { text, span } = self;
        let Some(span) = span.filter(|span| *span != Span::default()) else {
            return text;
        };
        let location = translator.span_to_location(span).unwrap();
        let sdl = &translator.sdl[span.start..span.end];
        format!("{text}\nSee schema at {location}:\n{sdl}")
    }

    pub fn with_prefix(self, mut prefix: String) -> Self {
        prefix.push_str(&self.text);
        Error { text: prefix, ..self }
    }

    pub fn with_span_if_absent(self, span: Span) -> Self {
        Error {
            span: self.span.or(Some(span)),
            ..self
        }
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
