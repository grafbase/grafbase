use crate::domain::Domain;

pub fn generated_from(domain: &Domain, span: cynic_parser::Span) -> String {
    let sdl = &domain.sdl[span.start..span.end];
    format!("Generated from:\n\n```custom,{{.language-graphql}}\n{sdl}\n```")
}
