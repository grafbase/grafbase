use runtime::auth::LegacyToken;
use wasmtime::{Result, component::Resource};

use super::super::wit::{context, token};

use crate::{
    WasmOwnedOrBorrowed,
    resources::{AuthorizationContext, Headers},
    state::WasiState,
};

impl context::HostAuthorizationContext for WasiState {
    async fn headers(&mut self, self_: Resource<AuthorizationContext>) -> Result<Resource<Headers>> {
        let AuthorizationContext(ctx) = WasiState::get(self, &self_)?;
        // TODO: /facepalm Headers are already complicated enough with the hooks resources, so I'm
        // just cloning them here...
        let rep = self.table.push(WasmOwnedOrBorrowed::Owned(ctx.headers().clone()))?;
        Ok(rep)
    }

    async fn token(&mut self, self_: Resource<AuthorizationContext>) -> Result<token::Token> {
        let AuthorizationContext(ctx) = WasiState::get(self, &self_)?;

        let token = match ctx.token() {
            LegacyToken::Anonymous => token::Token::Anonymous,
            LegacyToken::Jwt(jwt) => token::Token::Bytes(serde_json::to_vec(&jwt.claims).unwrap()),
            LegacyToken::Extension(token) => token.clone().into(),
        };

        Ok(token)
    }

    async fn drop(&mut self, rep: Resource<AuthorizationContext>) -> Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}
