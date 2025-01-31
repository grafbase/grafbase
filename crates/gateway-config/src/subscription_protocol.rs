use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionProtocol {
    ServerSentEvents,
    Websocket,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestStruct {
        subscription_protocol: SubscriptionProtocol,
    }

    impl TestStruct {
        fn new(subscription_protocol: SubscriptionProtocol) -> Self {
            Self { subscription_protocol }
        }
    }

    #[test]
    fn subscriptions_protocol_deseralize_sse() {
        let expected = TestStruct::new(SubscriptionProtocol::ServerSentEvents);
        let actual = toml::from_str(r#"subscription_protocol = "server_sent_events""#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn subscriptions_protocol_deserialize_ebsockets() {
        let expected = TestStruct::new(SubscriptionProtocol::Websocket);
        let actual = toml::from_str(r#"subscription_protocol = "websocket""#).unwrap();
        assert_eq!(expected, actual);
    }
}
