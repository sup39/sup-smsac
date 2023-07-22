/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

#[derive(Debug, Clone, Copy)]
pub enum SMSVersion {
  GMSJ01, GMSE01, GMSP01, GMSJ0A,
}
pub mod vt;

use crate::addr::Addr;
use crate::dolphin::{DolphinMemory, Dolphin};
pub struct SMSDolphin {
  d: DolphinMemory,
  ver: SMSVersion,
}
impl Dolphin for SMSDolphin {
  #[inline]
  unsafe fn mem<T: Into<isize>>(&self, offset: T) -> *mut u8 {
    self.d.mem(offset)
  }
}

impl SMSDolphin {
  #[inline]
  pub fn pid(&self) -> usize {
    self.d.pid()
  }
  #[inline]
  pub fn ver(&self) -> SMSVersion {
    self.ver
  }

  pub fn from_dolphin_memory(d: DolphinMemory) -> Result<SMSDolphin, Option<[u8; 8]>> {
    match d.read::<&[u8; 8]>(Addr(0x80000000)) {
      None => Err(None),
      Some(rver) => match rver {
        b"GMSJ01\x00\x00" => Ok(SMSVersion::GMSJ01),
        b"GMSE01\x00\x30" => Ok(SMSVersion::GMSE01),
        b"GMSP01\x00\x00" => Ok(SMSVersion::GMSP01),
        b"GMSJ01\x00\x01" => Ok(SMSVersion::GMSJ0A),
        _ => Err(Some(rver.to_owned())),
      }.map(|ver| SMSDolphin {d, ver}),
    }
  }
  pub fn get_class(&self, addr: Addr) -> Option<&'static str> {
    vt::get_class(self.ver, addr)
  }
  pub fn get_class_string(&self, addr: Addr) -> String {
    vt::get_class_string(self.ver, addr)
  }
}

#[derive(Debug)]
pub enum SMSDolphinFindOneError {
  DolphinNotRunning,
  NoGameRunning,
  SMSNotRunning,
}
impl std::fmt::Display for SMSDolphinFindOneError {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      SMSDolphinFindOneError::DolphinNotRunning => write!(fmt, "Dolphin is not running"),
      SMSDolphinFindOneError::NoGameRunning => write!(fmt, "Dolphin is found, but no game is running"),
      SMSDolphinFindOneError::SMSNotRunning => write!(fmt, "SMS is not running"),
    }
  }
}

impl SMSDolphin {
  pub fn find_one() -> Result<SMSDolphin, SMSDolphinFindOneError> {
    let mut dolphin_running = false;
    let mut game_running = false;
    for (_pid, d) in DolphinMemory::list_dolphin() {
      match d {
        Ok(d) => {
          if let Ok(o) = SMSDolphin::from_dolphin_memory(d) {
            return Ok(o)
          }
          game_running = true;
        },
        Err(_) => dolphin_running = true,
      }
    }
    Err(if game_running {
      SMSDolphinFindOneError::SMSNotRunning
    } else if dolphin_running {
      SMSDolphinFindOneError::NoGameRunning
    } else {
      SMSDolphinFindOneError::DolphinNotRunning
    })
  }
}