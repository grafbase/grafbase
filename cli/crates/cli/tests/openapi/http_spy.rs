use crossbeam_channel::{Receiver, Sender};
use serde_json::Value;
use wiremock::{Match, Request};

#[derive(Clone)]
/// An impl of `wiremock::Match` that lets you see what requests were made
pub struct HttpSpy {
    receiver: Receiver<Request>,
    sender: Sender<Request>,
}

impl HttpSpy {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        HttpSpy { receiver, sender }
    }

    pub fn drain_requests(&self) -> Vec<Request> {
        self.receiver.try_iter().collect()
    }

    pub fn drain_json_bodies(&self) -> Vec<Value> {
        self.receiver
            .try_iter()
            .map(|request| request.body_json().expect("Expected JSON body"))
            .collect()
    }
}

impl Match for HttpSpy {
    fn matches(&self, request: &wiremock::Request) -> bool {
        self.sender.send(request.clone()).expect("channel to be open");

        true
    }
}
