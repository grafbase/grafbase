//! Pagination cursor.

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ParentEdge {
    pub relation_name: String,
    pub parent_id: String,
}

/// A Cursor.
/// The first elements are the most recents ones.
/// The last elements are the most anciens.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum PaginatedCursor {
    // after
    Forward {
        exclusive_last_key: Option<String>,
        first: usize,
        maybe_parent_edge: Option<ParentEdge>,
    },
    // before
    Backward {
        exclusive_first_key: Option<String>,
        last: usize,
        maybe_parent_edge: Option<ParentEdge>,
    },
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CursorCreation {
    #[error("The `first` and `last` parameters cannot exist at the same time.")]
    SameParameterSameTime,
    #[error("The `first` parameter must be a non-negative number.")]
    FirstNonNegative,
    #[error("The `last` parameter must be a non-negative number.")]
    LastNonNegative,
    #[error("The `first` and `before` parameters cannot exist at the same time.")]
    FirstAndBeforeSameTime,
    #[error("The `last` and `after` parameters cannot exist at the same time.")]
    LastAndAfterSameTime,
    #[error("You must choose a pagination direction by having the `first` or `last` parameter.")]
    Direction,
}

impl PaginatedCursor {
    /// To create the Cursor from GraphQL Input
    #[allow(
        clippy::missing_const_for_fn,
        /* reason = "False positive, destructors cannot be evaluated at compile-time" */
    )]
    pub fn from_graphql(
        first: Option<usize>,
        last: Option<usize>,
        after: Option<String>,
        before: Option<String>,
        nested: Option<ParentEdge>,
    ) -> Result<Self, CursorCreation> {
        match (first, after, last, before) {
            (Some(_), _, Some(_), _) => Err(CursorCreation::SameParameterSameTime),
            (Some(_), _, _, Some(_)) => Err(CursorCreation::FirstAndBeforeSameTime),
            (_, Some(_), Some(_), _) => Err(CursorCreation::LastAndAfterSameTime),
            (Some(first), after, None, None) => Ok(Self::Forward {
                exclusive_last_key: after,
                first,
                maybe_parent_edge: nested,
            }),
            (None, None, Some(last), before) => Ok(Self::Backward {
                exclusive_first_key: before,
                last,
                maybe_parent_edge: nested,
            }),
            (None, _, None, _) => Err(CursorCreation::Direction),
        }
    }

    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward { .. })
    }

    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward { .. })
    }

    pub fn maybe_parent_edge(&self) -> Option<&ParentEdge> {
        match self {
            PaginatedCursor::Forward { maybe_parent_edge, .. }
            | PaginatedCursor::Backward { maybe_parent_edge, .. } => maybe_parent_edge.as_ref(),
        }
    }

    pub fn maybe_origin(&self) -> Option<String> {
        match self {
            PaginatedCursor::Forward { exclusive_last_key, .. } => exclusive_last_key.clone(),
            PaginatedCursor::Backward {
                exclusive_first_key, ..
            } => exclusive_first_key.clone(),
        }
    }

    pub fn nested_parent_pk(&self) -> Option<String> {
        self.maybe_parent_edge()
            .map(|parent_edge| parent_edge.parent_id.clone())
    }

    pub fn limit(&self) -> usize {
        match self {
            PaginatedCursor::Forward { first, .. } => *first,
            PaginatedCursor::Backward { last, .. } => *last,
        }
    }
}
