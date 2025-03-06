use wasmtime::{Result, component::Resource};

use crate::{resources::Headers, state::WasiState};

use super::{AuthorizationContext, HostAuthorizationContext, Token};

impl HostAuthorizationContext for WasiState {
    async fn headers(&mut self, self_: Resource<AuthorizationContext>) -> Result<Resource<Headers>> {
        let AuthorizationContext(ctx) = WasiState::get(self, &self_)?;
        // TODO: /facepalm Headers are already complicated enough with the hooks resources, so I'm
        // just cloning them here...
        let rep = self
            .table
            .push(crate::WasmOwnedOrBorrowed::Owned(ctx.headers().clone()))?;
        Ok(rep)
    }

    async fn token(&mut self, self_: Resource<AuthorizationContext>) -> Result<Token> {
        let AuthorizationContext(ctx) = WasiState::get(self, &self_)?;
        let token = match ctx.token() {
            runtime::auth::LegacyToken::Anonymous => Token::Anonymous,
            runtime::auth::LegacyToken::Jwt(jwt) => Token::Bytes(serde_json::to_vec(&jwt.claims).unwrap()),
            runtime::auth::LegacyToken::Extension(token) => token.clone().into(),
        };
        Ok(token)
    }

    async fn drop(&mut self, rep: Resource<AuthorizationContext>) -> Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
