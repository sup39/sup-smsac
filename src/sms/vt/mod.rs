/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::addr::Addr;
use crate::sms::SMSVersion;
use sup_smsac_derive::match_class_from_json;

pub fn get_class(ver: SMSVersion, addr: Addr) -> Option<&'static str> {
  match ver {
    SMSVersion::GMSJ01 => match_class_from_json!("src/sms/vt/GMSJ01.json")(addr.0),
    SMSVersion::GMSE01 => match_class_from_json!("src/sms/vt/GMSE01.json")(addr.0),
    SMSVersion::GMSP01 => match_class_from_json!("src/sms/vt/GMSP01.json")(addr.0),
    SMSVersion::GMSJ0A => match_class_from_json!("src/sms/vt/GMSJ0A.json")(addr.0),
  }
}
pub fn get_class_string(ver: SMSVersion, addr: Addr) -> String {
  match get_class(ver, addr) {
    Some(s) => s.to_string(),
    None => format!("({addr})"),
  }
}
