use id_newtypes::IdRange;
use operation::{Location, QueryPosition};
use schema::FieldDefinition;
use walker::Walk;

use crate::prepare::{
    DataOrLookupField, DataOrLookupFieldId, OperationPlanContext, PartitionFieldArgumentId, PlanFieldArguments,
    SubgraphSelectionSet,
};

#[derive(Clone, Copy)]
pub(crate) struct SubgraphField<'a> {
    pub(in crate::prepare::operation_plan) ctx: OperationPlanContext<'a>,
    pub id: DataOrLookupFieldId,
}

#[allow(unused)]
impl<'a> SubgraphField<'a> {
    #[allow(clippy::should_implement_trait)]
    fn inner(&self) -> DataOrLookupField<'a> {
        self.id.walk(self.ctx)
    }
    pub fn subgraph_response_key_str(&self) -> &'a str {
        let field = self.inner();
        let key = field.subgraph_key();
        &self.ctx.cached.operation.response_keys[key]
    }

    pub fn query_position(&self) -> Option<QueryPosition> {
        match self.inner() {
            DataOrLookupField::Data(field) => field.query_position,
            DataOrLookupField::Lookup(field) => None,
        }
    }

    pub fn location(&self) -> Location {
        self.inner().location()
    }

    pub fn definition(&self) -> FieldDefinition<'a> {
        self.inner().definition()
    }

    pub fn argument_ids(&self) -> IdRange<PartitionFieldArgumentId> {
        match self.inner() {
            DataOrLookupField::Data(field) => field.argument_ids,
            DataOrLookupField::Lookup(field) => field.argument_ids,
        }
    }

    pub fn arguments(&self) -> PlanFieldArguments<'a> {
        self.inner().arguments()
    }

    pub fn selection_set(&self) -> SubgraphSelectionSet<'a> {
        match self.inner() {
            DataOrLookupField::Data(field) => SubgraphSelectionSet {
                ctx: self.ctx,
                item: field.selection_set_record,
                requires_typename: field.selection_set_requires_typename,
            },
            DataOrLookupField::Lookup(field) => SubgraphSelectionSet {
                ctx: self.ctx,
                item: field.selection_set_record,
                requires_typename: false,
            },
        }
    }
}

impl std::fmt::Debug for SubgraphField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanDataField")
            .field("key", &self.subgraph_response_key_str())
            .field("location", &self.location())
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> runtime::extension::Field<'a> for SubgraphField<'a> {
    type SelectionSet = SubgraphSelectionSet<'a>;

    fn alias(&self) -> Option<&'a str> {
        let key_str = self.subgraph_response_key_str();
        if key_str != self.definition().name() {
            Some(key_str)
        } else {
            None
        }
    }

    fn definition(&self) -> FieldDefinition<'a> {
        self.definition()
    }

    fn arguments(&self) -> Option<runtime::extension::ArgumentsId> {
        if self.inner().arguments().len() == 0 {
            None
        } else {
            Some(runtime::extension::ArgumentsId(self.id.into()))
        }
    }

    fn selection_set(&self) -> Option<Self::SelectionSet> {
        let selection_set = self.selection_set();
        if selection_set.is_empty() {
            None
        } else {
            Some(selection_set)
        }
    }

    fn as_dyn(&self) -> Box<dyn runtime::extension::DynField<'a>> {
        Box::new(*self)
    }
}

impl<'a> runtime::extension::DynField<'a> for SubgraphField<'a> {
    fn alias(&self) -> Option<&'a str> {
        runtime::extension::Field::alias(self)
    }
    fn definition(&self) -> FieldDefinition<'a> {
        self.definition()
    }
    fn arguments(&self) -> Option<runtime::extension::ArgumentsId> {
        runtime::extension::Field::arguments(self)
    }
    fn selection_set(&self) -> Option<Box<dyn runtime::extension::DynSelectionSet<'a>>> {
        runtime::extension::Field::selection_set(self)
            .map(|s| -> Box<dyn runtime::extension::DynSelectionSet<'a>> { Box::new(s) })
    }
}
