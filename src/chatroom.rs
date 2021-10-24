use crate::utils;
use std::str;
use worker::wasm_bindgen::UnwrapThrowExt;
use worker::*;

#[durable_object]
pub struct Chatroom {
    messages: Vec<String>,
    number_of_requests: u32,
    useless_data: Vec<u8>,
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        utils::set_panic_hook();

        Self {
            messages: vec![],
            number_of_requests: 0,
            useless_data: vec![0; 10 * 1024 * 1024],
            state,
            env,
        }
    }

    async fn fetch(&mut self, mut req: Request) -> worker::Result<Response> {
        self.number_of_requests += 1;
        match req.method() {
            Method::Get => match req.path().as_str() {
                "/messages" => Response::ok(&format!(
                    "{} messages, {} requests:\n\n{}",
                    self.messages.len(),
                    self.number_of_requests,
                    self.messages.join("\n")
                )),
                other => Response::error(&format!("Unsupported path {}", other), 400),
            },
            Method::Post => match req.path().as_str() {
                "/message" => {
                    let message = str::from_utf8(&req.bytes().await?).unwrap().to_owned();
                    let response_message = &format!("Added message: {}", &message);
                    self.messages.push(message);
                    Response::ok(response_message)
                }
                other => Response::error("bleh", 400),
            },
            other => Response::error(&format!("Unsupported method {}", other.to_string()), 400),
        }
    }
}
