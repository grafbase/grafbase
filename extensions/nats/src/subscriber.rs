use std::{cell::RefCell, rc::Rc};

use grafbase_sdk::{
    host_io::pubsub::{
        nats::{self, NatsSubscriber},
        Subscriber,
    },
    jq_selection::JqSelection,
    types::FieldOutput,
    Error,
};

pub struct FilteringSubscriber {
    nats: nats::NatsSubscriber,
    jq_selection: Rc<RefCell<JqSelection>>,
    selection: Option<String>,
}

impl FilteringSubscriber {
    pub fn new(nats: NatsSubscriber, jq_selection: Rc<RefCell<JqSelection>>, selection: Option<String>) -> Self {
        Self {
            nats,
            jq_selection,
            selection,
        }
    }
}

impl Subscriber for FilteringSubscriber {
    fn next(&mut self) -> Result<Option<FieldOutput>, Error> {
        let item = match self.nats.next() {
            Some(item) => item,
            None => return Ok(None),
        };

        let mut field_output = FieldOutput::default();

        let payload: serde_json::Value = item.payload().map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Error parsing NATS value as JSON: {e}"),
        })?;

        match self.selection {
            Some(ref selection) => {
                let mut jq = self.jq_selection.borrow_mut();

                let filtered = jq.select(selection, payload).map_err(|e| Error {
                    extensions: Vec::new(),
                    message: format!("Failed to filter with selection: {e}"),
                })?;

                for payload in filtered {
                    match payload {
                        Ok(payload) => field_output.push_value(payload),
                        Err(error) => field_output.push_error(Error {
                            extensions: Vec::new(),
                            message: format!("Error parsing result value: {error}"),
                        }),
                    }
                }
            }
            None => {
                field_output.push_value(payload);
            }
        };

        Ok(Some(field_output))
    }
}
