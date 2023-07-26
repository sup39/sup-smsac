/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

pub trait DecodeBE {
  const PACKED_SIZE: usize;
  /// # Safety
  ///
  /// [`ptr`, `ptr+size_of(Self)`) must be valid
  unsafe fn decode_be(ptr: *const u8) -> Self;
}

macro_rules! impl_decode_be_for_int {
  ($type:ident, $size:literal) => {
    impl DecodeBE for $type {
      const PACKED_SIZE: usize = $size;
      #[inline]
      unsafe fn decode_be(ptr: *const u8) -> Self {
        $type::to_be(*(ptr as *const $type))
      }
    }
  };
}
macro_rules! impl_decode_be_for_float {
  ($ftype:ident, $itype:ident, $size:literal) => {
    impl DecodeBE for $ftype {
      const PACKED_SIZE: usize = $size;
      #[inline]
      unsafe fn decode_be(ptr: *const u8) -> Self {
        $ftype::from_bits($itype::to_be(*(ptr as *const $itype)))
      }
    }
  };
}

impl_decode_be_for_int!(u8, 1);
impl_decode_be_for_int!(i8, 1);
impl_decode_be_for_int!(u16, 2);
impl_decode_be_for_int!(i16, 2);
impl_decode_be_for_int!(u32, 4);
impl_decode_be_for_int!(i32, 4);
impl_decode_be_for_int!(u64, 8);
impl_decode_be_for_int!(i64, 8);
impl_decode_be_for_int!(u128, 16);
impl_decode_be_for_int!(i128, 16);
impl_decode_be_for_float!(f32, u32, 4);
impl_decode_be_for_float!(f64, u64, 8);

impl<const N: usize> DecodeBE for [u8; N] {
  const PACKED_SIZE: usize = N;
  unsafe fn decode_be(ptr: *const u8) -> Self {
    *(ptr as *const [u8; N])
  }
}
