/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
  sms::SMSDolphin,
  server::{http::HttpEnv, api::handle_command},
};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::{tungstenite::{self, Message}, HyperWebsocket};
use serde_json::{self, json, Value as JsonValue};

pub async fn serve_websocket(
  ws: HyperWebsocket,
  env: Arc<HttpEnv>,
) -> Result<(), tungstenite::Error> {
  let mut ws = ws.await?;
  let mut dolphin: Option<SMSDolphin> = None;
  macro_rules! return_err {
    ($($msg:expr),+) => {
      eprintln!($($msg),+);
      return None;
    }
  }

  while let Some(msg) = ws.next().await {
    let Ok(msg) = msg else {continue};
    if let Some(res) = (|| async {match msg {
      Message::Text(payload) => {
        let Ok(payload) = serde_json::from_str::<JsonValue>(&payload) else {
          eprintln!("Invalid payload (failed to deserialize): {payload}");
          return None;
        };
        let Some((Some(id), Some(command), body)) = payload.as_array()
          .and_then(|v| if v.len() == 3 {Some(v)} else {None})
          .map(|args| (
            // id must be positive
            args[0].as_i64().and_then(|x| if x<=0 {None} else {Some(x)}),
            args[1].as_str(),
            &args[2],
          ))
        else {
          return_err!("Invalid payload (invalid format): {payload}");
        };

        match handle_command(&env, &mut dolphin, command, body).await {
          Ok(body) => Some(json!([id, body])),
          Err(msg) => Some(json!([-id, msg])),
        }
      },
      Message::Binary(payload) => {
        Some(json!(format!("{}", payload.len())))
      },
      _ => None,
    }})().await {
      if let Err(e) = ws.send(Message::Text(res.to_string())).await {
        eprintln!("Fail to send message: {e}");
      }
    }
  }

  Ok(())
}
