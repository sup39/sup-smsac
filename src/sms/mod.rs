/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::addr::Addr;
use crate::dolphin::{DolphinMemory, Dolphin, DolphinMemAddr, PidType};

#[derive(Debug, Clone, Copy)]
pub enum SMSVersion {
  GMSJ01, GMSE01, GMSP01, GMSJ0A,
}
impl std::fmt::Display for SMSVersion {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    std::fmt::Debug::fmt(self, fmt)
  }
}

pub struct SMSDolphin {
  d: DolphinMemory,
  pid: PidType,
  ver: SMSVersion,
}
impl Dolphin for SMSDolphin {
  unsafe fn read_memory_unchecked<T, F>(&self, maddr: DolphinMemAddr, size: usize, operator: F) -> Option<T>
    where F: FnOnce(*const u8) -> T
  {
    self.d.read_memory_unchecked(maddr, size, operator)
  }
  unsafe fn write_memory_unchecked(&self, maddr: DolphinMemAddr, payload: &[u8]) -> Option<()> {
    self.d.write_memory_unchecked(maddr, payload)
  }
}

pub mod vt;
impl SMSDolphin {
  #[inline]
  pub fn ver(&self) -> SMSVersion {
    self.ver
  }

  pub fn from_dolphin_memory(d: DolphinMemory, pid: PidType) -> Result<SMSDolphin, Option<[u8; 8]>> {
    unsafe {
      d.read_memory_unchecked(DolphinMemAddr::MEM1(0), 8, |ptr| {
        match &*(ptr as *const [u8; 8]) {
          b"GMSJ01\x00\x00" => Ok(SMSVersion::GMSJ01),
          b"GMSE01\x00\x30" => Ok(SMSVersion::GMSE01),
          b"GMSP01\x00\x00" => Ok(SMSVersion::GMSP01),
          b"GMSJ01\x00\x01" => Ok(SMSVersion::GMSJ0A),
          rver => Err(Some(rver.to_owned())),
        }
      })
    } .unwrap_or(Err(None))
      .map(|ver| SMSDolphin {d, ver, pid})
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
  pub fn pid(&self) -> PidType {
    self.pid
  }

  pub fn find_one() -> Result<SMSDolphin, SMSDolphinFindOneError> {
    let mut dolphin_running = false;
    let mut game_running = false;
    for (pid, d) in DolphinMemory::list() {
      match d {
        Some(d) => {
          match SMSDolphin::from_dolphin_memory(d, pid) {
            Ok(o) => return Ok(o),
            Err(e) => {
              game_running = true;
              match e {
                Some(e) => eprintln!("Unknown game (pid: {pid}): {}",
                  e.map(|c| format!("{c:02X}")).join("")),
                None => eprintln!("Unknown game (pid: {pid}): fail to get version"),
              }
            }
          }
        },
        None => dolphin_running = true,
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
