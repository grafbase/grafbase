use crate::dot_graph::Attrs;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr)]
pub(crate) enum SpaceEdge {
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
    TypenameField,

    ///
    /// -- Resolver <-> Query --
    ///
    /// From a Field, the parent of a selection set, to a Resolver
    HasChildResolver,
    /// From a ProvidableField to a Field
    Provides,
    ProvidesTypename,
    /// From a Field (@authorized directive), Resolver or ProvidableField (@requires) to a Field
    RequiredBySubgraph,
    RequiredBySupergraph,
}

impl SpaceEdge {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label(&self) -> Attrs<'static> {
        match self {
            SpaceEdge::CreateChildResolver => Attrs::default().with("color=royalblue,fontcolor=royalblue"),
            SpaceEdge::CanProvide => Attrs::default().with("color=royalblue,fontcolor=royalblue"),
            SpaceEdge::Provides | SpaceEdge::ProvidesTypename => Attrs::default().with("color=violet,arrowhead=none"),
            SpaceEdge::Field | SpaceEdge::TypenameField => Attrs::default(),
            SpaceEdge::RequiredBySubgraph => Attrs::default().with("color=orangered,arrowhead=inv"),
            SpaceEdge::RequiredBySupergraph => Attrs::default().with("color=orangered,arrowhead=inv,style=dashed"),
            SpaceEdge::HasChildResolver => Attrs::default().with("style=dashed,arrowhead=none"),
        }
    }
}
