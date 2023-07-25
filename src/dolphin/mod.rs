pub use crate::big_endian::DecodeBE;
pub use crate::addr::Addr;
pub use crate::sys::process::PidType;
use crate::sys::process::{Process32Iterator, ProcessInfo};
use encoding_rs::SHIFT_JIS;

pub mod addr;
mod shared_memory;
mod process_memory;

pub use addr::DolphinMemAddr;
use shared_memory::DolphinSharedMemory;
use process_memory::DolphinProcessMemory;

pub trait Dolphin {
  /// # Safety
  /// `maddr + size` must be in bound
  unsafe fn read_memory_unchecked<T, F>(
    &self, maddr: DolphinMemAddr, size: usize, operator: F,
  ) -> Option<T>
    where F: FnOnce(*const u8) -> T;

  /// # Safety
  /// `maddr + payload.len()` must be in bound
  unsafe fn write_memory_unchecked(&self, maddr: DolphinMemAddr, payload: &[u8]) -> Option<()>;

  #[inline]
  fn read_memory<T, F>(&self, addr: Addr, size: usize, operator: F) -> Option<T>
    where F: FnOnce(*const u8) -> T
  {
    DolphinMemAddr::try_from(addr).ok().and_then(|maddr| {
      if (maddr.space() as usize) < size {return None}
      unsafe {self.read_memory_unchecked(maddr, size, operator)}
    })
  }

  #[inline]
  fn read_memory_truncated<T, F>(&self, addr: Addr, max_size: usize, operator: F) -> Option<T>
    where F: FnOnce(*const u8, usize) -> T
  {
    DolphinMemAddr::try_from(addr).ok().and_then(|maddr| {
      let size = std::cmp::min(maddr.space() as usize, max_size);
      unsafe {self.read_memory_unchecked(maddr, size, |ptr| operator(ptr, size))}
    })
  }

  #[inline]
  fn write_bytes(&self, addr: Addr, payload: &[u8]) -> Option<()> {
    let size = payload.len();
    DolphinMemAddr::try_from(addr).ok().and_then(|maddr| {
      if (maddr.space() as usize) < size {return None}
      unsafe {self.write_memory_unchecked(maddr, payload)}
    })
  }

  fn read<T: DecodeBE>(&self, addr: Addr) -> Option<T> {
    let size = std::mem::size_of::<T>();
    self.read_memory(addr, size, |ptr| unsafe {T::decode_be(ptr)})
  }
  fn read_str(&self, addr: Addr) -> Option<String> {
    let maxlen = 256; // TODO
    self.read_memory_truncated(addr, maxlen, |ptr, maxlen| {
      let mut len = 0usize;
      while len < maxlen && unsafe{*ptr.add(len)} != 0 {
        len += 1;
      }
      SHIFT_JIS.decode_without_bom_handling_and_without_replacement(
        unsafe{std::slice::from_raw_parts(ptr, len)}
      ).map(|s| s.into_owned())
    }).unwrap_or(None)
  }
  fn dump_hex(&self, addr: Addr, size: usize) -> Option<String> {
    self.read_memory(addr, size, |ptr| {
      (0..size)
        .map(|i| format!("{:02X}", unsafe {*ptr.add(i)}))
        .collect()
    })
  }
}

pub enum DolphinMemory {
  SharedMemory(DolphinSharedMemory),
  ProcessMemory(DolphinProcessMemory),
}
impl Dolphin for DolphinMemory {
  unsafe fn read_memory_unchecked<T, F>(&self, maddr: DolphinMemAddr, size: usize, operator: F) -> Option<T>
    where F: FnOnce(*const u8) -> T
  {
    match self {
      DolphinMemory::SharedMemory(m) => m.read_memory_unchecked(maddr, size, operator),
      DolphinMemory::ProcessMemory(m) => m.read_memory_unchecked(maddr, size, operator),
    }
  }
  unsafe fn write_memory_unchecked(&self, maddr: DolphinMemAddr, payload: &[u8]) -> Option<()> {
    match self {
      DolphinMemory::SharedMemory(m) => m.write_memory_unchecked(maddr, payload),
      DolphinMemory::ProcessMemory(m) => m.write_memory_unchecked(maddr, payload),
    }
  }
}
impl From<DolphinSharedMemory> for DolphinMemory {
  fn from(x: DolphinSharedMemory) -> Self {
    Self::SharedMemory(x)
  }
}
impl From<DolphinProcessMemory> for DolphinMemory {
  fn from(x: DolphinProcessMemory) -> Self {
    Self::ProcessMemory(x)
  }
}
impl DolphinMemory {
  pub fn list() -> impl Iterator<Item = (PidType, Option<DolphinMemory>)> {
    Process32Iterator::new().filter_map(|p| p.get_name().to_str().and_then(|name|
      match name {
        "Dolphin.exe" | "DolphinQt2.exe" | "DolphinWx.exe" => {
          let pid = p.pid();
          Some((pid, {
            DolphinSharedMemory::open_pid(pid).ok()
              .map(DolphinMemory::SharedMemory)
              .or_else(|| {
                DolphinProcessMemory::open_pid(pid).ok()
                  .map(DolphinMemory::ProcessMemory)
              })
          }))
        },
        _ => None,
      }
    ))
  }
}
