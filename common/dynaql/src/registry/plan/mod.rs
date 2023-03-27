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
}

impl SchemaPlan {
    pub fn projection(fields: Vec<String>) -> Self {
        Self::Projection(PlanProjection { fields })
    }

    pub fn related(from: Option<SchemaID>, to: SchemaID, relation_name: Option<String>) -> Self {
        Self::Related(PlanRelated {
            from,
            to,
            relation_name,
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

    pub fn pagination_page(forward: bool) -> Self {
        if forward {
            Self::PaginationPage(PaginationPage::Next)
        } else {
            Self::PaginationPage(PaginationPage::Previous)
        }
    }
}

/// Describe the fields projected
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanProjection {
    pub(crate) fields: Vec<String>,
}

/// Describe the relation
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanRelated {
    pub(crate) from: Option<SchemaID>,
    pub(crate) to: SchemaID,
    pub(crate) relation_name: Option<String>,
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
