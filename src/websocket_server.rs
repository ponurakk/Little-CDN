use std::{time::{Duration, Instant}, sync::{Arc, Mutex}, fmt::Display};

use actix::prelude::*;
use actix_web::{HttpRequest, http::StatusCode};
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{ApiError, error::WebSocketError};

const HEARBEAT_INTERVAL: Duration = Duration::from_secs(10);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Serialize, Deserialize, Debug)]
enum RequestType {
    Login,
    Logout,
    File,
    Sync,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBody {
    req_type: RequestType,
    data: Value,
}

#[derive(Serialize, Debug)]
struct ResponseBody {
    code: u16,
    message: String,
    data: Option<Value>,
}

impl Display for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
    }
}

impl ResponseBody {
    fn new() -> Self {
        Self {
            code: StatusCode::OK.as_u16(),
            message: String::new(),
            data: None,
        }
    }
}

#[derive(Debug)]
pub struct WebSocket {
    hb: Instant,
    req: HttpRequest
}

impl WebSocket {
    #[must_use]
    pub fn new(req: HttpRequest) -> Self {
        Self {
            hb: Instant::now(),
            req
        }
    }

    #[allow(clippy::unused_self)]
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::warn!(target: "Little CDN", "Websocket Client heartbeat failed, disconnecting!");
                
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!(target: "Little CDN", "Recieved new connection");
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if text.len() == 0 {
                    return;
                }
                let command_res: Result<RequestBody, serde_json::Error> = serde_json::from_str(&text.to_string());
                let command: RequestBody;
                match command_res {
                    Ok(v) => command = v,
                    Err(e) => {
                        return ctx.text(format!("{:?}", e));
                    }
                };
                let request = self.req.clone();
                async move {
                    // let res_command = handle_command(cmd.clone(), args.clone(), request).await;
                    let res_command = handler(command);

                    let response = match res_command {
                        Ok(v) => {
                            ResponseBody {
                                code: StatusCode::OK.as_u16(),
                                message: String::from("Ok"),
                                data: Some(v),
                            }
                        },
                        Err(e) => {
                            ResponseBody {
                                code: e.code(),
                                message: e.to_string(),
                                data: None,
                            }
                        }
                    };
                    response
                }.into_actor(self).map(move |res, _, ctx| {
                    ctx.text(res.to_string());
                }).wait(ctx);
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        } 
    }
}

fn handler(command: RequestBody) -> Result<Value, ApiError> {
    ret_ws_err()?;
    Ok(serde_json::to_value(command)?)
}

fn ret_ws_err() -> Result<String, WebSocketError> {
    Err(WebSocketError::LoginError("Test function"))
}
