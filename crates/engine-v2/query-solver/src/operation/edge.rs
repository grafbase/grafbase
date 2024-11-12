use crate::dot_graph::Attrs;

use super::{Operation, OperationGraph};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr)]
pub(crate) enum Edge {
    ///
    /// -- Resolver --
    ///
    /// From a ProvidableField or Root to a Resolver. Only incoming edge into Resolver
    CreateChildResolver,
    /// Incoming edge from a Resolver to a ProvidableField.
    CanProvide,

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
    pub(crate) fn pretty_label<Op: Operation>(&self, _graph: &OperationGraph<'_, Op>) -> String {
        match self {
            Edge::CreateChildResolver => Attrs::default().with("color=royalblue,fontcolor=royalblue"),
            Edge::CanProvide => Attrs::default().with("color=royalblue,fontcolor=royalblue"),
            Edge::Provides => Attrs::default().with("color=violet,arrowhead=none"),
            Edge::Field => Attrs::default(),
            Edge::TypenameField => Attrs::label("Typename"),
            Edge::Requires => Attrs::default().with("color=orangered,arrowhead=inv"),
            Edge::HasChildResolver => Attrs::default().with("style=dashed,arrowhead=none"),
        }
        .to_string()
    }
}
