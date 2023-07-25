/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi::c_void;
use windows::core::PCSTR;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::Memory::{
  OpenFileMappingA,
  FILE_MAP_ALL_ACCESS,
  MapViewOfFile,
  UnmapViewOfFile,
  MEMORYMAPPEDVIEW_HANDLE,
  MEMORY_BASIC_INFORMATION,
  VirtualQuery,
};

#[derive(Debug)]
pub enum SharedMemoryOpenError {
  OpenFileFailure(String),
  MapViewFailure(String),
  MemoryUninitialized,
}

pub struct SharedMemory {
  h_file_mapping: HANDLE,
  h_map_view: MEMORYMAPPEDVIEW_HANDLE,
  size: u32,
}
impl SharedMemory {
  #[inline]
  pub fn get_ptr(&self) -> *mut u8 {
    self.h_map_view.0 as *mut u8
  }
  #[inline]
  pub fn size(&self) -> u32 {
    self.size
  }
}

impl SharedMemory {
  pub fn open(name: &str) -> Result<Self, SharedMemoryOpenError> {
    let name = name.to_owned() + "\0";

    // open file mapping
    let h_file_mapping = unsafe {
      OpenFileMappingA(FILE_MAP_ALL_ACCESS.0, false, PCSTR::from_raw(name.as_ptr()))
        .map_err(|e| SharedMemoryOpenError::OpenFileFailure(e.message().to_string()))?
    };

    // create map view
    let h_map_view = unsafe {
      MapViewOfFile(h_file_mapping, FILE_MAP_ALL_ACCESS, 0, 0, 0).map_err(|e| {
        CloseHandle(h_file_mapping);
        SharedMemoryOpenError::MapViewFailure(e.message().to_string())
      })?
    };
    if h_map_view.is_invalid() {
      unsafe {CloseHandle(h_file_mapping)};
      return Err(SharedMemoryOpenError::MemoryUninitialized);
    }

    let mut meminfo = MEMORY_BASIC_INFORMATION::default();
    unsafe {VirtualQuery(
      Some(h_map_view.0 as *mut c_void), &mut meminfo, std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
    )};
    let size = meminfo.RegionSize as u32;

    // create SharedMemory successfully
    Ok(Self {h_file_mapping, h_map_view, size})
  }
}

impl Drop for SharedMemory {
  fn drop(&mut self) {
    unsafe {
      UnmapViewOfFile(self.h_map_view);
      CloseHandle(self.h_file_mapping);
    }
  }
}
