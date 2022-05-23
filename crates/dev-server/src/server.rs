use actix_web::{middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use common::consts::LOCALHOST;
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::thread;

pub fn start(port: u16) -> thread::JoinHandle<()> {
    trace!("spawining server thread");
    thread::spawn(move || {
        trace!("server thread id: {:?}", thread::current().id());
        HttpResponse::Ok().body("Hello world!");
        actix_main(port).unwrap();
    })
}

#[actix_web::main]
async fn actix_main(port: u16) -> std::io::Result<()> {
    trace!("running server on port {}", port);
    HttpServer::new(|| App::new().wrap(Logger::default()).service(root))
        .bind((LOCALHOST, port))?
        .run()
        .await
}

#[post("/")]
async fn root(_request: web::Json<GraphqlRequest>) -> actix_web::Result<impl Responder> {
    Ok(web::Json(GraphqlResponse {
        data: serde_json::Value::Object(Map::new()),
        errors: Value::Array(vec![]),
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct GraphqlRequest {
    query: String,
    operation_name: String,
    variables: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct GraphqlResponse {
    data: Value,
    errors: Value,
}
