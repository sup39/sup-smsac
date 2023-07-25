/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi::c_void;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::{
  Threading::{PROCESS_QUERY_INFORMATION, OpenProcess},
  Memory::{MEMORY_BASIC_INFORMATION, VirtualQueryEx},
};

pub struct ProcessMemoryIterator {
  h_proc: HANDLE,
  addr: *const c_void,
  should_drop: bool,
}
impl ProcessMemoryIterator {
  pub fn with_handle(h_proc: HANDLE) -> Self {
    Self {h_proc, addr: std::ptr::null(), should_drop: false}
  }
  pub fn try_new(pid: u32) -> Result<Self, windows::core::Error> {
    let h_proc = unsafe {
      OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)?
    };
    Ok(Self {h_proc, addr: std::ptr::null(), should_drop: true})
  }
}
impl Drop for ProcessMemoryIterator {
  fn drop(&mut self) {
    unsafe {
      if self.should_drop {
        CloseHandle(self.h_proc);
      }
    }
  }
}
impl Iterator for ProcessMemoryIterator {
  type Item = MEMORY_BASIC_INFORMATION;
  fn next(&mut self) -> Option<MEMORY_BASIC_INFORMATION> {
    let mut meminfo = MEMORY_BASIC_INFORMATION::default();
    unsafe {match {
      VirtualQueryEx(self.h_proc, Some(self.addr), &mut meminfo, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) > 0
    } {
      true => {
        self.addr = self.addr.add(meminfo.RegionSize);
        Some(meminfo)
      },
      false => None,
    }}
  }
}
