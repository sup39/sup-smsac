use crate::dolphin::Addr;

pub const MEM1_START_ADDR: Addr = Addr(0x8000_0000);
pub const MEM1_END_ADDR:   Addr = Addr(0x8180_0000);
pub const MEM1_SIZE:       u32 = MEM1_END_ADDR.0 - MEM1_START_ADDR.0;
pub const MEM2_START_ADDR: Addr = Addr(0x9000_0000);
pub const MEM2_END_ADDR:   Addr = Addr(0x9400_0000);
pub const MEM2_SIZE:       u32 = MEM2_END_ADDR.0 - MEM2_START_ADDR.0;

pub enum DolphinMemAddr {
  MEM1(u32),
  MEM2(u32),
}
impl DolphinMemAddr {
  pub fn space(&self) -> u32 {
    match self {
      DolphinMemAddr::MEM1(off) => MEM1_SIZE - off,
      DolphinMemAddr::MEM2(off) => MEM2_SIZE - off,
    }
  }
}
impl TryFrom<Addr> for DolphinMemAddr {
  type Error = ();
  fn try_from(addr: Addr) -> Result<Self, Self::Error> {
    if (MEM1_START_ADDR..MEM1_END_ADDR).contains(&addr) {
      Ok(DolphinMemAddr::MEM1(addr.0 - MEM1_START_ADDR.0))
    } else if (MEM2_START_ADDR..MEM2_END_ADDR).contains(&addr) {
      Ok(DolphinMemAddr::MEM2(addr.0 - MEM2_START_ADDR.0))
    } else {
      Err(())
    }
  }
}
