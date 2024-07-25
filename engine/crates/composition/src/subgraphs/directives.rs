use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DirectiveSiteId(usize);

type Arguments = Vec<(StringId, Value)>;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub(crate) enum Value {
    String(StringId),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Enum(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

#[derive(Default)]
pub(super) struct Directives {
    site_id_counter: usize,

    deprecated: BTreeMap<DirectiveSiteId, Deprecated>,
    r#override: BTreeMap<DirectiveSiteId, OverrideDirective>,
    provides: BTreeMap<DirectiveSiteId, Vec<Selection>>,
    requires: BTreeMap<DirectiveSiteId, Vec<Selection>>,
    authorized: BTreeMap<DirectiveSiteId, AuthorizedDirective>,

    requires_scopes: BTreeSet<(DirectiveSiteId, Vec<StringId>)>,
    policies: BTreeSet<(DirectiveSiteId, Vec<StringId>)>,

    authenticated: HashSet<DirectiveSiteId>,
    inaccessible: HashSet<DirectiveSiteId>,
    shareable: HashSet<DirectiveSiteId>,
    external: HashSet<DirectiveSiteId>,
    interface_object: HashSet<DirectiveSiteId>,

    tags: BTreeSet<(DirectiveSiteId, StringId)>,

    /// From @composeDirective.
    ///
    /// (subgraph_id, directive_name)
    composed_directives: BTreeSet<(SubgraphId, StringId)>,

    composed_directive_instances: Vec<(DirectiveSiteId, StringId, Arguments)>,
}

impl Subgraphs {
    pub(crate) fn insert_authenticated(&mut self, id: DirectiveSiteId) {
        self.directives.authenticated.insert(id);
    }

    pub(crate) fn insert_authorized(&mut self, id: DirectiveSiteId, directive: AuthorizedDirective) {
        self.directives.authorized.insert(id, directive);
    }

    pub(crate) fn insert_composed_directive(&mut self, subgraph_id: SubgraphId, directive_name: &str) {
        let directive_name = self.strings.intern(directive_name);
        self.directives
            .composed_directives
            .insert((subgraph_id, directive_name));
    }

    pub(crate) fn insert_composed_directive_instance(
        &mut self,
        id: DirectiveSiteId,
        directive_name: &str,
        arguments: Arguments,
    ) {
        let directive_name = self.strings.intern(directive_name);
        self.directives
            .composed_directive_instances
            .push((id, directive_name, arguments));
    }

    pub(crate) fn insert_deprecated(&mut self, id: DirectiveSiteId, reason: Option<&str>) {
        let reason = reason.map(|reason| self.strings.intern(reason));
        self.directives.deprecated.insert(id, Deprecated { reason });
    }

    pub(crate) fn insert_provides(&mut self, id: DirectiveSiteId, fields: &str) -> Result<(), String> {
        let fields = self.selection_set_from_str(fields)?;
        self.directives.provides.insert(id, fields);
        Ok(())
    }

    pub(crate) fn insert_requires(&mut self, id: DirectiveSiteId, fields: &str) -> Result<(), String> {
        let fields = self.selection_set_from_str(fields)?;
        self.directives.requires.insert(id, fields);
        Ok(())
    }

    pub(crate) fn insert_policy(&mut self, id: DirectiveSiteId, policies: Vec<StringId>) {
        self.directives.policies.insert((id, policies));
    }

    pub(crate) fn append_required_scopes(&mut self, id: DirectiveSiteId, scopes: Vec<StringId>) {
        self.directives.requires_scopes.insert((id, scopes));
    }

    pub(crate) fn insert_tag(&mut self, id: DirectiveSiteId, tag: &str) {
        let tag = self.strings.intern(tag);
        self.directives.tags.insert((id, tag));
    }

    pub(crate) fn new_directive_site(&mut self) -> DirectiveSiteId {
        let id = DirectiveSiteId(self.directives.site_id_counter);
        self.directives.site_id_counter += 1;
        id
    }

    pub(crate) fn set_external(&mut self, id: DirectiveSiteId) {
        self.directives.external.insert(id);
    }

    pub(crate) fn set_inaccessible(&mut self, id: DirectiveSiteId) {
        self.directives.inaccessible.insert(id);
    }

    pub(crate) fn set_interface_object(&mut self, id: DirectiveSiteId) {
        self.directives.interface_object.insert(id);
    }

    pub(crate) fn set_override(&mut self, id: DirectiveSiteId, directive: OverrideDirective) {
        self.directives.r#override.insert(id, directive);
    }

    pub(crate) fn set_shareable(&mut self, id: DirectiveSiteId) {
        self.directives.shareable.insert(id);
    }
}

pub(crate) type DirectiveSiteWalker<'a> = Walker<'a, DirectiveSiteId>;

impl<'a> DirectiveSiteWalker<'a> {
    pub(crate) fn authenticated(self) -> bool {
        self.subgraphs.directives.authenticated.contains(&self.id)
    }

    pub(crate) fn authorized(self) -> Option<&'a AuthorizedDirective> {
        self.subgraphs.directives.authorized.get(&self.id)
    }

    pub(crate) fn deprecated(self) -> Option<DeprecatedWalker<'a>> {
        self.subgraphs
            .directives
            .deprecated
            .get(&self.id)
            .map(|deprecated| self.walk(deprecated))
    }

    pub(crate) fn external(self) -> bool {
        self.subgraphs.directives.external.contains(&self.id)
    }

    pub(crate) fn inaccessible(self) -> bool {
        self.subgraphs.directives.inaccessible.contains(&self.id)
    }

    pub(crate) fn interface_object(self) -> bool {
        self.subgraphs.directives.interface_object.contains(&self.id)
    }

    pub(crate) fn iter_composed_directives(&self) -> impl Iterator<Item = (StringId, &Arguments)> {
        let instances = &self.subgraphs.directives.composed_directive_instances;
        let partition_point = instances.partition_point(|(id, _, _)| id < &self.id);
        instances[partition_point..]
            .iter()
            .take_while(|(id, _, _)| id == &self.id)
            .map(|(_, name, args)| (*name, args))
    }

    /// ```graphql,ignore
    /// type Query {
    ///   getRandomMammoth: Mammoth @override(from: "steppe")
    ///                             ^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub fn r#override(self) -> Option<&'a OverrideDirective> {
        self.subgraphs.directives.r#override.get(&self.id)
    }

    pub(crate) fn policies(self) -> impl Iterator<Item = &'a [StringId]> {
        self.subgraphs
            .directives
            .policies
            .range((self.id, vec![])..)
            .take_while(move |(site, _)| *site == self.id)
            .map(|(_, policies)| policies.as_slice())
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   others: [OtherObject!] @provides("size weight")
    ///                          ^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn provides(self) -> Option<&'a [Selection]> {
        self.subgraphs
            .directives
            .provides
            .get(&self.id)
            .map(|provides| &**provides)
    }

    /// ```ignore.graphql
    /// extend type Farm @federation__key(fields: "id") {
    ///   id: ID! @federation__external
    ///   chiliId: ID! @federation__external
    ///   chiliDetails: ChiliVariety @federation__requires(fields: "chiliId")
    ///                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn requires(self) -> Option<&'a [Selection]> {
        self.subgraphs
            .directives
            .requires
            .get(&self.id)
            .map(|requires| &**requires)
    }

    pub(crate) fn requires_scopes(self) -> impl Iterator<Item = &'a [StringId]> {
        self.subgraphs
            .directives
            .requires_scopes
            .range((self.id, vec![])..)
            .take_while(move |(site, _)| *site == self.id)
            .map(|(_, scopes)| scopes.as_slice())
    }

    pub(crate) fn shareable(self) -> bool {
        self.subgraphs.directives.shareable.contains(&self.id)
    }

    /// ```graphql,ignore
    /// type Query {
    ///     findManyUser(
    ///       filters: FindManyUserFilter?,
    ///       searchQuery: String?
    ///     ): [User!]! @tag(name: "Taste") @tag(name: "the") @tag(name: "Rainbow")
    ///                 ^^^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn tags(self) -> impl Iterator<Item = StringWalker<'a>> {
        self.subgraphs
            .directives
            .tags
            .range((self.id, StringId::MIN)..(self.id, StringId::MAX))
            .map(move |(_, id)| self.walk(*id))
    }
}

#[derive(Debug)]
pub(crate) struct OverrideDirective {
    pub(crate) from: StringId,
    pub(crate) label: Option<StringId>,
}

#[derive(Debug)]
pub(crate) struct AuthorizedDirective {
    pub(crate) arguments: Option<Vec<Selection>>,
    pub(crate) fields: Option<Vec<Selection>>,
    pub(crate) node: Option<Vec<Selection>>,
    pub(crate) metadata: Option<Value>,
}

/// Corresponds to an `@deprecated` directive.
pub(crate) type DeprecatedWalker<'a> = Walker<'a, &'a Deprecated>;

impl<'a> DeprecatedWalker<'a> {
    pub(crate) fn reason(self) -> Option<StringWalker<'a>> {
        self.id.reason.map(|reason| self.walk(reason))
    }
}

/// Corresponds to an `@deprecated` directive.
pub(crate) struct Deprecated {
    pub(crate) reason: Option<StringId>,
}
