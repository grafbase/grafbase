use super::super::consts::DATABASE_FILE;
use super::types::{EventRecord, Modification, StreamRecord};
use crate::bridge::consts::DATABASE_URL_PREFIX;
use crate::bridge::listener::consts::{
    CLI_API_KEY, DEFAULT_AWS_REGION, MODIFICATIONS_TABLE_NAME, MODIFICATION_POLL_INTERVAL, RECORDS_TABLE_NAME,
};
use crate::{
    errors::ServerError,
    event::{wait_for_event, Event},
};
use common::environment::Project;
use reqwest::Client;
use sqlx::{query, query_as, Connection, SqliteConnection};
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;
use uuid::Uuid;

async fn event_listener(worker_port: u16) -> Result<(), ServerError> {
    let project = Project::get();
    // existence is already guaranteed by the bridge server
    let database_file = project.database_directory_path.join(DATABASE_FILE);

    let database_url = match database_file.to_str() {
        Some(database_file) => format!("{DATABASE_URL_PREFIX}{database_file}"),
        None => return Err(ServerError::ProjectPath),
    };

    let mut connection = SqliteConnection::connect(&database_url).await?;

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
            Err(err) => {
                trace!("Failed to retrieve latest modifications with error: {:?}", err);
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

            let response = client
                .post(format!(
                    "http://127.0.0.1:{worker_port}/stream-router/main/dynamodb/{DEFAULT_AWS_REGION}"
                ))
                .header("x-api-key", CLI_API_KEY)
                .json(&dynamo_events)
                .send()
                .await
                // TODO: consider if this should panic or show a specific error to the user
                .expect("could not contact the stream router");
            trace!(
                "Sent update to stream-router, responded with status: {}",
                response.status()
            );
        }
    }
}

pub async fn start(worker_port: u16, event_bus: Sender<Event>) -> Result<(), ServerError> {
    trace!("starting db event listener");

    tokio::select! {
        _ = wait_for_event(event_bus.subscribe(), |event| matches!(event, Event::Reload(_))) => {}
        event_listener_result = event_listener(worker_port) => {  event_listener_result? }
    }

    Ok(())
}
