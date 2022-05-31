use super::{AlignmentToReferenceTime, CfgInfMask, DataBits, Parity, StopBits};

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
    ($buf:expr, u8) => {
        $buf[0]
    };
    ($buf:expr, u16) => {
        u16::from_le_bytes([$buf[0], $buf[1]])
    };
    ($buf:expr, i16) => {
      i16::from_le_bytes([$buf[0], $buf[1]])
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
    ($buf:expr, AlignmentToReferenceTime) => {
        match $buf[0] {
            0 => AlignmentToReferenceTime::Utc,
            1 => AlignmentToReferenceTime::Gps,
            2 => AlignmentToReferenceTime::Glo,
            3 => AlignmentToReferenceTime::Bds,
            4 => AlignmentToReferenceTime::Gal,
            _ => unreachable!(),
        }
    };
    ($buf:expr, TpPulse) => {
      match $buf[0] {
          0 => TpPulse::Period,
          1 => TpPulse::Freq,
          _ => unreachable!(),
      }
    };
    ($buf:expr, TpPulseLength) => {
      match $buf[0] {
          0 => TpPulseLength::Ratio,
          1 => TpPulseLength::Length,
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
    ($this:expr, u8) => {{
      into_cfg_kv_bytes!(@inner [$this.0])
    }};
    ($this:expr, u16) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1]])
    }};
    ($this:expr, i16) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1]])
    }};
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
    ($this:expr, AlignmentToReferenceTime) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, TpPulse) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, TpPulseLength) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
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
  Uart1Baudrate,        0x40520001, u32,
  Uart1StopBits,        0x20520002, StopBits,
  Uart1DataBits,        0x20520003, DataBits,
  Uart1Parity,          0x20520004, Parity,
  Uart1Enabled,         0x10520005, bool,

  // CFG-UART1INPROT
  Uart1InProtUbx,       0x10730001, bool,
  Uart1InProtNmea,      0x10730002, bool,
  Uart1InProtRtcm3x,    0x10730004, bool,

  // CFG-UART1OUTPROT
  Uart1OutProtUbx,       0x10740001, bool,
  Uart1OutProtNmea,      0x10740002, bool,
  Uart1OutProtRtcm3x,    0x10740004, bool,

  // CFG-INFMSG
  InfmsgUbxI2c,          0x20920001, CfgInfMask,
  InfmsgUbxUart1,        0x20920002, CfgInfMask,
  InfmsgUbxUart2,        0x20920003, CfgInfMask,
  InfmsgUbxUsb,          0x20920004, CfgInfMask,
  InfmsgUbxSpi,          0x20920005, CfgInfMask,
  InfmsgNmeaI2c,         0x20920006, CfgInfMask,
  InfmsgNmeaUart1,       0x20920007, CfgInfMask,
  InfmsgNmeaUart2,       0x20920008, CfgInfMask,
  InfmsgNmeaUsb,         0x20920009, CfgInfMask,
  InfmsgNmeaSpi,         0x2092000a, CfgInfMask,

  // CFG-RATE-*
  RateMeas,              0x30210001, u16,
  RateNav,               0x30210002, u16,
  RateTimeref,           0x20210003, AlignmentToReferenceTime,

  // CFG-MSGOUT-*
  MsgoutNmeaIdGgaI2c,    0x209100ba, u8,
  MsgoutNmeaIdGgaSpi,    0x209100be, u8,
  MsgoutNmeaIdGgaUart1,  0x209100bb, u8,
  MsgoutNmeaIdGgaUart2,  0x209100bc, u8,
  MsgoutNmeaIdGgaUsb,    0x209100bd, u8,

  MsgoutNmeaIdGllI2c,    0x209100c9, u8,
  MsgoutNmeaIdGllSpi,    0x209100cd, u8,
  MsgoutNmeaIdGllUart1,  0x209100ca, u8,
  MsgoutNmeaIdGllUart2,  0x209100cb, u8,
  MsgoutNmeaIdGllUsb,    0x209100cc, u8,

  MsgoutNmeaIdGnsI2c,    0x209100b5, u8,
  MsgoutNmeaIdGnsSpi,    0x209100b9, u8,
  MsgoutNmeaIdGnsUart1,  0x209100b6, u8,
  MsgoutNmeaIdGnsUart2,  0x209100b7, u8,
  MsgoutNmeaIdGnsUsb,    0x209100b8, u8,

  MsgoutNmeaIdGrsI2c,    0x209100ce, u8,
  MsgoutNmeaIdGrsSpi,    0x209100d2, u8,
  MsgoutNmeaIdGrsUart1,  0x209100cf, u8,
  MsgoutNmeaIdGrsUart2,  0x209100d0, u8,
  MsgoutNmeaIdGrsUsb,    0x209100d1, u8,

  MsgoutNmeaIdGsaI2c,    0x209100bf, u8,
  MsgoutNmeaIdGsaSpi,    0x209100c3, u8,
  MsgoutNmeaIdGsaUart1,  0x209100c0, u8,
  MsgoutNmeaIdGsaUart2,  0x209100c1, u8,
  MsgoutNmeaIdGsaUsb,    0x209100c2, u8,

  MsgoutNmeaIdGstI2c,    0x209100d3, u8,
  MsgoutNmeaIdGstSpi,    0x209100d7, u8,
  MsgoutNmeaIdGstUart1,  0x209100d4, u8,
  MsgoutNmeaIdGstUart2,  0x209100d5, u8,
  MsgoutNmeaIdGstUsb,    0x209100d6, u8,

  MsgoutNmeaIdGsvI2c,    0x209100c4, u8,
  MsgoutNmeaIdGsvSpi,    0x209100c8, u8,
  MsgoutNmeaIdGsvUart1,  0x209100c5, u8,
  MsgoutNmeaIdGsvUart2,  0x209100c6, u8,
  MsgoutNmeaIdGsvUsb,    0x209100c7, u8,

  MsgoutNmeaIdRmcI2c,    0x209100ab, u8,
  MsgoutNmeaIdRmcSpi,    0x209100af, u8,
  MsgoutNmeaIdRmcUart1,  0x209100ac, u8,
  MsgoutNmeaIdRmcUart2,  0x209100ad, u8,
  MsgoutNmeaIdRmcUsb,    0x209100ae, u8,

  MsgoutNmeaIdVlwI2c,    0x209100e7, u8,
  MsgoutNmeaIdVlwSpi,    0x209100eb, u8,
  MsgoutNmeaIdVlwUart1,  0x209100e8, u8,
  MsgoutNmeaIdVlwUart2,  0x209100e9, u8,
  MsgoutNmeaIdVlwUsb,    0x209100ea, u8,

  MsgoutNmeaIdVtgI2c,    0x209100b0, u8,
  MsgoutNmeaIdVtgSpi,    0x209100b4, u8,
  MsgoutNmeaIdVtgUart1,  0x209100b1, u8,
  MsgoutNmeaIdVtgUart2,  0x209100b2, u8,
  MsgoutNmeaIdVtgUsb,    0x209100b3, u8,

  MsgoutNmeaIdZdaI2c,    0x209100d8, u8,
  MsgoutNmeaIdZdaSpi,    0x209100dc, u8,
  MsgoutNmeaIdZdaUart1,  0x209100d9, u8,
  MsgoutNmeaIdZdaUart2,  0x209100da, u8,
  MsgoutNmeaIdZdaUsb,    0x209100db, u8,

  MsgoutUbxRxmRawxI2x,   0x209102a4, u8,
  MsgoutUbxRxmRawxSpi,   0x209102a8, u8,
  MsgoutUbxRxmRawxUart1, 0x209102a5, u8,
  MsgoutUbxRxmRawxUart2, 0x209102a6, u8,
  MsgoutUbxRxmRawxUsb,   0x209102a7, u8,

  MsgoutUbxTimTpI2c,     0x2091017d, u8,
  MsgoutUbxTimTpSpi,     0x20910181, u8,
  MsgoutUbxTimTpUart1,   0x2091017e, u8,
  MsgoutUbxTimTpUart2,   0x2091017f, u8,
  MsgoutUbxTimTpUsb,     0x20910180, u8,

  // CFG-SIGNAL-*
  SignalGpsEna,          0x1031001f, bool,
  SignalGpsL1caEna,      0x10310001, bool,
  SignalGpsL2cEna,       0x10310003, bool,
  SignalGalEna,          0x10310021, bool,
  SignalGalE1Ena,        0x10310007, bool,
  SignalGalE5bEna,       0x1031000a, bool,
  SignalBdsEna,          0x10310022, bool,
  SignalBdsB1Ena,        0x1031000d, bool,
  SignalBdsB2Ena,        0x1031000e, bool,
  SignalQzssEna,         0x10310024, bool,
  SignalQzssL1caEna,     0x10310012, bool,
  SignalQzssL2cEna,      0x10310015, bool,
  SignalGloEna,          0x10310025, bool,
  SignalGloL1Ena,        0x10310018, bool,
  SignalGLoL2Ena,        0x1031001a, bool,

  // CFG-TP-*
  TpPulseDef,            0x20050023, TpPulse,
  TpPulseLengthDef,      0x20050030, TpPulseLength,
  TpAntCableDelay,       0x30050001, i16,
  TpPeriodTp1,           0x40050002, u32,
  TpPeriodLockTp1,       0x40050003, u32,
  TpFreqTp1,             0x40050024, u32,
  TpFreqLockTp1,         0x40050025, u32,
  TpLenTp1,              0x40050004, u32,
  TpLenLockTp1,          0x40050005, u32,
  TpTp1Ena,              0x10050007, bool,
  TpSyncGnssTp1,         0x10050008, bool,
  TpUseLockedTp1,        0x10050009, bool,
  TpAlignToTowTp1,       0x1005000a, bool,
  TpPolTp1,              0x1005000b, bool,
  TpTimegridTp1,         0x2005000c, AlignmentToReferenceTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TpPulse {
  /// Time pulse period
  Period = 0,
  /// Time pulse frequency
  Freq = 1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TpPulseLength {
  /// Time pulse ratio
  Ratio = 0,
  /// Time pulse length
  Length = 1,
}
