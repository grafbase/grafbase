use super::dot_graph::Attrs;

pub type Cost = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Edge {
    ///
    /// -- Resolver --
    ///
    /// From a ProvidableField or Root to a Resolver. Only incoming edge into Resolver
    CreateChildResolver(Cost),
    /// Incoming edge from a Resolver to a ProvidableField.
    CanProvide(Cost),

    ///
    /// -- Query --
    ///
    /// From a parent QueryField to a nested QueryField.
    Field,
    /// For a QueryField to a QueryField which is a __typename
    TypenameField,

    ///
    /// -- Resolver <-> Query --
    ///
    /// From a Field, the parent of a selection set, to a Resolver
    HasChildResolver,
    /// From a ProvidableField to a Field
    Provides,
    /// From a Field (@authorized directive), Resolver or ProvidableField (@requires) to a Field
    Requires,
}

impl Edge {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label(&self) -> String {
        match self {
            Edge::CreateChildResolver(cost) => if *cost > 0 {
                Attrs::new(format!("{cost}")).bold()
            } else {
                Attrs::new("")
            }
            .with("color=blue,fontcolor=blue"),
            Edge::CanProvide(cost) => if *cost > 0 {
                Attrs::new(format!("{cost}")).bold()
            } else {
                Attrs::new("")
            }
            .with("color=blue,fontcolor=blue"),
            Edge::Provides => Attrs::new("").with("color=turquoise"),
            Edge::Field => Attrs::new(""),
            Edge::TypenameField => Attrs::new("Typename"),
            Edge::Requires => Attrs::new("").with("color=orange"),
            Edge::HasChildResolver => Attrs::new("").with("style=dashed"),
        }
        .to_string()
    }
}
