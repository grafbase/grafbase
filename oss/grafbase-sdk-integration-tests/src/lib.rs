mod kv;

use worker::{event, Context, Env, Request, Response, Result, Router};

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/kv/:key", kv::get)
        .get_async("/kv/:key/metadata", kv::get_metadata)
        .get_async("/kv", kv::list)
        .post_async("/kv/:key", kv::put)
        .post_async("/kv/:key/metadata", kv::put_metadata)
        .delete_async("/kv/:key", kv::delete)
        .run(req, env)
        .await
}
