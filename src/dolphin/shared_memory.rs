/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::sys::process::PidType;
use crate::sys::shared_memory::{SharedMemory, SharedMemoryOpenError};
use crate::dolphin::{Dolphin, DolphinMemAddr, addr::MEM2_SIZE};

pub struct DolphinSharedMemory {
  shared_memory: SharedMemory,
  has_mem2: bool,
}

pub const MEM2_OFFSET: u32 = 0x4040000;
impl Dolphin for DolphinSharedMemory {
  unsafe fn read_memory_unchecked<T, F>(&self, maddr: DolphinMemAddr, _size: usize, operator: F) -> Option<T>
    where F: FnOnce(*const u8) -> T
  {
    match maddr {
      DolphinMemAddr::MEM1(offset) => Some(offset),
      DolphinMemAddr::MEM2(offset) => match self.has_mem2 {
        true => Some(MEM2_OFFSET + offset),
        false => None,
      },
    }.map(|offset| {
      operator(self.shared_memory.get_ptr().add(offset as usize))
    })
  }

  unsafe fn write_memory_unchecked(&self, maddr: DolphinMemAddr, payload: &[u8]) -> Option<()> {
    match maddr {
      DolphinMemAddr::MEM1(offset) => Some(offset),
      DolphinMemAddr::MEM2(offset) => match self.has_mem2 {
        true => Some(MEM2_OFFSET + offset),
        false => None,
      },
    }.map(|offset| {
      std::ptr::copy(
        payload.as_ptr(),
        self.shared_memory.get_ptr().add(offset as usize),
        payload.len(),
      );
    })
  }
}

impl DolphinSharedMemory {
  pub fn open_pid(pid: PidType) -> Result<DolphinSharedMemory, SharedMemoryOpenError> {
    let shared_memory = SharedMemory::open(&format!("dolphin-emu.{}", pid))?;
    let has_mem2 = shared_memory.size() >= MEM2_OFFSET + MEM2_SIZE;
    Ok(DolphinSharedMemory {shared_memory, has_mem2})
  }
}
