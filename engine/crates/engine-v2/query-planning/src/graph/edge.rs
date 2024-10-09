use super::dot_graph::Attrs;

pub type Cost = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Edge {
    Resolver(Cost),
    CanResolveField(Cost),
    Resolves,
    Field,
    TypenameField,
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
