/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HANDLE, BOOL, CloseHandle};
use windows::Win32::System::Diagnostics::ToolHelp::{
  CreateToolhelp32Snapshot,
  TH32CS_SNAPPROCESS,
  PROCESSENTRY32W,
  Process32FirstW,
  Process32NextW,
};

pub struct Process32Iterator {
  hsnapshot: HANDLE,
  fn_next: unsafe fn(HANDLE, *mut PROCESSENTRY32W) -> BOOL,
}

impl Process32Iterator {
  pub fn new() -> Self {
    Process32Iterator {
      hsnapshot: unsafe {CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap()},
      fn_next: Process32FirstW,
    }
  }
}
impl Default for Process32Iterator {
  fn default() -> Self {
    Self::new()
  }
}
impl Drop for Process32Iterator {
  fn drop(&mut self) {
    unsafe {
      CloseHandle(self.hsnapshot);
    }
  }
}

impl Iterator for Process32Iterator {
  type Item = PROCESSENTRY32W;
  fn next(&mut self) -> Option<PROCESSENTRY32W> {
    let mut lppe = PROCESSENTRY32W {
      dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
      ..Default::default()
    };
    match unsafe {(self.fn_next)(self.hsnapshot, &mut lppe)}.as_bool() {
      false => None,
      true => {
        self.fn_next = Process32NextW;
        Some(lppe)
      },
    }
  }
}

pub type PidType = u32;
pub trait ProcessInfo {
  fn pid(&self) -> PidType;
  fn get_name(&self) -> OsString;
}
impl ProcessInfo for PROCESSENTRY32W {
  fn pid(&self) -> PidType {
    self.th32ProcessID as PidType
  }
  fn get_name(&self) -> OsString {
    let len = self.szExeFile.iter().position(|p| *p==0).unwrap_or(self.szExeFile.len());
    OsString::from_wide(&self.szExeFile[..len])
  }
}
