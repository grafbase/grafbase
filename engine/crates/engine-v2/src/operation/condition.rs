use std::ops::BitAnd;

use schema::RequiredScopesId;

use crate::response::GraphqlError;

use super::ConditionId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Condition {
    Authenticated,
    RequiresScopes(RequiredScopesId),
    All(Vec<ConditionId>),
}

#[derive(Debug)]
pub(crate) enum ConditionResult {
    Include,
    Errors(Vec<GraphqlError>),
}

impl BitAnd<&Self> for ConditionResult {
    type Output = ConditionResult;

    fn bitand(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Self::Include, Self::Include) => Self::Include,
            (Self::Errors(mut errors), Self::Errors(other_errors)) => {
                errors.extend_from_slice(other_errors);
                Self::Errors(errors)
            }
            (err @ Self::Errors(_), Self::Include) => err,
            (Self::Include, Self::Errors(errors)) => Self::Errors(errors.clone()),
        }
    }
}
