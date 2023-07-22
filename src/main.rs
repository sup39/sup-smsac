/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod dolphin;
pub mod addr;
pub mod big_endian;
pub mod sys;
pub mod sms;
pub mod server;
pub mod obj_params;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
struct Args {
  #[arg(long, default_value_t = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))]
  host: IpAddr,

  #[arg(short='p', long, default_value_t = 35353)]
  port: u16,

  #[arg(long)]
  no_browser: bool,

  #[arg(short='d', long)]
  root_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();

  let listener = {
    let mut sock_addr = SocketAddr::new(args.host, args.port);
    match tokio::net::TcpListener::bind(&sock_addr).await {
      Ok(listener) => {
        listener
      },
      Err(err) => {
        println!("Failed to listen on {sock_addr}: {err}\nTrying other port...\n");
        sock_addr.set_port(0);
        tokio::net::TcpListener::bind(&sock_addr).await.unwrap()
      },
    }
  };

  let url = format!("http://{}", listener.local_addr().unwrap());
  println!("Listening on {url}");
  if !args.no_browser {
    let _ = open::that(url);
  }

  let root_dir = args.root_dir
    .unwrap_or_else(|| {
      let mut path = std::env::current_exe().unwrap();
      path.pop();
      path
    }).canonicalize().unwrap().into_boxed_path();

  server::http::serve(listener, root_dir).await.unwrap();
}
