use cynic_parser::Span;

use super::sdl;

pub(crate) struct Error {
    pub text: String,
    pub span: Span,
}

impl Error {
    pub fn into_string(self, translator: &sdl::SpanTranslator) -> String {
        let Self { text, span } = self;
        if span == Span::default() {
            return text;
        }
        let location = translator.span_to_location(span).unwrap();
        let sdl = &translator.sdl[span.start..self.span.end];
        format!("{text}. See schema at {location}:\n{sdl}")
    }

    pub fn without_span(err: impl ToString) -> Self {
        Self {
            text: err.to_string(),
            span: Span::default(),
        }
    }
}

impl From<(String, sdl::Span)> for Error {
    fn from((text, span): (String, sdl::Span)) -> Self {
        Error { text, span }
    }
}

impl From<(&'static str, sdl::Span)> for Error {
    fn from((text, span): (&'static str, sdl::Span)) -> Self {
        Error {
            text: text.to_string(),
            span,
        }
    }
}
