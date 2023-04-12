use query_planning::logical_plan::apply_fct::ApplyFunction;
use serde::{Deserialize, Serialize};

use super::SchemaID;

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SchemaPlan {
    /// Create a projection by selecting only a subset of the data.
    Projection(PlanProjection),
    /// Get the Entities related to this node
    Related(PlanRelated),
    /// Apply a function on a column
    Apply(Apply),
    First(First),
    Last(Last),
    PaginationPage(PaginationPage),
    Resolver(Resolver),
}

impl SchemaPlan {
    pub fn is_from_maindb(&self) -> bool {
        match self {
            SchemaPlan::Projection(_) => true, // We don't know in fact.
            SchemaPlan::Related(_) => true,
            SchemaPlan::Apply(ref input) => input.plan.as_ref().is_from_maindb(),
            SchemaPlan::First(ref input) => input
                .plan
                .as_ref()
                .map(|x| x.is_from_maindb())
                .unwrap_or(true),
            SchemaPlan::Last(ref input) => input
                .plan
                .as_ref()
                .map(|x| x.is_from_maindb())
                .unwrap_or(true),
            SchemaPlan::PaginationPage(_) => true,
            SchemaPlan::Resolver(_) => false,
        }
    }
}

impl SchemaPlan {
    pub fn projection(fields: Vec<String>) -> Self {
        Self::Projection(PlanProjection { fields })
    }

    pub fn resolver(resolver_name: String) -> Self {
        Self::Resolver(Resolver { resolver_name })
    }

    pub fn related(
        from: Option<SchemaID>,
        to: SchemaID,
        relation_name: Option<String>,
        ty: String,
    ) -> Self {
        Self::Related(PlanRelated {
            from,
            to,
            relation_name,
            ty,
        })
    }

    pub fn apply_cursor_encode(self, fields: Vec<String>) -> Self {
        Self::Apply(Apply {
            plan: Box::new(self),
            fun_fields: fields
                .into_iter()
                .map(|field| (field, ApplyFunction::CursorEncode))
                .collect(),
        })
    }

    pub fn first(previous: Option<Self>) -> Self {
        Self::First(First {
            plan: previous.map(Box::new),
        })
    }

    pub fn last(previous: Option<Self>) -> Self {
        Self::Last(Last {
            plan: previous.map(Box::new),
        })
    }

    pub fn pagination_page(page: PaginationPage) -> Self {
        Self::PaginationPage(page)
    }
}

/// Describe the fields projected
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanProjection {
    pub(crate) fields: Vec<String>,
}

/// Describe the relation
/// TODO: When handling Union for GraphQL: We need to sort an Union of multiple Schema.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanRelated {
    pub(crate) from: Option<SchemaID>,
    pub(crate) to: SchemaID,
    pub(crate) relation_name: Option<String>,
    /// Type name for the output Schema.
    pub(crate) ty: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Apply {
    pub plan: Box<SchemaPlan>,
    pub(crate) fun_fields: Vec<(String, ApplyFunction)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct First {
    pub plan: Option<Box<SchemaPlan>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Last {
    pub plan: Option<Box<SchemaPlan>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaginationPage {
    Next,
    Previous,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resolver {
    pub resolver_name: String,
}
