use super::super::consts::DB_FILE;
use super::types::{EventRecord, Modification, StreamRecord};
use crate::bridge::consts::DB_URL_PREFIX;
use crate::bridge::listener::consts::{
    CLI_API_KEY, DEFAULT_AWS_REGION, MODIFICATIONS_TABLE_NAME, MODIFICATION_POLL_INTERVAL, RECORDS_TABLE_NAME,
};
use crate::{
    errors::ServerError,
    event::{wait_for_event, Event},
};
use common::environment::Environment;
use reqwest::Client;
use sqlx::{query, query_as, Connection, SqliteConnection};
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;
use uuid::Uuid;

async fn event_listener(worker_port: u16) -> Result<(), ServerError> {
    let environment = Environment::get();
    let db_file = environment.project_dot_grafbase_path.join(DB_FILE);

    let db_url = match db_file.to_str() {
        Some(db_file) => format!("{DB_URL_PREFIX}{db_file}"),
        None => return Err(ServerError::ProjectPath),
    };

    let mut connection = SqliteConnection::connect(&db_url).await?;

    let client = Client::new();

    let clean_modifications_table = format!("DELETE FROM {MODIFICATIONS_TABLE_NAME}");

    // clean the modifications table to prevent old events
    // firing before miniflare has started
    // (any existing modifications as there will be no listening live query)
    trace!("cleaning modifications");

    query(&clean_modifications_table).execute(&mut connection).await?;

    loop {
        sleep(MODIFICATION_POLL_INTERVAL).await;

        let delete_and_return_modifications = format!("DELETE FROM {MODIFICATIONS_TABLE_NAME} RETURNING *");

        let modifications = query_as::<_, Modification>(&delete_and_return_modifications);

        let results = match modifications.fetch_all(&mut connection).await {
            Ok(results) => results,
            // retry on the next interval if the DB is busy (due to a trigger writing an update)
            Err(_) => {
                // TODO: narrow this
                continue;
            }
        };
        if !results.is_empty() {
            let dynamo_events = results
                .iter()
                .map(|result| EventRecord {
                    aws_region: DEFAULT_AWS_REGION.to_owned(),
                    change: StreamRecord {
                        approximate_creation_date_time: result.approximate_creation_date_time,
                        keys: result.to_keys(),
                        new_image: result.document_new.clone().unwrap_or_default(),
                        old_image: result.document_old.clone().unwrap_or_default(),
                        // unused by the stream router
                        size_bytes: 0,
                    },
                    event_id: Uuid::new_v4().to_string(),
                    event_name: result.to_event_name().to_owned(),
                    event_source_arn: Some(RECORDS_TABLE_NAME.to_owned()),
                })
                .collect::<Vec<_>>();

            client
                .post(format!("http://127.0.0.1:{worker_port}/stream-router/main/dynamodb"))
                .header("x-api-key", CLI_API_KEY)
                .json(&dynamo_events)
                .send()
                .await
                // TODO: consider if this should panic or show a specific error to the user
                .expect("could not contact the stream router");

            trace!("sent update to stream-router");
        }
    }
}

pub async fn start(worker_port: u16, event_bus: Sender<Event>) -> Result<(), ServerError> {
    trace!("starting db event listener");

    tokio::select! {
        _ = wait_for_event(event_bus.subscribe(), Event::Reload) => {}
        event_listener_result = event_listener(worker_port) => {  event_listener_result? }
    }

    Ok(())
}
