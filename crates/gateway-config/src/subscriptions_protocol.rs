use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionsProtocol {
    ServerSentEvents,
    Websocket,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestStruct {
        subscriptions_protocol: SubscriptionsProtocol,
    }

    impl TestStruct {
        fn new(subscriptions_protocol: SubscriptionsProtocol) -> Self {
            Self { subscriptions_protocol }
        }
    }

    #[test]
    fn subscriptions_protocol_deserialize_sse() {
        let expected = TestStruct::new(SubscriptionsProtocol::ServerSentEvents);
        let actual = toml::from_str(r#"subscriptions_protocol = "server_sent_events""#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn subscriptions_protocol_deserialize_websockets() {
        let expected = TestStruct::new(SubscriptionsProtocol::Websocket);
        let actual = toml::from_str(r#"subscriptions_protocol = "websocket""#).unwrap();
        assert_eq!(expected, actual);
    }
}
