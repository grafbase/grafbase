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
    /// From a Field (@authorized directive), Resolver or ProvidableField (@requires) to a Field
    Requires,
}

impl SpaceEdge {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label(&self) -> Attrs<'static> {
        // let label: &'static str = self.into();
        let label = "";
        match self {
            SpaceEdge::CreateChildResolver => Attrs::label(label).with("color=royalblue,fontcolor=royalblue"),
            SpaceEdge::CanProvide => Attrs::label(label).with("color=royalblue,fontcolor=royalblue"),
            SpaceEdge::Provides => Attrs::label(label).with("color=violet,arrowhead=none"),
            SpaceEdge::Field | SpaceEdge::TypenameField => Attrs::label(label),
            SpaceEdge::Requires => Attrs::label(label).with("color=orangered,arrowhead=inv"),
            SpaceEdge::HasChildResolver => Attrs::label(label).with("style=dashed,arrowhead=none"),
        }
    }
}
