/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0
///
/// The `DolphinProcessMemory::open_pid` function is based on
///   `WindowsDolphinProcess::obtainEmuRAMInformations()`
///     (https://github.com/aldelaro5/Dolphin-memory-engine/blob/master/Source/DolphinProcess/Windows/WindowsDolphinProcess.cpp#L47)
///   from aldelaro5's Dolphin memory engine
///     (https://github.com/aldelaro5/Dolphin-memory-engine)
/// SPDX-FileCopyrightText: 2017 aldelaro5
/// SPDX-License-Identifier: MIT

use super::{Dolphin, DolphinMemAddr, PidType};
use crate::sys::process_memory::ProcessMemoryIterator;
use core::ffi::c_void;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::{
  Threading::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_VM_WRITE, OpenProcess},
  Memory::MEM_MAPPED,
  Diagnostics::Debug::ReadProcessMemory,
};

pub struct DolphinProcessMemory {
  h_proc: HANDLE,
  base_addr_mem1: usize,
  base_addr_mem2: Option<usize>,
}
impl Drop for DolphinProcessMemory {
  fn drop(&mut self) {
    unsafe {
      CloseHandle(self.h_proc);
    }
  }
}
impl Dolphin for DolphinProcessMemory {
  unsafe fn operate_memory_unchecked<T, F>(&self, maddr: DolphinMemAddr, size: usize, operator: F) -> Option<T>
    where F: FnOnce(*mut u8) -> T
  {
    match maddr {
      DolphinMemAddr::MEM1(offset) => Some(self.base_addr_mem1 + (offset as usize)),
      DolphinMemAddr::MEM2(offset) => self.base_addr_mem2.map(|base_addr| base_addr + (offset as usize)),
    }.and_then(|base_addr| {
      let mut buf = Vec::with_capacity(size);
      let ptr = buf.as_mut_ptr();
      match unsafe {ReadProcessMemory(
        self.h_proc, base_addr as *const c_void,
        ptr as *mut c_void, size, None,
      ).as_bool()} {
        true => Some(operator(ptr)),
        false => None,
      }
    })
  }
}

pub enum DolphinProcessMemoryFindError {
  OpenError(windows::core::Error),
  MemoryNotFound,
}
impl From<windows::core::Error> for DolphinProcessMemoryFindError {
  fn from(e: windows::core::Error) -> Self {
    Self::OpenError(e)
  }
}
impl DolphinProcessMemory {
  pub fn open_pid(pid: PidType) -> Result<DolphinProcessMemory, DolphinProcessMemoryFindError> {
    unsafe {
      let h_proc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid)?;
      let mut itr = ProcessMemoryIterator::with_handle(h_proc);

      // find MEM1
      let Some(base_addr_mem1) =
        itr.find(|meminfo| meminfo.RegionSize == 0x2000000 && meminfo.Type == MEM_MAPPED)
          .map(|meminfo| meminfo.BaseAddress)
      else {
        CloseHandle(h_proc);
        return Err(DolphinProcessMemoryFindError::MemoryNotFound);
      };

      // find MEM2
      let base_addr_mem2_check = base_addr_mem1.add(0x10000000);
      let base_addr_mem2 =
        itr.find(|meminfo| {
          meminfo.BaseAddress == base_addr_mem2_check
            && meminfo.RegionSize == 0x4000000
            && meminfo.Type == MEM_MAPPED
        }).map(|_| base_addr_mem2_check as usize);

      Ok(DolphinProcessMemory {
        h_proc, base_addr_mem1: base_addr_mem1 as usize, base_addr_mem2,
      })
    }
  }
}
