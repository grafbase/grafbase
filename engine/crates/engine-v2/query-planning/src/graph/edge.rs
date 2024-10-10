use super::dot_graph::Attrs;

pub type Cost = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Edge {
    /// Incoming edge to a resolver
    Resolver(Cost),
    /// Incoming edge from a resolver/field resolver to a nested field resolver. The field resolver
    /// holds extra metadata relevant to the field.
    CanResolveField(Cost),
    /// Outgoing edge from a field resolver to a field
    Resolves,
    /// From a parent field to a nested field.
    Field,
    /// for a parent field to a __typename field
    TypenameField,
    /// From a field (directives), resolver or field resolver to a required field
    Requires,
}

impl Edge {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label(&self) -> String {
        match self {
            Edge::Resolver(cost) => if *cost > 0 {
                Attrs::new(format!("Resolver:{cost}")).bold()
            } else {
                Attrs::new("")
            }
            .with("color=blue,fontcolor=blue"),
            Edge::CanResolveField(cost) => if *cost > 0 {
                Attrs::new(format!("CanResolve:{cost}")).bold()
            } else {
                Attrs::new("")
            }
            .with("color=blue,fontcolor=blue"),
            Edge::Resolves => Attrs::new("").with("color=turquoise"),
            Edge::Field => Attrs::new(""),
            Edge::TypenameField => Attrs::new("Typename"),
            Edge::Requires => Attrs::new("").with("color=orange"),
        }
        .to_string()
    }
}
