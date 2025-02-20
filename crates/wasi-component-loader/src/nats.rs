use std::future::Future;

use async_nats::ServerAddr;
use wasmtime::{
    component::{LinkerInstance, Resource, ResourceType},
    StoreContextMut,
};

use crate::{
    instance::extensions::NatsAuth,
    names::{NATS_CLIENT_CONNECT_FUNCTION, NATS_CLIENT_PUBLISH_METHOD, NATS_CLIENT_RESOURCE},
    state::WasiState,
};

type ConnectResult<'a> =
    Box<dyn Future<Output = anyhow::Result<(Result<Resource<async_nats::Client>, String>,)>> + Send + 'a>;

type PublishResult<'a> = Box<dyn Future<Output = anyhow::Result<(Result<(), String>,)>> + Send + 'a>;

pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        NATS_CLIENT_RESOURCE,
        ResourceType::host::<async_nats::Client>(),
        |mut ctx, id| {
            ctx.data_mut().take_resource(id)?;
            Ok(())
        },
    )?;

    types.func_wrap_async(NATS_CLIENT_CONNECT_FUNCTION, connect)?;
    types.func_wrap_async(NATS_CLIENT_PUBLISH_METHOD, publish)?;

    Ok(())
}

fn connect(
    mut ctx: StoreContextMut<'_, WasiState>,
    (servers, auth): (Vec<String>, Option<NatsAuth>),
) -> ConnectResult<'_> {
    Box::new(async move {
        let Ok(addrs) = servers
            .iter()
            .map(|url| url.parse())
            .collect::<Result<Vec<ServerAddr>, _>>()
        else {
            return Ok((Err("Failed to parse server URLs".to_string()),));
        };

        let opts = async_nats::ConnectOptions::new();

        let opts = match auth {
            Some(NatsAuth::UsernamePassword((username, password))) => opts.user_and_password(username, password),
            Some(NatsAuth::Token(token)) => opts.token(token),
            Some(NatsAuth::Credentials(ref credentials)) => match opts.credentials(credentials) {
                Ok(opts) => opts,
                Err(err) => return Ok((Err(err.to_string()),)),
            },
            None => opts,
        };

        match async_nats::connect_with_options(addrs, opts).await {
            Ok(client) => {
                let client = ctx.data_mut().push_resource(client)?;

                Ok((Ok(client),))
            }
            Err(err) => Ok((Err(err.to_string()),)),
        }
    })
}

fn publish(
    mut ctx: StoreContextMut<'_, WasiState>,
    (this, subject, message): (Resource<async_nats::Client>, String, Vec<u8>),
) -> PublishResult<'_> {
    Box::new(async move {
        let client = ctx.data_mut().get_mut(&this)?;

        match client.publish(subject, message.into()).await {
            Ok(()) => Ok((Ok(()),)),
            Err(e) => Ok((Err(e.to_string()),)),
        }
    })
}
