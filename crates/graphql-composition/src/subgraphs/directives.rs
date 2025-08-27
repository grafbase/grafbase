mod directive_definition;
mod link;
mod record;

pub(crate) use self::{directive_definition::*, link::*, record::*};

use super::*;
use crate::federated_graph::ListSizeDirective;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DirectiveSiteId(usize);

impl From<usize> for DirectiveSiteId {
    fn from(value: usize) -> Self {
        DirectiveSiteId(value)
    }
}

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
    Null,
}

#[derive(Default)]
pub(super) struct Directives {
    site_id_counter: usize,

    deprecated: BTreeMap<DirectiveSiteId, Deprecated>,
    r#override: BTreeMap<DirectiveSiteId, OverrideDirective>,
    provides: BTreeMap<DirectiveSiteId, Vec<Selection>>,
    requires: BTreeMap<DirectiveSiteId, Vec<Selection>>,

    requires_scopes: BTreeSet<(DirectiveSiteId, Vec<StringId>)>,
    policies: BTreeSet<(DirectiveSiteId, Vec<StringId>)>,

    authenticated: HashSet<DirectiveSiteId>,
    inaccessible: HashSet<DirectiveSiteId>,
    one_of: HashSet<DirectiveSiteId>,
    shareable: HashSet<DirectiveSiteId>,
    external: HashSet<DirectiveSiteId>,
    interface_object: HashSet<DirectiveSiteId>,

    tags: BTreeSet<(DirectiveSiteId, StringId)>,

    costs: BTreeMap<DirectiveSiteId, i32>,
    list_sizes: BTreeMap<DirectiveSiteId, ListSizeDirective>,

    pub(super) directive_definitions: Vec<DirectiveDefinition>,
    composed_directives: HashSet<(SubgraphId, StringId)>,

    /// Directives that can go straight to composition IR.
    ir_directives: Vec<(DirectiveSiteId, crate::composition_ir::Directive)>,
    pub(super) extra_directives: Vec<ExtraDirectiveRecord>,
    extra_directives_on_schema_definition: Vec<(SubgraphId, ExtraDirectiveRecord)>,
}

impl Subgraphs {
    pub(crate) fn directive_definitions(&self) -> &[DirectiveDefinition] {
        &self.directives.directive_definitions
    }

    pub(crate) fn insert_authenticated(&mut self, id: DirectiveSiteId) {
        self.directives.authenticated.insert(id);
    }

    pub(crate) fn insert_composed_directive(&mut self, subgraph_id: SubgraphId, directive_name: &str) {
        let directive_name = self.strings.intern(directive_name);

        self.directives
            .composed_directives
            .insert((subgraph_id, directive_name));
    }

    pub(crate) fn insert_deprecated(&mut self, id: DirectiveSiteId, reason: Option<&str>) {
        let reason = reason.map(|reason| self.strings.intern(reason));
        self.directives.deprecated.insert(id, Deprecated { reason });
    }

    pub(crate) fn insert_provides(&mut self, id: DirectiveSiteId, fields: &str) -> Result<(), String> {
        let fields = self.selection_set_from_str(fields, "provides", "fields")?;
        self.directives.provides.insert(id, fields);
        Ok(())
    }

    pub(crate) fn insert_requires(&mut self, id: DirectiveSiteId, fields: &str) -> Result<(), String> {
        let fields = self.selection_set_from_str(fields, "requires", "fields")?;
        self.directives.requires.insert(id, fields);
        Ok(())
    }

    pub(crate) fn insert_policy(&mut self, id: DirectiveSiteId, policies: Vec<StringId>) {
        self.directives.policies.insert((id, policies));
    }

    pub(crate) fn is_composed_directive(&self, subgraph_id: SubgraphId, name_id: StringId) -> bool {
        self.directives.composed_directives.contains(&(subgraph_id, name_id))
    }

    pub(crate) fn append_required_scopes(&mut self, id: DirectiveSiteId, scopes: Vec<StringId>) {
        self.directives.requires_scopes.insert((id, scopes));
    }

    pub(crate) fn insert_tag(&mut self, id: DirectiveSiteId, tag: &str) {
        let tag = self.strings.intern(tag);
        self.directives.tags.insert((id, tag));
    }

    pub(crate) fn push_extra_directive_on_schema_definition(
        &mut self,
        subgraph_id: SubgraphId,
        directive: ExtraDirectiveRecord,
    ) {
        self.directives
            .extra_directives_on_schema_definition
            .push((subgraph_id, directive));
    }

    pub(crate) fn iter_extra_directives_on_schema_definition(
        &self,
    ) -> impl Iterator<Item = &(SubgraphId, ExtraDirectiveRecord)> {
        self.directives.extra_directives_on_schema_definition.iter()
    }

    pub(crate) fn push_directive_definition(&mut self, definition: DirectiveDefinition) -> DirectiveDefinitionId {
        self.directives.directive_definitions.push_return_idx(definition).into()
    }

    /// Push a directive that can go straight to composition IR.
    pub(crate) fn push_ir_directive(
        &mut self,
        directive_site_id: DirectiveSiteId,
        directive: crate::composition_ir::Directive,
    ) {
        self.directives.ir_directives.push((directive_site_id, directive));
    }

    pub(crate) fn push_directive(&mut self, directive: ExtraDirectiveRecord) {
        if let Some(last) = self.directives.extra_directives.last() {
            assert!(directive.directive_site_id >= last.directive_site_id);
        }

        self.directives.extra_directives.push(directive);
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

    pub(crate) fn set_one_of(&mut self, id: DirectiveSiteId) {
        self.directives.one_of.insert(id);
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

    pub(crate) fn set_cost(&mut self, id: DirectiveSiteId, cost: i32) {
        self.directives.costs.insert(id, cost);
    }

    pub(crate) fn set_list_size(&mut self, id: DirectiveSiteId, directive: ListSizeDirective) {
        self.directives.list_sizes.insert(id, directive);
    }
}

impl DirectiveSiteId {
    pub(crate) fn authenticated(&self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.authenticated.contains(self)
    }

    pub(crate) fn deprecated<'a>(&self, subgraphs: &'a Subgraphs) -> Option<&'a Deprecated> {
        subgraphs.directives.deprecated.get(self)
    }

    pub(crate) fn external(&self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.external.contains(self)
    }

    pub(crate) fn inaccessible(&self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.inaccessible.contains(self)
    }

    pub(crate) fn interface_object(&self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.interface_object.contains(self)
    }

    pub(crate) fn iter_extra_directives(self, subgraphs: &Subgraphs) -> impl Iterator<Item = ExtraDirective<'_>> {
        let instances = &subgraphs.directives.extra_directives;
        let partition_point = instances.partition_point(|record| record.directive_site_id < self);

        instances[partition_point..]
            .iter()
            .take_while(move |record| record.directive_site_id == self)
            .enumerate()
            .map(move |(idx, record)| ExtraDirective {
                id: (partition_point + idx).into(),
                record,
            })
    }

    pub(crate) fn iter_ir_directives(
        self,
        subgraphs: &Subgraphs,
    ) -> impl Iterator<Item = &crate::composition_ir::Directive> {
        let instances = &subgraphs.directives.ir_directives;
        let partition_point = instances.partition_point(|(directive_site_id, _)| *directive_site_id < self);

        instances[partition_point..]
            .iter()
            .take_while(move |(directive_site_id, _)| *directive_site_id == self)
            .map(|(_, record)| record)
    }

    pub(crate) fn one_of(self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.one_of.contains(&self)
    }

    pub(crate) fn shareable(self, subgraphs: &Subgraphs) -> bool {
        subgraphs.directives.shareable.contains(&self)
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   others: [OtherObject!] @provides("size weight")
    ///                          ^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn provides(self, subgraphs: &Subgraphs) -> Option<&[Selection]> {
        subgraphs.directives.provides.get(&self).map(|provides| &**provides)
    }

    /// ```ignore.graphql
    /// extend type Farm @federation__key(fields: "id") {
    ///   id: ID! @federation__external
    ///   chiliId: ID! @federation__external
    ///   chiliDetails: ChiliVariety @federation__requires(fields: "chiliId")
    ///                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn requires(self, subgraphs: &Subgraphs) -> Option<&[Selection]> {
        subgraphs.directives.requires.get(&self).map(|requires| &**requires)
    }

    /// ```graphql,ignore
    /// type Query {
    ///   getRandomMammoth: Mammoth @override(from: "steppe")
    ///                             ^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn r#override(self, subgraphs: &Subgraphs) -> Option<&OverrideDirective> {
        subgraphs.directives.r#override.get(&self)
    }

    pub(crate) fn policies<'a>(self, subgraphs: &'a Subgraphs) -> impl Iterator<Item = &'a [StringId]> {
        subgraphs
            .directives
            .policies
            .range((self, vec![])..)
            .take_while(move |(site, _)| *site == self)
            .map(|(_, policies)| policies.as_slice())
    }

    pub(crate) fn requires_scopes<'a>(self, subgraphs: &'a Subgraphs) -> impl Iterator<Item = &'a [StringId]> {
        subgraphs
            .directives
            .requires_scopes
            .range((self, vec![])..)
            .take_while(move |(site, _)| *site == self)
            .map(|(_, scopes)| scopes.as_slice())
    }

    pub(crate) fn cost(self, subgraphs: &Subgraphs) -> Option<i32> {
        subgraphs.directives.costs.get(&self).copied()
    }

    pub(crate) fn list_size(self, subgraphs: &Subgraphs) -> Option<&ListSizeDirective> {
        subgraphs.directives.list_sizes.get(&self)
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
    pub(crate) fn tags(self, subgraphs: &Subgraphs) -> impl Iterator<Item = StringId> {
        subgraphs
            .directives
            .tags
            .range((self, StringId::MIN)..(self, StringId::MAX))
            .map(|(_, tag)| *tag)
    }
}

#[derive(Debug)]
pub(crate) struct OverrideDirective {
    pub(crate) from: StringId,
    pub(crate) label: Option<StringId>,
}

/// Corresponds to an `@deprecated` directive.
pub(crate) struct Deprecated {
    pub(crate) reason: Option<StringId>,
}

impl SubgraphId {
    pub(crate) fn iter_directive_definitions(
        self,
        subgraphs: &Subgraphs,
    ) -> impl Iterator<Item = View<'_, DirectiveDefinitionId, DirectiveDefinition>> {
        let start = subgraphs
            .directives
            .directive_definitions
            .partition_point(|def| def.subgraph_id < self);

        subgraphs.directives.directive_definitions[start..]
            .iter()
            .take_while(move |def| def.subgraph_id == self)
            .enumerate()
            .map(move |(idx, record)| View {
                id: (start + idx).into(),
                record,
            })
    }
}
