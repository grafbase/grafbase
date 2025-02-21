use async_nats::ServerAddr;
use futures::StreamExt;
use wasmtime::component::Resource;

use crate::{
    WasiState,
    extension::wit::{HostNatsClient, HostNatsSubscriber, NatsAuth, NatsMessage},
    resources::{NatsClient, NatsSubscriber},
};

impl HostNatsClient for WasiState {
    async fn connect(
        &mut self,
        servers: Vec<String>,
        auth: Option<NatsAuth>,
    ) -> wasmtime::Result<Result<Resource<NatsClient>, String>> {
        let Ok(addrs) = servers
            .iter()
            .map(|url| url.parse())
            .collect::<Result<Vec<ServerAddr>, _>>()
        else {
            return Ok(Err("Failed to parse server URLs".to_string()));
        };

        let opts = async_nats::ConnectOptions::new();

        let opts = match auth {
            Some(NatsAuth::UsernamePassword((username, password))) => opts.user_and_password(username, password),
            Some(NatsAuth::Token(token)) => opts.token(token),
            Some(NatsAuth::Credentials(ref credentials)) => match opts.credentials(credentials) {
                Ok(opts) => opts,
                Err(err) => return Ok(Err(err.to_string())),
            },
            None => opts,
        };

        Ok(match async_nats::connect_with_options(addrs, opts).await {
            Ok(client) => {
                let client = self.push_resource(client)?;

                Ok(client)
            }
            Err(err) => Err(err.to_string()),
        })
    }

    async fn publish(
        &mut self,
        self_: Resource<NatsClient>,
        subject: String,
        message: Vec<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        let client = self.get_mut(&self_)?;

        Ok(client
            .publish(subject, message.into())
            .await
            .map_err(|err| err.to_string()))
    }

    async fn subscribe(
        &mut self,
        self_: Resource<NatsClient>,
        subject: String,
    ) -> wasmtime::Result<Result<Resource<NatsSubscriber>, String>> {
        let client = self.get_mut(&self_)?;

        match client.subscribe(subject).await {
            Ok(subscriber) => {
                let subscriber = self.push_resource(subscriber)?;
                Ok(Ok(subscriber))
            }
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<NatsClient>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl HostNatsSubscriber for WasiState {
    async fn next(&mut self, self_: Resource<NatsSubscriber>) -> wasmtime::Result<Option<NatsMessage>> {
        let subscriber = self.get_mut(&self_)?;

        match subscriber.next().await {
            Some(message) => Ok(Some(NatsMessage {
                subject: message.subject.to_string(),
                payload: message.payload.into(),
            })),
            None => Ok(None),
        }
    }

    async fn drop(&mut self, rep: Resource<NatsSubscriber>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
