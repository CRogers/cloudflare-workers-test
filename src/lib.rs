use serde_json::json;
use worker::*;

mod chatroom;
mod utils;

#[event(fetch)]
pub async fn main(original_req: Request, env: Env) -> Result<Response> {
    utils::log_request(&original_req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .on_async("/chat/*any", |req, ctx| async move {
            chatroom(&ctx)?
                .fetch_with_request(Request::new_with_init(
                    &ctx.param("any").unwrap(),
                    &RequestInit {
                        body: req.inner().body().map(|body| body.into()),
                        headers: req.headers().to_owned(),
                        cf: Default::default(),
                        method: req.method(),
                        redirect: Default::default(),
                    },
                )?)
                .await
        })
        .run(original_req, env)
        .await
}

fn chatroom(ctx: &RouteContext<()>) -> Result<Stub> {
    let namespace = ctx.durable_object("CHATROOM")?;
    let stub = namespace.id_from_name("A")?.get_stub()?;
    Ok(stub)
}
