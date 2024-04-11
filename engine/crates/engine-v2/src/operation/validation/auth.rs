use crate::{
    execution::ExecutionContext,
    operation::{Location, OperationWalker, SelectionSetWalker},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Not authenticated")]
    NotAuthenticated { location: Location },
    #[error("Not allowed")]
    InvalidOAuth2Scope { location: Location },
}

impl AuthError {
    pub fn location(&self) -> Location {
        match self {
            Self::NotAuthenticated { location } | Self::InvalidOAuth2Scope { location } => *location,
        }
    }
}

pub(super) fn validate_auth(ctx: ExecutionContext<'_>, operation: OperationWalker<'_>) -> Result<(), AuthError> {
    let scopes = ctx
        .access_token
        .get_claim("scope")
        .as_str()
        .map(|scope| scope.split(' ').collect::<Vec<_>>())
        .unwrap_or_default();
    let is_anonymous = ctx.access_token.is_anonymous();
    operation.selection_set().validate_auth(is_anonymous, &scopes)
}

impl SelectionSetWalker<'_> {
    fn validate_auth(&self, is_anonymous: bool, scopes: &[&str]) -> Result<(), AuthError> {
        for field in self.fields() {
            if field.is_extra() {
                continue;
            }
            if let Some(directives) = field.definition().map(|d| d.directives()) {
                if directives.has_authenticated() && is_anonymous {
                    return Err(AuthError::NotAuthenticated {
                        location: field.location(),
                    });
                }
                if let Some(required_scopes) = directives.requires_scopes() {
                    if !required_scopes.matches(scopes) {
                        return Err(AuthError::InvalidOAuth2Scope {
                            location: field.location(),
                        });
                    }
                }
            }
            if let Some(subselection) = field.selection_set() {
                subselection.validate_auth(is_anonymous, scopes)?;
            }
        }
        Ok(())
    }
}
