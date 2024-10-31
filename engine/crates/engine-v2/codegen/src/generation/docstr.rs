use crate::domain::Domain;
use std::fmt::Write;

pub fn generated_from(domain: &Domain, span: cynic_parser::Span, description: Option<&str>) -> String {
    let sdl = &domain.sdl[span.start..span.end];
    let mut doc = String::new();
    if let Some(description) = description {
        doc.push_str(description);
        doc.push_str("\n\n--------------\n");
    }
    write!(doc, "Generated from:\n\n```custom,{{.language-graphql}}\n{sdl}\n```").unwrap();
    doc
}
