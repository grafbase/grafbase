use wasmtime::component::Resource;

use crate::{
    resources::{AuthorizationContext, Headers},
    state::WasiState,
};

use super::grafbase::sdk::context::*;

impl Host for WasiState {}

impl HostSharedContext for WasiState {
    async fn get(&mut self, self_: Resource<SharedContext>, name: String) -> wasmtime::Result<Option<String>> {
        let ctx = WasiState::get(self, &self_)?;
        Ok(ctx.kv.get(&name).cloned())
    }

    async fn trace_id(&mut self, _: Resource<SharedContext>) -> wasmtime::Result<String> {
        Ok(String::new())
    }

    async fn drop(&mut self, rep: Resource<SharedContext>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl HostAuthorizationContext for WasiState {
    async fn headers(&mut self, self_: Resource<AuthorizationContext>) -> wasmtime::Result<Resource<Headers>> {
        let AuthorizationContext { headers, .. } = WasiState::get(self, &self_)?;
        // TODO: /facepalm Headers are already complicated enough with the hooks resources, so I'm
        // just cloning them here...
        let rep = self
            .table
            .push(headers.with_ref(|headers| Headers::from(headers.clone())).await)?;
        Ok(rep)
    }

    async fn token(&mut self, self_: Resource<AuthorizationContext>) -> wasmtime::Result<Token> {
        let AuthorizationContext { token, .. } = WasiState::get(self, &self_)?;

        Ok(match token {
            runtime::extension::Token::Anonymous => Token::Anonymous,
            runtime::extension::Token::Bytes(bytes) => Token::Bytes(bytes.clone()),
        })
    }

    async fn drop(&mut self, _rep: Resource<AuthorizationContext>) -> wasmtime::Result<()> {
        Ok(())
    }
}
