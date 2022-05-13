use super::{CfgInfMask, DataBits, Parity, StopBits};

pub struct KeyId(u32);

pub enum StorageSize {
    OneBit,
    OneByte,
    TwoBytes,
    FourBytes,
    EightBytes,
}

impl StorageSize {
    pub const fn to_usize(self) -> usize {
        match self {
            Self::OneBit | Self::OneByte => 1,
            Self::TwoBytes => 2,
            Self::FourBytes => 4,
            Self::EightBytes => 8,
        }
    }
}

impl KeyId {
    pub(crate) const SIZE: usize = 4;

    pub const fn value_size(&self) -> StorageSize {
        match (self.0 >> 28) & 0b111 {
            1 => StorageSize::OneBit,
            2 => StorageSize::OneByte,
            3 => StorageSize::TwoBytes,
            4 => StorageSize::FourBytes,
            5 => StorageSize::EightBytes,
            _ => unreachable!(),
        }
    }

    pub const fn group_id(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub const fn item_id(&self) -> u8 {
        self.0 as u8
    }
}

macro_rules! from_cfg_v_bytes {
    ($buf:expr, bool) => {
        match $buf[0] {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    };
    ($buf:expr, u32) => {
        u32::from_le_bytes([$buf[0], $buf[1], $buf[2], $buf[3]])
    };
    ($buf:expr, CfgInfMask) => {
        CfgInfMask::from_bits_truncate($buf[0])
    };
    ($buf:expr, DataBits) => {
        match $buf[0] {
            0 => DataBits::Eight,
            1 => DataBits::Seven,
            _ => unreachable!(),
        }
    };
    ($buf:expr, Parity) => {
        match $buf[0] {
            0 => Parity::None,
            1 => Parity::Odd,
            2 => Parity::Even,
            _ => unreachable!(),
        }
    };
    ($buf:expr, StopBits) => {
        match $buf[0] {
            0 => StopBits::Half,
            1 => StopBits::One,
            2 => StopBits::OneHalf,
            3 => StopBits::Two,
            _ => unreachable!(),
        }
    };
}

macro_rules! into_cfg_kv_bytes {
    (@inner [$($byte:expr),+]) => {{
      let key_id = Self::KEY.0.to_le_bytes();

      [
        key_id[0], key_id[1], key_id[2], key_id[3],
        $(
          $byte,
        )*
      ]
    }};
    ($this:expr, bool) => {
      into_cfg_kv_bytes!(@inner [$this.0 as u8])
    };
    ($this:expr, u32) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1], bytes[2], bytes[3]])
    }};
    ($this:expr, CfgInfMask) => {
      into_cfg_kv_bytes!(@inner [
        $this.0.bits()
      ])
    };
    ($this:expr, DataBits) => {
      into_cfg_kv_bytes!(@inner [
        match $this.0 {
          DataBits::Eight => 0,
          DataBits::Seven => 1,
        }
      ])
    };
    ($this:expr, Parity) => {
      into_cfg_kv_bytes!(@inner [
        match $this.0 {
          Parity::None => 0,
          Parity::Odd => 1,
          Parity::Even => 2,
        }
      ])
    };
    ($this:expr, StopBits) => {
      into_cfg_kv_bytes!(@inner [
        match $this.0 {
          StopBits::Half => 0,
          StopBits::One => 1,
          StopBits::OneHalf => 2,
          StopBits::Two => 3,
        }
      ])
    };
}

macro_rules! cfg_val {
  ($($cfg_item:ident, $cfg_key_id:expr, $cfg_value_type:ident,)*) => {
    #[derive(Debug, Clone, Copy)]
    pub enum CfgVal {
      $(
        $cfg_item($cfg_value_type),
      )*
    }

    impl CfgVal {
      pub const fn len(&self) -> usize {
        match self {
          $(
            Self::$cfg_item(value) => {
              $cfg_item::SIZE
            }
          )*
        }
      }

      #[track_caller]
      pub fn parse(buf: &[u8]) -> Self {
        let key_id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        match key_id {
          $(
            $cfg_key_id => {
              Self::$cfg_item(from_cfg_v_bytes!(&buf[4..], $cfg_value_type))
            },
          )*
          _ => unimplemented!("unknown key ID: 0x{:8X}", key_id),
        }
      }

      pub fn extend_to<T>(&self, buf: &mut T) -> usize
      where
          T: core::iter::Extend<u8>
      {
        match self {
          $(
            Self::$cfg_item(value) => {
              let bytes = $cfg_item(*value).into_cfg_kv_bytes();
              let bytes_len = bytes.len();
              buf.extend(bytes);
              bytes_len
            }
          )*
        }
      }

      pub fn write_to(&self, buf: &mut [u8]) -> usize {
        match self {
          $(
            Self::$cfg_item(value) => {
              let kv: [u8; $cfg_item::SIZE] = $cfg_item(*value).into_cfg_kv_bytes();
              buf[..kv.len()].copy_from_slice(&kv[..]);
              kv.len()
            }
          )*
        }
      }
    }

    $(
      struct $cfg_item(pub $cfg_value_type);

      impl $cfg_item {
        const KEY: KeyId = KeyId($cfg_key_id);
        const SIZE: usize = KeyId::SIZE + Self::KEY.value_size().to_usize();

        pub const fn into_cfg_kv_bytes(self) -> [u8; Self::SIZE] {
          into_cfg_kv_bytes!(self, $cfg_value_type)
        }
      }
    )*
  }
}

cfg_val! {
  // CFG-UART1
  Uart1Baudrate,     0x40520001, u32,
  Uart1StopBits,     0x20520002, StopBits,
  Uart1DataBits,     0x20520003, DataBits,
  Uart1Parity,       0x20520004, Parity,
  Uart1Enabled,      0x10520005, bool,

  // CFG-UART1INPROT
  Uart1InProtUbx,    0x10730001, bool,
  Uart1InProtNmea,   0x10730002, bool,
  Uart1InProtRtcm3x, 0x10730004, bool,

  // CFG-UART1OUTPROT
  Uart1OutProtUbx,    0x10740001, bool,
  Uart1OutProtNmea,   0x10740002, bool,
  Uart1OutProtRtcm3x, 0x10740004, bool,

  // CFG-INFMSG
  InfmsgUbxI2c,    0x20920001, CfgInfMask,
  InfmsgUbxUart1,  0x20920002, CfgInfMask,
  InfmsgUbxUart2,  0x20920003, CfgInfMask,
  InfmsgUbxUsb,    0x20920004, CfgInfMask,
  InfmsgUbxSpi,    0x20920005, CfgInfMask,
  InfmsgNmeaI2c,   0x20920006, CfgInfMask,
  InfmsgNmeaUart1, 0x20920007, CfgInfMask,
  InfmsgNmeaUart2, 0x20920008, CfgInfMask,
  InfmsgNmeaUsb,   0x20920009, CfgInfMask,
  InfmsgNmeaSpi,   0x2092000a, CfgInfMask,
}
