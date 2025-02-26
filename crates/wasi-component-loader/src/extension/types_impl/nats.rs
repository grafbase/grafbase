use std::time::Duration;

use async_nats::{ServerAddr, jetstream};
use wasmtime::component::Resource;

use crate::{
    WasiState,
    extension::wit::{
        self, HostNatsClient, HostNatsKeyValue, HostNatsSubscriber, NatsAuth, NatsKeyValue, NatsMessage,
        NatsStreamConfig,
    },
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

        let result = client
            .publish(subject, message.into())
            .await
            .map_err(|err| err.to_string());

        Ok(result)
    }

    async fn subscribe(
        &mut self,
        self_: Resource<NatsClient>,
        subject: String,
        config: Option<NatsStreamConfig>,
    ) -> wasmtime::Result<Result<Resource<NatsSubscriber>, String>> {
        let client = self.get_mut(&self_)?;

        let Some(config) = config else {
            let result = match client.subscribe(subject).await {
                Ok(subscriber) => {
                    let subscriber = self.push_resource(NatsSubscriber::Subject(subscriber))?;
                    Ok(Ok(subscriber))
                }
                Err(err) => Ok(Err(err.to_string())),
            };

            return result;
        };

        let client = client.clone();
        let context = jetstream::new(client);

        let NatsStreamConfig {
            consumer_name,
            durable_name,
            deliver_policy,
            inactive_threshold_ms,
            description,
            stream_name,
        } = config;

        let consumer_config = jetstream::consumer::pull::Config {
            durable_name,
            name: Some(consumer_name.clone()),
            description,
            deliver_policy: deliver_policy.into(),
            inactive_threshold: Duration::from_millis(inactive_threshold_ms),
            ..Default::default()
        };

        let stream = match context.get_stream(&stream_name).await {
            Ok(stream) => stream,
            Err(err) => return Ok(Err(err.to_string())),
        };

        let consumer = match stream.get_or_create_consumer(&consumer_name, consumer_config).await {
            Ok(consumer) => consumer,
            Err(err) => return Ok(Err(err.to_string())),
        };

        match consumer.messages().await {
            Ok(stream) => {
                let subscriber = self.push_resource(NatsSubscriber::Stream(stream))?;
                Ok(Ok(subscriber))
            }
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn request(
        &mut self,
        self_: Resource<NatsClient>,
        subject: String,
        message: Vec<u8>,
        timeout_ms: Option<u64>,
    ) -> wasmtime::Result<Result<NatsMessage, String>> {
        let client = self.get_mut(&self_)?;
        let request = client.request(subject, message.into());

        let result = match timeout_ms {
            Some(ms) => {
                let duration = Duration::from_millis(ms);

                match tokio::time::timeout(duration, request).await {
                    Ok(message) => message,
                    Err(err) => {
                        return Ok(Err(err.to_string()));
                    }
                }
            }
            None => request.await,
        };

        match result {
            Ok(message) => Ok(Ok(NatsMessage {
                subject: message.subject.to_string(),
                payload: message.payload.into(),
            })),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn key_value(
        &mut self,
        self_: Resource<NatsClient>,
        bucket: String,
    ) -> wasmtime::Result<Result<Resource<NatsKeyValue>, String>> {
        let client = self.get_mut(&self_)?;
        let stream = async_nats::jetstream::new(client.clone());

        match stream.get_key_value(bucket).await {
            Ok(store) => {
                let resource = self.push_resource(store)?;
                Ok(Ok(resource))
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
    async fn next(&mut self, self_: Resource<NatsSubscriber>) -> wasmtime::Result<Result<Option<NatsMessage>, String>> {
        let subscriber = self.get_mut(&self_)?;

        match subscriber.next().await {
            Ok(Some(message)) => Ok(Ok(Some(NatsMessage {
                subject: message.subject.to_string(),
                payload: message.payload.into(),
            }))),
            Ok(None) => Ok(Ok(None)),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<NatsSubscriber>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl HostNatsKeyValue for WasiState {
    async fn create(
        &mut self,
        self_: Resource<NatsKeyValue>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, String>> {
        let store = self.get_mut(&self_)?;

        match store.create(&key, value.into()).await {
            Ok(seq) => Ok(Ok(seq)),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn put(
        &mut self,
        self_: Resource<NatsKeyValue>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<u64, String>> {
        let store = self.get_mut(&self_)?;

        match store.put(&key, value.into()).await {
            Ok(seq) => Ok(Ok(seq)),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn get(
        &mut self,
        self_: Resource<NatsKeyValue>,
        key: String,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, String>> {
        let store = self.get_mut(&self_)?;

        match store.get(&key).await {
            Ok(value) => Ok(Ok(value.map(Into::into))),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn delete(&mut self, self_: Resource<NatsKeyValue>, key: String) -> wasmtime::Result<Result<(), String>> {
        let store = self.get_mut(&self_)?;

        match store.delete(&key).await {
            Ok(()) => Ok(Ok(())),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<NatsKeyValue>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}

impl From<wit::NatsStreamDeliverPolicy> for jetstream::consumer::DeliverPolicy {
    fn from(policy: wit::NatsStreamDeliverPolicy) -> Self {
        match policy {
            wit::NatsStreamDeliverPolicy::All => jetstream::consumer::DeliverPolicy::All,
            wit::NatsStreamDeliverPolicy::Last => jetstream::consumer::DeliverPolicy::Last,
            wit::NatsStreamDeliverPolicy::New => jetstream::consumer::DeliverPolicy::New,
            wit::NatsStreamDeliverPolicy::LastPerSubject => jetstream::consumer::DeliverPolicy::LastPerSubject,
            wit::NatsStreamDeliverPolicy::ByStartSequence(start_sequence) => {
                jetstream::consumer::DeliverPolicy::ByStartSequence { start_sequence }
            }
            wit::NatsStreamDeliverPolicy::ByStartTimeMs(ms) => {
                let start_time = time::OffsetDateTime::from_unix_timestamp_nanos((ms * 1_000_000) as i128)
                    .map_err(|e| e.to_string())
                    .unwrap();

                jetstream::consumer::DeliverPolicy::ByStartTime { start_time }
            }
        }
    }
}
