/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use hyper::{Body, Request, Response, StatusCode};
use hyper_tungstenite::tungstenite;
use urlencoding;
use mime_guess;
use crate::{
  sms::SMSDolphin,
  obj_params::{load_obj_params, ObjParamsLoadResult},
  server::ws::serve_websocket,
};

pub struct HttpEnv {
  static_dir: Box<Path>,
  pub obj_params_dir: Box<Path>,
  pub obj_params_result: Mutex<ObjParamsLoadResult<SMSDolphin>>,
}

pub async fn serve(listener: TcpListener, root_dir: Box<Path>) -> Result<(), tungstenite::Error> {
  let obj_params_dir = {
    let mut dir = root_dir.to_path_buf();
    dir.push("res/ObjectParameters");
    dir.into_boxed_path()
  };
  let obj_params_result = Mutex::new(load_obj_params(&obj_params_dir));

  let env = Arc::new(HttpEnv {
    static_dir: {
      let mut static_dir = root_dir.to_path_buf();
      static_dir.push("www");
      static_dir.into_boxed_path()
    },
    obj_params_dir,
    obj_params_result,
  });

  let http = hyper::server::conn::Http::new();
  loop {
    let env = env.clone();
    let (stream, _) = listener.accept().await?;
    let connection = http
      .serve_connection(stream, hyper::service::service_fn(move |req| handle_request(req, env.clone())))
      .with_upgrades();
    tokio::spawn(async move {
      if let Err(err) = connection.await {
        println!("Error serving HTTP connection: {:?}", err);
      }
    });
  }
}

#[inline]
fn response_text<T>(status: T, e: &dyn std::fmt::Display) -> Response<Body>
where
  StatusCode: TryFrom<T>,
  <StatusCode as TryFrom<T>>::Error: Into<hyper::http::Error>,
{
  Response::builder()
    .status(status)
    .header("Content-Type", "text/plain; charset=utf-8")
    .body(Body::from(format!("{}", e)))
    .unwrap()
}

async fn handle_request(
  mut req: Request<Body>,
  env: Arc<HttpEnv>,
) -> Result<Response<Body>, tungstenite::Error> {
  let is_upgrade = hyper_tungstenite::is_upgrade_request(&req);

  let mut lock_obj_params = env.obj_params_result.lock().await;
  if lock_obj_params.is_err() {
    *lock_obj_params = load_obj_params(&env.obj_params_dir);
    if let Err(e) = &*lock_obj_params {
      return Ok(response_text(500, &format!(
        "Fail to load ObjectParameters at {}: {e}",
        env.obj_params_dir.to_string_lossy(),
      )));
    }
  }

  if is_upgrade {
    let (res, ws) = hyper_tungstenite::upgrade(&mut req, None)?;
    let env = env.clone();
    tokio::spawn(async move {
      if let Err(e) = serve_websocket(ws, env).await {
        eprintln!("Error in websocket connection: {}", e);
      }
    });
    Ok(res)
  } else {
    let url_path = match urlencoding::decode(req.uri().path()) {
      Ok(p) if p.starts_with('/') => p,
      _ => return Ok(Response::builder().status(400).body(Body::empty()).unwrap()),
    };

    let mut file_path = PathBuf::new();
    file_path.push(&*env.static_dir);
    file_path.push(if url_path.ends_with('/') {
      "index.html"
    } else {
      &url_path[1..]
    });
    Ok(serve_file(&file_path).await)
  }
}

async fn serve_file(path: &Path) -> Response<Body> {
  match File::open(path).await {
    Err(err) => response_text(404, &match path.canonicalize() {
      Ok(e) => format!("\"{}\" not found: {err}", e.to_string_lossy()),
      Err(_) => {
        match std::env::current_dir() {
          Ok(mut cwd) => {
            cwd.push(path);
            format!("\"{}\" not found: {err}", cwd.to_string_lossy())
          },
          Err(_) => format!("\"{}\" not found: {err}", path.to_string_lossy()),
        }
      },
    }),
    Ok(file) => {
      let mut res = Response::builder();
      if let Some(mime) = mime_guess::from_path(path).first() {
        res = res.header("Content-Type", format!("{mime}; charset=utf-8"));
      }
      let stream = FramedRead::new(file, BytesCodec::new());
      res.body(Body::wrap_stream(stream)).unwrap()
    },
  }
}
