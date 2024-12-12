use walker::Walk;

use crate::{OperationContext, QueryInputValueId, QueryInputValueRecord, VariableDefinitionId};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct IncludeDirectiveRecord {
    pub condition: QueryInputValueId,
}

#[derive(Clone, Copy)]
pub struct IncludeDirective<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub(in crate::model) item: IncludeDirectiveRecord,
}

impl std::ops::Deref for IncludeDirective<'_> {
    type Target = IncludeDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl IncludeDirective<'_> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &IncludeDirectiveRecord {
        &self.item
    }
}

impl<'a> Walk<OperationContext<'a>> for IncludeDirectiveRecord {
    type Walker<'w>
        = IncludeDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        IncludeDirective {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for IncludeDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("SkipDirective");
        match self.ctx.operation.query_input_values[self.item.condition] {
            QueryInputValueRecord::Boolean(b) => f.field("condition", &b).finish(),
            QueryInputValueRecord::Variable(id) => f
                .field(
                    "condition",
                    &format!(
                        "${}",
                        <VariableDefinitionId as Walk<OperationContext<'_>>>::walk(id, self.ctx).name
                    ),
                )
                .finish(),
            _ => f.field("condition", &"???").finish(),
        }
    }
}
