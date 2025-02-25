use std::{cell::RefCell, rc::Rc};

use grafbase_sdk::{
    host_io::pubsub::nats::{self, NatsSubscription},
    jq_selection::JqSelection,
    types::FieldOutput,
    Error, Subscription,
};

pub struct FilteredSubscription {
    nats: nats::NatsSubscription,
    jq_selection: Rc<RefCell<JqSelection>>,
    selection: Option<String>,
}

impl FilteredSubscription {
    pub fn new(nats: NatsSubscription, jq_selection: Rc<RefCell<JqSelection>>, selection: Option<String>) -> Self {
        Self {
            nats,
            jq_selection,
            selection,
        }
    }
}

impl Subscription for FilteredSubscription {
    fn next(&mut self) -> Result<Option<FieldOutput>, Error> {
        let item = match self.nats.next() {
            Ok(Some(item)) => item,
            Ok(None) => return Ok(None),
            Err(e) => return Err(format!("Failed to receive message from NATS: {e}").into()),
        };

        let mut field_output = FieldOutput::default();

        let payload: serde_json::Value = item
            .payload()
            .map_err(|e| format!("Error parsing NATS value as JSON: {e}"))?;

        match self.selection {
            Some(ref selection) => {
                let mut jq = self.jq_selection.borrow_mut();

                let filtered = jq
                    .select(selection, payload)
                    .map_err(|e| format!("Failed to filter with selection: {e}"))?;

                for payload in filtered {
                    match payload {
                        Ok(payload) => field_output.push_value(payload),
                        Err(error) => field_output.push_error(format!("Error parsing result value: {error}")),
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
