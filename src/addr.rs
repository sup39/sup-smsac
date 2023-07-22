/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::big_endian::DecodeBE;
use sup_smsac_derive::DecodeBE;

#[derive(DecodeBE, Clone, Copy, PartialEq, PartialOrd)]
pub struct Addr(pub u32);

impl From<Addr> for u32 {
  fn from(x: Addr) -> u32 {
    x.0
  }
}
impl From<u32> for Addr {
  fn from(x: u32) -> Self {
    Self(x)
  }
}

impl Addr {
  pub fn add(&self, rhs: u32) -> Addr {
    Addr(self.0 + rhs)
  }
  pub fn offset(&self, by: i32) -> Addr {
    Addr(self.0 + by as u32)
  }
}

impl std::ops::Add<u32> for Addr {
  type Output = Addr;
  fn add(self, other: u32) -> Addr {
    Addr(self.0+other)
  }
}
impl std::ops::Sub<u32> for Addr {
  type Output = Addr;
  fn sub(self, other: u32) -> Addr {
    Addr(self.0-other)
  }
}
impl std::ops::Sub<Addr> for Addr {
  type Output = isize;
  fn sub(self, other: Addr) -> isize {
    (self.0 - other.0) as i32 as isize
  }
}
impl std::fmt::Display for Addr {
  fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:08X}", self.0)
  }
}
impl std::fmt::Debug for Addr {
  fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:08X}", self.0)
  }
}

#[derive(Debug, Clone)]
pub struct AddrOffsets<T=u32>(pub T, pub Box<[T]>);
impl std::ops::Add<&AddrOffsets> for &AddrOffsets {
  type Output = AddrOffsets;
  fn add(self, other: &AddrOffsets) -> AddrOffsets {
    match self.1.split_last() {
      Some((last, init)) => AddrOffsets(
        self.0,
        [init, &[last+other.0], &other.1].concat().into(),
      ),
      None => AddrOffsets(self.0+other.0, other.1.clone()),
    }
  }
}
impl std::fmt::Display for AddrOffsets {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(fmt, "{:X}", self.0)?;
    for off in self.1.iter() {
      write!(fmt, ",{:X}", off)?;
    }
    Ok(())
  }
}
