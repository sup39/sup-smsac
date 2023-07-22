use crate::{
  addr::Addr,
  big_endian::DecodeBE,
  dolphin::Dolphin,
  sms::SMSDolphin,
};
use std::marker::PhantomData;

pub trait FieldReader<D: Dolphin, T> {
  fn read(&self, d: &D, addr: Addr) -> Option<T>;
}

pub struct PrimitiveFieldReader<T> {
  phantom: PhantomData<T>,
}
impl<T> PrimitiveFieldReader<T> {
  pub fn new() -> Self {
    Self {
      phantom: PhantomData,
    }
  }
}
impl<D: Dolphin, T: DecodeBE> FieldReader<D, T> for PrimitiveFieldReader<T> {
  fn read(&self, d: &D, addr: Addr) -> Option<T> {
    d.read::<T>(addr)
  }
}
impl<D: Dolphin, T: DecodeBE + ToString> FieldReader<D, String> for PrimitiveFieldReader<T> {
  fn read(&self, d: &D, addr: Addr) -> Option<String> {
    d.read::<T>(addr).map(|x| x.to_string())
  }
}

pub struct F32FieldReader;
impl<D: Dolphin> FieldReader<D, f32> for F32FieldReader {
  fn read(&self, d: &D, addr: Addr) -> Option<f32> {
    d.read::<f32>(addr)
  }
}
impl<D: Dolphin> FieldReader<D, String> for F32FieldReader {
  fn read(&self, d: &D, addr: Addr) -> Option<String> {
    d.read::<f32>(addr).map(|x| match x.abs() {
      m if (1e-4..1e8).contains(&m) || m == 0f32 => {
        let s = format!("{x}");
        match m < 8388608f32 && s.contains('.') {
          true => s,
          false => s+".0",
        }
      },
      _ => format!("{x:e}"),
    })
  }
}

pub struct StringFieldReader;
impl<D: Dolphin> FieldReader<D, String> for StringFieldReader {
  fn read(&self, d: &D, addr: Addr) -> Option<String> {
    d.read::<Addr>(addr).and_then(|a| d.read_str(a))
  }
}

pub struct ClassNameReader;
impl FieldReader<SMSDolphin, String> for ClassNameReader {
  fn read(&self, d: &SMSDolphin, addr: Addr) -> Option<String> {
    d.read::<Addr>(addr)
      .map(|addr| d.get_class_string(addr))
  }
}

pub struct HexFieldReader(pub usize);
impl<D: Dolphin> FieldReader<D, String> for HexFieldReader {
  fn read(&self, d: &D, addr: Addr) -> Option<String> {
    d.read_bytes(addr, self.0)
      .map(|bytes| bytes.iter().map(|x| format!("{x:02X}")).collect())
  }
}
