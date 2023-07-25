/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
  addr::{Addr, AddrOffsets},
  dolphin::Dolphin,
  sms::{SMSDolphin, SMSVersion},
  big_endian::DecodeBE,
  server::http::HttpEnv,
  obj_params::{load_obj_params, ObjectType},
};
use sup_smsac_derive::DecodeBE;
use serde_json::{self, json, Value as JsonValue};

#[derive(Debug, DecodeBE)]
struct ConductorNode {
  next: Addr,
  _prev: Addr,
  obj: Addr,
}
#[derive(Debug, DecodeBE)]
struct ChildInfo {
  count: u32,
  addr: Addr,
}

trait DolphinMemoryJsExt {
  fn resolve_addr(&self, addr: &JsonValue) -> Result<Option<Addr>, ()>;
  fn resolve_addr_offsets(&self, base: Addr, offsets: &AddrOffsets) -> Option<Addr>;
}
impl<T: Dolphin> DolphinMemoryJsExt for T {
  fn resolve_addr(&self, addr: &JsonValue) -> Result<Option<Addr>, ()> {
    // single addr
    if let Some(addr) = addr.as_u64() {
      return Ok(Some(Addr(addr as u32)));
    }
    // addr + offsets
    let Some((Some(mut addr), offs)) = addr.as_array()
      .and_then(|x| x.split_first())
      .map(|e| (e.0.as_u64().map(|x| Addr(x as u32)), e.1))
    else {return Err(())};
    // resolve
    for off in offs {
      let Some(off) = off.as_i64().map(|x| x as u32) else {
        return Err(());
      };
      match self.read::<Addr>(addr) {
        None => return Ok(None),
        Some(_addr) => addr = _addr+off,
      }
    }
    Ok(Some(addr))
  }
  fn resolve_addr_offsets(&self, base: Addr, offsets: &AddrOffsets) -> Option<Addr> {
    let mut addr = base + offsets.0;
    for off in offsets.1.iter() {
      match self.read::<Addr>(addr) {
        None => return None,
        Some(_addr) => addr = _addr + *off,
      }
    }
    Some(addr)
  }
}

pub async fn handle_command(
  env: &HttpEnv,
  dolphin: &mut Option<SMSDolphin>,
  command: &str,
  body: &JsonValue,
) -> Result<JsonValue, JsonValue> {
  macro_rules! return_err {
    ($($msg:expr),+) => {
      return Err(json!(format!($($msg),+)))
    }
  }
  macro_rules! let_dolphin {
    ($d: ident) => {
      let $d = match &dolphin {
        Some(d) => d,
        None => match SMSDolphin::find_one() {
          Ok(d) => {
            *dolphin = Some(d);
            match &dolphin {
              Some(d) => d,
              None => unreachable!(),
            }
          },
          Err(e) => return_err!("{}", e),
        },
      };
    };
  }

  macro_rules! let_obj_params_fields {
    ($fields:ident, $type: ident) => {
      let lock_obj_params = env.obj_params_result.lock().await;
      let obj_params = match &*lock_obj_params {
        Ok(v) => v,
        Err(e) => return_err!("Fail to get ObjectParameters: {e}"),
      };
      let Some($fields) = obj_params.get($type).or_else(|| obj_params.get("_default")) else {
        return_err!("unknown type: \"{}\". Please defined \"_default\" type in ObjectParameters/*.json", $type);
      };
    };
  }

  match command {
    "init" => {
      let_dolphin!(d);
      Ok(json!(d.pid()))
    },

    "getManagers" => {
      let_dolphin!(d);
      Ok(d.read::<Addr>(Addr(match d.ver() {
        // TODO put addr in external file
        SMSVersion::GMSJ01 => 0x8040A6E8,
        SMSVersion::GMSE01 => 0x8040D110,
        SMSVersion::GMSP01 => 0x80404870,
        SMSVersion::GMSJ0A => 0x803FE048,
      }))
        .and_then(|a| d.read::<ChildInfo>(a+0x14))
        .and_then(|o| {
          let mut next = o.addr;
          let mut arr: Vec<JsonValue> = Vec::with_capacity(o.count as usize);
          for _i in 0..o.count {
            let Some(node) = d.read::<ConductorNode>(next) else {return None};
            arr.push(json!([
              node.obj.0,
              d.read::<Addr>(node.obj)
                .map(|a| d.get_class_string(a))
                .unwrap_or_else(|| "({a})".to_string()),
              d.read::<Addr>(node.obj+4).and_then(|a| d.read_str(a)).unwrap_or_else(|| "�".to_string()),
              d.read::<i32>(node.obj+0x14).unwrap_or(-1),
            ]));
            next = node.next;
          }
          Some(JsonValue::Array(arr))
        }).unwrap_or_else(|| json!(null))
      )
    },

    "getManagees" => {
      let_dolphin!(d);
      let Some(addr) = body.as_u64().map(|x| Addr(x as u32)) else {
        return_err!("\"body\" must be a string");
      };
      Ok(d.read::<ChildInfo>(addr+0x14).and_then(|o| {
        let mut arr: Vec<JsonValue> = Vec::with_capacity(o.count as usize);
        for a in (0..o.count).map(|i| d.read::<Addr>(o.addr+4*i)) {
          let Some(a) = a else {return None};
          arr.push(json!([
            a.0,
            d.read::<Addr>(a).map(|a| d.get_class_string(a)),
            d.read::<Addr>(a+4).and_then(|a| d.read_str(a)).unwrap_or_else(|| "�".to_string()),
          ]));
        }
        Some(JsonValue::Array(arr))
      }).unwrap_or_else(|| json!(null)))
    },

    "read" => {
      let_dolphin!(d);
      let Some(addr) = body.get("addr") else {
        return_err!("addr must be specified");
      };
      let Ok(addr) = d.resolve_addr(addr) else {
        return_err!("invalid addr: {addr:?}");
      };
      let Some(addr) = addr else {
        return Ok(json!(null));
      };

      Ok(match body.get("size") {
        Some(size) => {
          if body.get("type").is_some() {
            return_err!("\"size\" and \"type\" cannot be specified at the same time");
          }
          let Some(size) = size.as_u64().map(|x| x as usize) else {
            return_err!("\"size\" must be a positive integer");
          };
          d.dump_hex(addr, size)
            .map(|s| json!(s))
            .unwrap_or_else(|| json!(null))
        },
        None => {
          let Some(type_) = body.get("type") else {
            return_err!("either \"size\" and \"type\" must be specified");
          };
          let Some(type_) = type_.as_str() else {
            return_err!("\"type\" must be a string");
          };
          let_obj_params_fields!(fields, type_);
          match fields {
            ObjectType::Primitive(p) => p.read(d, addr)
              .map(|x| json!(x))
              .unwrap_or_else(|| json!(null)),
            ObjectType::Class(fields) => JsonValue::Array(fields.iter().map(|field| {
              d.resolve_addr_offsets(addr, &field.offset)
                .map(|addr| field.reader.read(d, addr))
                .map(|x| json!(x))
                .unwrap_or_else(|| json!(null))
            }).collect()),
          }
        },
      })
    },

    "readString" => {
      let_dolphin!(d);
      let Ok(addr) = body.get("addr").ok_or(()).and_then(|o| d.resolve_addr(o)) else {
        return_err!("Invalid body: {body:?}");
      };
      Ok(addr
        .and_then(|addr| d.read_str(addr))
        .map(|addr| json!(addr))
        .unwrap_or_else(|| json!(null)))
    },

    "write" => {
      let_dolphin!(d);
      let (Ok(addr), Ok(payload)) = (
        body.get("addr").ok_or(()).and_then(|o| d.resolve_addr(o)),
        body.get("payload")
          .and_then(|x| x.as_str()).ok_or(())
          .and_then(|s| (0..s.len()).step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i+2], 16).map_err(|_| ()))
            .collect::<Result<Vec<u8>, ()>>()
          ),
      ) else {
        return_err!("Invalid body: {body:?}");
      };
      Ok(json!(
        addr.and_then(|addr| d.write_bytes(addr, &payload)).is_some()
      ))
    },

    "getClass" => {
      let_dolphin!(d);
      let Ok(addr) = body.get("addr").ok_or(()).and_then(|o| d.resolve_addr(o)) else {
        return_err!("Invalid body: {body:?}");
      };
      Ok(addr
        .and_then(|addr| d.read::<Addr>(addr))
        .map(|a| json!(d.get_class_string(a)))
        .unwrap_or_else(|| json!(null)))
    },

    "getFields" => {
      let Some(type_) = body.as_str() else {
        return_err!("body must be a string");
      };
      let_obj_params_fields!(fields, type_);
      Ok(match fields {
        // [offsets, name, value, notes, type, class]
        ObjectType::Primitive(_) =>
          json!([["0", "value", "", type_, type_]]),
        ObjectType::Class(fields) => JsonValue::Array(fields.iter().map(|r| json!([
          r.offset.to_string(), r.name, r.notes, r.type_, r.class,
        ])).collect()),
      })
    },

    "getVersion" => {
      let_dolphin!(d);
      Ok(json!(d.ver().to_string()))
    },

    "reload" => {
      let mut lock_obj_params = env.obj_params_result.lock().await;
      load_obj_params(&env.obj_params_dir)
        .map(|db| {
          *lock_obj_params = Ok(db);
          json!(null)
        })
        .map_err(|e| json!(e.to_string()))
    },

    _ => {
      return_err!("Unknown command: {command}")
    },
  }
}
