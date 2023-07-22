/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::sys::process::PidType;
use crate::sys::shared_memory::{SharedMemory, SharedMemoryOpenError};
use crate::addr::Addr;
use crate::big_endian::DecodeBE;
use crate::sys::process::{Process32Iterator, ProcessInfo};
use encoding_rs::SHIFT_JIS;

pub const MEM1_START_ADDR: Addr = Addr(0x8000_0000);
pub const MEM1_END_ADDR:   Addr = Addr(0x8180_0000);
pub trait Dolphin {
  /// # Safety
  ///
  /// The offset must be smaller than the size of the memory region
  unsafe fn mem<T: Into<isize>>(&self, offset: T) -> *mut u8;

  fn get_ptr_mut(&self, addr: Addr, size: usize) -> Option<*mut u8> {
    if MEM1_START_ADDR <= addr && addr < MEM1_END_ADDR - size as u32 {
      Some(unsafe {self.mem(addr - MEM1_START_ADDR)})
    } else {
      None
    }
  }

  #[inline]
  fn read_bytes(&self, addr: Addr, size: usize) -> Option<&[u8]> {
    self.get_ptr_mut(addr, size)
      .map(|ptr| unsafe {std::slice::from_raw_parts(ptr, size)})
  }
  #[inline]
  fn read<T: DecodeBE>(&self, addr: Addr) -> Option<T> {
    let size = std::mem::size_of::<T>();
    self.get_ptr_mut(addr, size)
      .map(|ptr| unsafe {T::decode_be(ptr)})
  }
  fn read_str(&self, addr: Addr) -> Option<String> {
    if MEM1_START_ADDR <= addr && addr < MEM1_END_ADDR {
      let ptr = unsafe {self.mem(addr - MEM1_START_ADDR)};
      const MAX_LENGTH: u32 = 256; // TODO
      let max_length = MAX_LENGTH; // TODO
      let maxlen = std::cmp::min(max_length as usize, (MEM1_END_ADDR - addr) as usize);
      // let maxlen = (MEM1_END_ADDR - addr) as usize;
      let mut len = 0usize;
      while len < maxlen {
        if unsafe{*ptr.add(len) == 0} {break;}
        len += 1;
      }
      SHIFT_JIS.decode_without_bom_handling_and_without_replacement(unsafe{std::slice::from_raw_parts(ptr, len)}).map(|x| x.into_owned())
    } else {
      None
    }
  }

  #[inline]
  fn write_bytes(&self, addr: Addr, payload: &[u8]) -> Option<()> {
    let size = payload.len();
    self.get_ptr_mut(addr, size)
      .map(|ptr| unsafe {std::ptr::copy(payload.as_ptr(), ptr, size)})
  }
}

pub struct DolphinMemory {
  shared_memory: SharedMemory,
  pid: PidType,
}

impl Dolphin for DolphinMemory {
  #[inline]
  unsafe fn mem<T: Into<isize>>(&self, offset: T) -> *mut u8 {
    self.shared_memory.get_ptr().offset(offset.into())
  }
}
impl DolphinMemory {
  #[inline]
  pub fn pid(&self) -> PidType {
    self.pid
  }

  pub fn open_pid<T: Into<PidType>>(pid: T) -> Result<DolphinMemory, SharedMemoryOpenError> {
    let pid: usize = pid.into();
    let shared_memory = SharedMemory::open(&format!("dolphin-emu.{}", pid))?;
    Ok(DolphinMemory {shared_memory, pid})
  }
  pub fn list_dolphin() -> impl Iterator<Item = (usize, Result<DolphinMemory, SharedMemoryOpenError>)> {
    Process32Iterator::new().filter_map(|p| p.get_name().to_str().and_then(|name|
      match name {
        "Dolphin.exe" => Some((p.pid(), DolphinMemory::open_pid(p.pid()))),
        _ => None,
      }
    ))
  }
}
