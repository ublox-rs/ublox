use crate::{NavDynamicModel, NavFixMode, UtcStandardIdentifier};

use super::{AlignmentToReferenceTime, CfgInfMask, CfgTModeModes, DataBits, Parity, StopBits};

/// Supported storage size identiﬁers for the Configuration Value
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

/// Configuration Key ID for the uBlox Conﬁguration Interface
pub struct KeyId(u32);

impl KeyId {
    pub(crate) const SIZE: usize = 4;

    /// Retrieve the storage size of the Configuration Value
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

    /// Group ID portion of the Key ID
    pub const fn group_id(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Item ID portion of the Key ID
    pub const fn item_id(&self) -> u8 {
        self.0 as u8
    }

    /// Extend a buffer with the contents of [KeyId].
    pub fn extend_to<T>(&self, buf: &mut T) -> usize
    where
        T: core::iter::Extend<u8>,
    {
        let bytes = self.0.to_le_bytes();
        buf.extend(bytes);
        Self::SIZE
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
    ($buf:expr, i8) => {
        $buf[0] as i8
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
    ($buf:expr, i32) => {
        i32::from_le_bytes([$buf[0], $buf[1], $buf[2], $buf[3]])
    };
    ($buf:expr, u64) => {
        u64::from_le_bytes([
            $buf[0], $buf[1], $buf[2], $buf[3], $buf[4], $buf[5], $buf[6], $buf[7],
        ])
    };
    ($buf:expr, f32) => {
        f32::from_le_bytes([$buf[0], $buf[1], $buf[2], $buf[3]])
    };
    ($buf:expr, f64) => {
        f64::from_le_bytes([
            $buf[0], $buf[1], $buf[2], $buf[3], $buf[4], $buf[5], $buf[6], $buf[7],
        ])
    };
    ($buf:expr, CfgInfMask) => {
        CfgInfMask::from_bits_truncate($buf[0])
    };
    ($buf:expr, DataBits) => {
        match $buf[0] {
            0 => DataBits::Eight,
            1 => DataBits::Seven,
            _ => unreachable!("DataBits value not supported by protocol specification"),
        }
    };
    ($buf:expr, Parity) => {
        match $buf[0] {
            0 => Parity::None,
            1 => Parity::Odd,
            2 => Parity::Even,
            _ => unreachable!("Parity value not supported by protocol specification"),
        }
    };
    ($buf:expr, StopBits) => {
        match $buf[0] {
            0 => StopBits::Half,
            1 => StopBits::One,
            2 => StopBits::OneHalf,
            3 => StopBits::Two,
            _ => unreachable!("StopBits value not supported by protocol specification"),
        }
    };
    ($buf:expr, AlignmentToReferenceTime) => {
        match $buf[0] {
            0 => AlignmentToReferenceTime::Utc,
            1 => AlignmentToReferenceTime::Gps,
            2 => AlignmentToReferenceTime::Glo,
            3 => AlignmentToReferenceTime::Bds,
            4 => AlignmentToReferenceTime::Gal,
            _ => unreachable!("CFG-RATE-TIMEREF value not supported by protocol specification"),
        }
    };
    ($buf:expr, TpPulse) => {
        match $buf[0] {
            0 => TpPulse::Period,
            1 => TpPulse::Freq,
            _ => unreachable!("CFG-TP-PULSE_DEF value not supported by protocol specification"),
        }
    };
    ($buf:expr, TpPulseLength) => {
        match $buf[0] {
            0 => TpPulseLength::Ratio,
            1 => TpPulseLength::Length,
            _ => unreachable!(
                "CFG-TP-PULSE_LENGTH_DEF value not supported by protocol specification"
            ),
        }
    };
    ($buf:expr, CfgTModeModes) => {
        match $buf[0] {
            0 => CfgTModeModes::Disabled,
            1 => CfgTModeModes::SurveyIn,
            2 => CfgTModeModes::Fixed,
            _ => unreachable!("CFG-TMODE-MODE value not supported by protocol specification"),
        }
    };
    ($buf:expr, TModePosType) => {
        match $buf[0] {
            0 => TModePosType::ECEF,
            1 => TModePosType::LLH,
            _ => unreachable!("CFG-TMODE-POS_TYPE value not supported by protocol specification"),
        }
    };
    ($buf:expr, NavFixMode) => {
        match $buf[0] {
            1 => NavFixMode::Only2D,
            2 => NavFixMode::Only3D,
            3 => NavFixMode::Auto2D3D,
            _ => unreachable!(
                "CFG-NAVSPG-FIXMODE_TYPE value not supported by protocol specification"
            ),
        }
    };
    ($buf:expr, UtcStandardIdentifier) => {
        match $buf[0] {
            0 => UtcStandardIdentifier::Automatic,
            3 => UtcStandardIdentifier::Usno,
            6 => UtcStandardIdentifier::UtcSu,
            7 => UtcStandardIdentifier::UtcChina,
            _ => unreachable!(
                "CFG-NAVSPG-UTCSTANDARD_TYPE value not supported by protocol specification"
            ),
        }
    };
    ($buf:expr, NavDynamicModel) => {
        match $buf[0] {
            0 => NavDynamicModel::Portable,
            2 => NavDynamicModel::Stationary,
            3 => NavDynamicModel::Pedestrian,
            4 => NavDynamicModel::Automotive,
            5 => NavDynamicModel::Sea,
            6 => NavDynamicModel::AirborneWithLess1gAcceleration,
            7 => NavDynamicModel::AirborneWithLess2gAcceleration,
            8 => NavDynamicModel::AirborneWithLess4gAcceleration,
            #[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
            9 => NavDynamicModel::WristWornWatch,
            #[cfg(feature = "ubx_proto31")]
            10 => NavDynamicModel::Bike,
            _ => unreachable!(
                "CFG-NAVSPG-DYNMODEL_TYPE value not supported by protocol specification"
            ),
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
    ($this:expr, i8) => {{
      into_cfg_kv_bytes!(@inner [$this.0 as u8])
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
    ($this:expr, i32) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1], bytes[2], bytes[3]])
    }};
    ($this:expr, u64) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }};
    ($this:expr, f32) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1], bytes[2], bytes[3]])
    }};
    ($this:expr, f64) => {{
      let bytes = $this.0.to_le_bytes();
      into_cfg_kv_bytes!(@inner [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
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
    ($this:expr, CfgTModeModes) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, TModePosType) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, NavFixMode) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, UtcStandardIdentifier) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
    ($this:expr, NavDynamicModel) => {
      into_cfg_kv_bytes!(@inner [
          $this.0 as u8
      ])
    };
}

macro_rules! cfg_val {
  (
    $(
      $(#[$class_comment:meta])*
      $cfg_item:ident, $cfg_key_id:expr, $cfg_value_type:ident,
    )*
  ) => {
    #[derive(Debug, Clone, Copy)]
    #[non_exhaustive]
    pub enum CfgKey {
      WildcardAll = 0x7fffffff,
      $(
        $(#[$class_comment])*
        $cfg_item = $cfg_key_id,
      )*
    }

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[non_exhaustive]
    pub enum CfgVal {
      $(
        $(#[$class_comment])*
        $cfg_item($cfg_value_type),
      )*
    }

    impl CfgVal {
      pub const fn len(&self) -> usize {
        match self {
          $(
            Self::$cfg_item(_) => {
              $cfg_item::SIZE
            }
          )*
        }
      }

      pub const fn is_empty(&self) -> bool {
          self.len() == 0
      }

      #[track_caller]
      pub fn parse(buf: &[u8]) -> Option<Self> {
        let key_id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        match key_id {
          $(
            $cfg_key_id => {
              Some(Self::$cfg_item(from_cfg_v_bytes!(&buf[4..], $cfg_value_type)))
            },
          )*
          _ => {
            // TODO: add a mechanism to log such messages that supports also no_std
            // eprintln!("unknown key ID: 0x{:8X}", key_id);
            None
           },
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

impl CfgKey {
    pub fn extend_to<T>(&self, buf: &mut T) -> usize
    where
        T: core::iter::Extend<u8>,
    {
        let bytes = (*self as u32).to_le_bytes();
        let bytes_len = bytes.len();
        buf.extend(bytes);
        bytes_len
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

  // CFG-UART2
  Uart2Baudrate,        0x40530001, u32,
  Uart2StopBits,        0x20530002, StopBits,
  Uart2DataBits,        0x20530003, DataBits,
  Uart2Parity,          0x20530004, Parity,
  Uart2Enabled,         0x10530005, bool,
  Uart2Remap,           0x10530006, bool,

  // CFG-UART2INPROT
  Uart2InProtUbx,       0x10750001, bool,
  Uart2InProtNmea,      0x10750002, bool,
  Uart2InProtRtcm3x,    0x10750004, bool,

  // CFG-UART2OUTPROT
  Uart2OutProtUbx,       0x10760001, bool,
  Uart2OutProtNmea,      0x10760002, bool,
  Uart2OutProtRtcm3x,    0x10760004, bool,

  // CFG-USB
  UsbEnabled,           0x10650001, bool,
  UsbSelfpow,           0x10650002, bool,
  UsbVendorId,          0x3065000a, u16,
  UsbProductId,         0x3065000b, u16,
  UsbPower,             0x3065000c, u16,
  UsbVendorStr0,        0x5065000d, u64,
  UsbVendorStr1,        0x5065000e, u64,
  UsbVendorStr2,        0x5065000f, u64,
  UsbVendorStr3,        0x50650010, u64,
  UsbProductStr0,       0x50650011, u64,
  UsbProductStr1,       0x50650012, u64,
  UsbProductStr2,       0x50650013, u64,
  UsbProductStr3,       0x50650014, u64,
  UsbSerialNoStr0,      0x50650015, u64,
  UsbSerialNoStr1,      0x50650016, u64,
  UsbSerialNoStr2,      0x50650017, u64,
  UsbSerialNoStr3,      0x50650018, u64,

  // CFG-USBINPROT-*
  UsbInProtUbx,         0x10770001, bool,
  UsbInProtNmea,        0x10770002, bool,
  UsbInProtRtcm3x,      0x10770004, bool,

  // CFG-USBOUTPROT-*
  UsbOutProtUbx,        0x10780001, bool,
  UsbOutProtNmea,       0x10780002, bool,
  UsbOutProtRtcm3x,     0x10780004, bool,

  // CFG-INFMSG
  InfMsgUbxI2c,          0x20920001, CfgInfMask,
  InfMsgUbxUart1,        0x20920002, CfgInfMask,
  InfMsgUbxUart2,        0x20920003, CfgInfMask,
  InfMsgUbxUsb,          0x20920004, CfgInfMask,
  InfMsgUbxSpi,          0x20920005, CfgInfMask,
  InfMsgNmeaI2c,         0x20920006, CfgInfMask,
  InfMsgNmeaUart1,       0x20920007, CfgInfMask,
  InfMsgNmeaUart2,       0x20920008, CfgInfMask,
  InfMsgNmeaUsb,         0x20920009, CfgInfMask,
  InfMsgNmeaSpi,         0x2092000a, CfgInfMask,

  // CFG-RATE-*
  /// Nominal time between GNSS measurements
  /// (e.g. 100ms results in 10Hz measurement rate, 1000ms = 1Hz measurement rate)
  RateMeas,              0x30210001, u16,
  /// Ratio of number of measurements to number of navigation solutions
  RateNav,               0x30210002, u16,
  /// Time system to which measurements are aligned
  RateTimeref,           0x20210003, AlignmentToReferenceTime,

  // CFG-MSGOUT-*
  /// Output rate of the NMEA-GX-DTM message on port I2C
  MsgOutNmeaIdDtmI2c, 0x209100a6, u8,
  /// Output rate of the NMEA-GX-DTM message on port SPI
  MsgOutNmeaIdDtmSpi, 0x209100aa, u8,
  /// Output rate of the NMEA-GX-DTM message on port UART1
  MsgOutNmeaIdDtmuart1, 0x209100a7, u8,
  /// Output rate of the NMEA-GX-DTM message on port UART2
  MsgOutNmeaIdDtmuart2, 0x209100a8, u8,
  /// Output rate of the NMEA-GX-DTM message on port USB
  MsgOutNmeaIdDtmUsb, 0x209100a9, u8,
  /// Output rate of the NMEA-GX-GBS message on port I2C
  MsgOutNmeaIdGbsI2c, 0x209100dd, u8,
  /// Output rate of the NMEA-GX-GBS message on port SPI
  MsgOutNmeaIdGbsSpi, 0x209100e1, u8,
  /// Output rate of the NMEA-GX-GBS message on port UART1
  MsgOutNmeaIdGbsUart1, 0x209100de, u8,
  /// Output rate of the NMEA-GX-GBS message on port UART2
  MsgOutNmeaIdGbsUart2, 0x209100df, u8,
  /// Output rate of the NMEA-GX-GBS message on port USB
  MsgOutNmeaIdGbsUsb, 0x209100e0, u8,
  /// Output rate of the NMEA-GX-GGA message on port I2C
  MsgOutNmeaIdGgaI2c, 0x209100ba, u8,
  /// Output rate of the NMEA-GX-GGA message on port SPI
  MsgOutNmeaIdGgaSpi, 0x209100be, u8,
  /// Output rate of the NMEA-GX-GGA message on port UART1
  MsgOutNmeaIdGgaUart1, 0x209100bb, u8,
  /// Output rate of the NMEA-GX-GGA message on port UART2
  MsgOutNmeaIdGgaUart2, 0x209100bc, u8,
  /// Output rate of the NMEA-GX-GGA message on port USB
  MsgOutNmeaIdGgaUsb, 0x209100bd, u8,
  /// Output rate of the NMEA-GX-GLL message on port I2C
  MsgOutNmeaIdGllI2c, 0x209100c9, u8,
  /// Output rate of the NMEA-GX-GLL message on port SPI
  MsgOutNmeaIdGllSpi, 0x209100cd, u8,
  /// Output rate of the NMEA-GX-GLL message on port UART1
  MsgOutNmeaIdGllUart1, 0x209100ca, u8,
  /// Output rate of the NMEA-GX-GLL message on port UART2
  MsgOutNmeaIdGllUart2, 0x209100cb, u8,
  /// Output rate of the NMEA-GX-GLL message on port USB
  MsgOutNmeaIdGllUsb, 0x209100cc, u8,
  /// Output rate of the NMEA-GX-GNS message on port I2C
  MsgOutNmeaIdGnsI2c, 0x209100b5, u8,
  /// Output rate of the NMEA-GX-GNS message on port SPI
  MsgOutNmeaIdGnsSpi, 0x209100b9, u8,
  /// Output rate of the NMEA-GX-GNS message on port UART1
  MsgOutNmeaIdGnsUart1, 0x209100b6, u8,
  /// Output rate of the NMEA-GX-GNS message on port UART2
  MsgOutNmeaIdGnsUart2, 0x209100b7, u8,
  /// Output rate of the NMEA-GX-GNS message on port USB
  MsgOutNmeaIdGnsUsb, 0x209100b8, u8,
  /// Output rate of the NMEA-GX-GRS message on port I2C
  MsgOutNmeaIdGrsI2c, 0x209100ce, u8,
  /// Output rate of the NMEA-GX-GRS message on port SPI
  MsgOutNmeaIdGrsSpi, 0x209100d2, u8,
  /// Output rate of the NMEA-GX-GRS message on port UART1
  MsgOutNmeaIdGrsUart1, 0x209100cf, u8,
  /// Output rate of the NMEA-GX-GRS message on port UART2
  MsgOutNmeaIdGrsUart2, 0x209100d0, u8,
  /// Output rate of the NMEA-GX-GRS message on port USB
  MsgOutNmeaIdGrsUsb, 0x209100d1, u8,
  /// Output rate of the NMEA-GX-GSA message on port I2C
  MsgOutNmeaIdGsaI2c, 0x209100bf, u8,
  /// Output rate of the NMEA-GX-GSA message on port SPI
  MsgOutNmeaIdGsaSpi, 0x209100c3, u8,
  /// Output rate of the NMEA-GX-GSA message on port UART1
  MsgOutNmeaIdGsaUart1, 0x209100c0, u8,
  /// Output rate of the NMEA-GX-GSA message on port UART2
  MsgOutNmeaIdGsaUart2, 0x209100c1, u8,
  /// Output rate of the NMEA-GX-GSA message on port USB
  MsgOutNmeaIdGsaUsb, 0x209100c2, u8,
  /// Output rate of the NMEA-GX-GST message on port I2C
  MsgOutNmeaIdGstI2c, 0x209100d3, u8,
  /// Output rate of the NMEA-GX-GST message on port SPI
  MsgOutNmeaIdGstSpi, 0x209100d7, u8,
  /// Output rate of the NMEA-GX-GST message on port UART1
  MsgOutNmeaIdGstUart1, 0x209100d4, u8,
  /// Output rate of the NMEA-GX-GST message on port UART2
  MsgOutNmeaIdGstUart2, 0x209100d5, u8,
  /// Output rate of the NMEA-GX-GST message on port USB
  MsgOutNmeaIdGstUsb, 0x209100d6, u8,
  /// Output rate of the NMEA-GX-GSV message on port I2C
  MsgOutNmeaIdGsvI2c, 0x209100c4, u8,
  /// Output rate of the NMEA-GX-GSV message on port SPI
  MsgOutNmeaIdGsvSpi, 0x209100c8, u8,
  /// Output rate of the NMEA-GX-GSV message on port UART1
  MsgOutNmeaIdGsvUart1, 0x209100c5, u8,
  /// Output rate of the NMEA-GX-GSV message on port UART2
  MsgOutNmeaIdGsvUart2, 0x209100c6, u8,
  /// Output rate of the NMEA-GX-GSV message on port USB
  MsgOutNmeaIdGsvUsb, 0x209100c7, u8,
  /// Output rate of the NMEA-GX-RMC message on port I2C
  MsgOutNmeaIdRmcI2c, 0x209100ab, u8,
  /// Output rate of the NMEA-GX-RMC message on port SPI
  MsgOutNmeaIdRmcSpi, 0x209100af, u8,
  /// Output rate of the NMEA-GX-RMC message on port UART1
  MsgOutNmeaIdRmcUart1, 0x209100ac, u8,
  /// Output rate of the NMEA-GX-RMC message on port UART2
  MsgOutNmeaIdRmcUart2, 0x209100ad, u8,
  /// Output rate of the NMEA-GX-RMC message on port USB
  MsgOutNmeaIdRmcUsb, 0x209100ae, u8,
  /// Output rate of the NMEA-GX-VLW message on port I2C
  MsgOutNmeaIdVlwI2c, 0x209100e7, u8,
  /// Output rate of the NMEA-GX-VLW message on port SPI
  MsgOutNmeaIdVlwSpi, 0x209100eb, u8,
  /// Output rate of the NMEA-GX-VLW message on port UART1
  MsgOutNmeaIdVlwUart1, 0x209100e8, u8,
  /// Output rate of the NMEA-GX-VLW message on port UART2
  MsgOutNmeaIdVlwUart2, 0x209100e9, u8,
  /// Output rate of the NMEA-GX-VLW message on port USB
  MsgOutNmeaIdVlwUsb, 0x209100ea, u8,
  /// Output rate of the NMEA-GX-VTG message on port I2C
  MsgOutNmeaIdVtgI2c, 0x209100b0, u8,
  /// Output rate of the NMEA-GX-VTG message on port SPI
  MsgOutNmeaIdVtgSpi, 0x209100b4, u8,
  /// Output rate of the NMEA-GX-VTG message on port UART1
  MsgOutNmeaIdVtgUart1, 0x209100b1, u8,
  /// Output rate of the NMEA-GX-VTG message on port UART2
  MsgOutNmeaIdVtgUart2, 0x209100b2, u8,
  /// Output rate of the NMEA-GX-VTG message on port USB
  MsgOutNmeaIdVtgUsb, 0x209100b3, u8,
  /// Output rate of the NMEA-GX-ZDA message on port I2C
  MsgOutNmeaIdZdaI2c, 0x209100d8, u8,
  /// Output rate of the NMEA-GX-ZDA message on port SPI
  MsgOutNmeaIdZdaSpi, 0x209100dc, u8,
  /// Output rate of the NMEA-GX-ZDA message on port UART1
  MsgOutNmeaIdZdaUart1, 0x209100d9, u8,
  /// Output rate of the NMEA-GX-ZDA message on port UART2
  MsgOutNmeaIdZdaUart2, 0x209100da, u8,
  /// Output rate of the NMEA-GX-ZDA message on port USB
  MsgOutNmeaIdZdaUsb, 0x209100db, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port I2C
  MsgOutPubxIdPolypI2c, 0x209100ec, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port SPI
  MsgOutPubxIdPolypSpi, 0x209100f0, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port UART1
  MsgOutPubxIdPolypUart1, 0x209100ed, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port UART2
  MsgOutPubxIdPolypUart2, 0x209100ee, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port USB
  MsgOutPubxIdPolypUsb, 0x209100ef, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port I2C
  MsgOutPubxIdPolysI2c, 0x209100f1, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port SPI
  MsgOutPubxIdPolysSpi, 0x209100f5, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port UART1
  MsgOutPubxIdPolysUart1, 0x209100f2, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port UART2
  MsgOutPubxIdPolysUart2, 0x209100f3, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port USB
  MsgOutPubxIdPolysUsb, 0x209100f4, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port I2C
  MsgOutPubxIdPolytI2c, 0x209100f6, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port SPI
  MsgOutPubxIdPolytSpi, 0x209100fa, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port UART1
  MsgOutPubxIdPolytUart1, 0x209100f7, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port UART2
  MsgOutPubxIdPolytUart2, 0x209100f8, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port USB
  MsgOutPubxIdPolytUsb, 0x209100f9, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port I2C
  MsgOutRtcm3Xtype1005I2c, 0x209102bd, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port SPI
  MsgOutRtcm3Xtype1005Spi, 0x209102c1, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port UART1
  MsgOutRtcm3Xtype1005Uart1, 0x209102be, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port UART2
  MsgOutRtcm3Xtype1005Uart2, 0x209102bf, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port USB
  MsgOutRtcm3Xtype1005Usb, 0x209102c0, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port I2C
  MsgOutRtcm3Xtype1074I2c, 0x2091035e, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port SPI
  MsgOutRtcm3Xtype1074Spi, 0x20910362, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port UART1
  MsgOutRtcm3Xtype1074Uart1, 0x2091035f, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port UART2
  MsgOutRtcm3Xtype1074Uart2, 0x20910360, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port USB
  MsgOutRtcm3Xtype1074Usb, 0x20910361, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port I2C
  MsgOutRtcm3Xtype1077I2c, 0x209102cc, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port SPI
  MsgOutRtcm3Xtype1077Spi, 0x209102d0, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port UART1
  MsgOutRtcm3Xtype1077Uart1, 0x209102cd, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port UART2
  MsgOutRtcm3Xtype1077Uart2, 0x209102ce, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port USB
  MsgOutRtcm3Xtype1077Usb, 0x209102cf, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port I2C
  MsgOutRtcm3Xtype1084I2c, 0x20910363, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port SPI
  MsgOutRtcm3Xtype1084Spi, 0x20910367, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port UART1
  MsgOutRtcm3Xtype1084Uart1, 0x20910364, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port UART2
  MsgOutRtcm3Xtype1084Uart2, 0x20910365, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port USB
  MsgOutRtcm3Xtype1084Usb, 0x20910366, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port I2C
  MsgOutRtcm3Xtype1087I2c, 0x209102d1, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port SPI
  MsgOutRtcm3Xtype1087Spi, 0x209102d5, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port UART1
  MsgOutRtcm3Xtype1087Uart1, 0x209102d2, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port UART2
  MsgOutRtcm3Xtype1087Uart2, 0x209102d3, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port USB
  MsgOutRtcm3Xtype1087Usb, 0x209102d4, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port I2C
  MsgOutRtcm3Xtype1094I2c, 0x20910368, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port SPI
  MsgOutRtcm3Xtype1094Spi, 0x2091036c, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port UART1
  MsgOutRtcm3Xtype1094Uart1, 0x20910369, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port UART2
  MsgOutRtcm3Xtype1094Uart2, 0x2091036a, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port USB
  MsgOutRtcm3Xtype1094Usb, 0x2091036b, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port I2C
  MsgOutRtcm3Xtype1097I2c, 0x20910318, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port SPI
  MsgOutRtcm3Xtype1097Spi, 0x2091031c, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port UART1
  MsgOutRtcm3Xtype1097Uart1, 0x20910319, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port UART2
  MsgOutRtcm3Xtype1097Uart2, 0x2091031a, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port USB
  MsgOutRtcm3Xtype1097Usb, 0x2091031b, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port I2C
  MsgOutRtcm3Xtype1124I2c, 0x2091036d, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port SPI
  MsgOutRtcm3Xtype1124Spi, 0x20910371, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port UART1
  MsgOutRtcm3Xtype1124Uart1, 0x2091036e, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port UART2
  MsgOutRtcm3Xtype1124Uart2, 0x2091036f, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port USB
  MsgOutRtcm3Xtype1124Usb, 0x20910370, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port I2C
  MsgOutRtcm3Xtype1127I2c, 0x209102d6, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port SPI
  MsgOutRtcm3Xtype1127Spi, 0x209102da, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port UART1
  MsgOutRtcm3Xtype1127Uart1, 0x209102d7, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port UART2
  MsgOutRtcm3Xtype1127Uart2, 0x209102d8, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port USB
  MsgOutRtcm3Xtype1127Usb, 0x209102d9, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port I2C
  MsgOutRtcm3Xtype1230I2c, 0x20910303, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port SPI
  MsgOutRtcm3Xtype1230Spi, 0x20910307, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port UART1
  MsgOutRtcm3Xtype1230Uart1, 0x20910304, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port UART2
  MsgOutRtcm3Xtype1230Uart2, 0x20910305, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port USB
  MsgOutRtcm3Xtype1230Usb, 0x20910306, u8,
  /// Output rate of the RTCM-3XTYPE4072_0 message on port I2C
  MsgOutRtcm3Xtype40720I2c, 0x209102fe, u8,
  /// Output rate of the RTCM-3XTYPE4072_0 message on port SPI
  MsgOutRtcm3Xtype40720Spi, 0x20910302, u8,
  /// Output rate of the RTCM-3XTYPE4072_0 message on port UART1
  MsgOutRtcm3Xtype40720Uart1, 0x209102ff, u8,
  /// Output rate of the RTCM-3XTYPE4072_0 message on port UART2
  MsgOutRtcm3Xtype40720Uart2, 0x20910300, u8,
  /// Output rate of the RTCM-3XTYPE4072_0 message on port USB
  MsgOutRtcm3Xtype40720Usb, 0x20910301, u8,
  /// Output rate of the RTCM-3XTYPE4072_1 message on port I2C
  MsgOutRtcm3Xtype40721I2c, 0x20910381, u8,
  /// Output rate of the RTCM-3XTYPE4072_1 message on port SPI
  MsgOutRtcm3Xtype40721Spi, 0x20910385, u8,
  /// Output rate of the RTCM-3XTYPE4072_1 message on port UART1
  MsgOutRtcm3Xtype40721Uart1, 0x20910382, u8,
  /// Output rate of the RTCM-3XTYPE4072_1 message on port UART2
  MsgOutRtcm3Xtype40721Uart2, 0x20910383, u8,
  /// Output rate of the RTCM-3XTYPE4072_1 message on port USB
  MsgOutRtcm3Xtype40721Usb, 0x20910384, u8,

  /// Output rate of the UBX-LOG-INFO message on port I2C
  MsgOutUbxLogInfoI2c, 0x20910259, u8,
  /// Output rate of the UBX-LOG-INFO message on port SPI
  MsgOutUbxLogInfoSpi, 0x2091025d, u8,
  /// Output rate of the UBX-LOG-INFO message on port UART1
  MsgOutUbxLogInfoUart1, 0x2091025a, u8,
  /// Output rate of the UBX-LOG-INFO message on port UART2
  MsgOutUbxLogInfoUart2, 0x2091025b, u8,
  /// Output rate of the UBX-LOG-INFO message on port USB
  MsgOutUbxLogInfoUsb, 0x2091025c, u8,
  /// Output rate of the UBX-MONCOMMS message on port I2C
  MsgOutUbxMoncommsI2c, 0x2091034f, u8,
  /// Output rate of the UBX-MONCOMMS message on port SPI
  MsgOutUbxMoncommsSpi, 0x20910353, u8,
  /// Output rate of the UBX-MONCOMMS message on port UART1
  MsgOutUbxMoncommsUart1, 0x20910350, u8,
  /// Output rate of the UBX-MONCOMMS message on port UART2
  MsgOutUbxMoncommsUart2, 0x20910351, u8,
  /// Output rate of the UBX-MONCOMMS message on port USB
  MsgOutUbxMoncommsUsb, 0x20910352, u8,
  /// Output rate of the UBX-MON-HW2 message on port I2C
  MsgOutUbxMonHw2I2c, 0x209101b9, u8,
  /// Output rate of the UBX-MON-HW2 message on port SPI
  MsgOutUbxMonHw2Spi, 0x209101bd, u8,
  /// Output rate of the UBX-MON-HW2 message on port UART1
  MsgOutUbxMonHw2Uart1, 0x209101ba, u8,
  /// Output rate of the UBX-MON-HW2 message on port UART2
  MsgOutUbxMonHw2Uart2, 0x209101bb, u8,
  /// Output rate of the UBX-MON-HW2 message on port USB
  MsgOutUbxMonHw2Usb, 0x209101bc, u8,
  /// Output rate of the UBX-MON-HW3 message on port I2C
  MsgOutUbxMonHw3I2c, 0x20910354, u8,
  /// Output rate of the UBX-MON-HW3 message on port SPI
  MsgOutUbxMonHw3Spi, 0x20910358, u8,
  /// Output rate of the UBX-MON-HW3 message on port UART1
  MsgOutUbxMonHw3Uart1, 0x20910355, u8,
  /// Output rate of the UBX-MON-HW3 message on port UART2
  MsgOutUbxMonHw3Uart2, 0x20910356, u8,
  /// Output rate of the UBX-MON-HW3 message on port USB
  MsgOutUbxMonHw3Usb, 0x20910357, u8,
  /// Output rate of the UBX-MON-HW message on port I2C
  MsgOutUbxMonHwI2c, 0x209101b4, u8,
  /// Output rate of the UBX-MON-HW message on port SPI
  MsgOutUbxMonHwSpi, 0x209101b8, u8,
  /// Output rate of the UBX-MON-HW message on port UART1
  MsgOutUbxMonHwUart1, 0x209101b5, u8,
  /// Output rate of the UBX-MON-HW message on port UART2
  MsgOutUbxMonHwUart2, 0x209101b6, u8,
  /// Output rate of the UBX-MON-HW message on port USB
  MsgOutUbxMonHwUsb, 0x209101b7, u8,
  /// Output rate of the UBX-MON-IO message on port I2C
  MsgOutUbxMonIoI2c, 0x209101a5, u8,
  /// Output rate of the UBX-MON-IO message on port SPI
  MsgOutUbxMonIoSpi, 0x209101a9, u8,
  /// Output rate of the UBX-MON-IO message on port UART1
  MsgOutUbxMonIoUart1, 0x209101a6, u8,
  /// Output rate of the UBX-MON-IO message on port UART2
  MsgOutUbxMonIoUart2, 0x209101a7, u8,
  /// Output rate of the UBX-MON-IO message on port USB
  MsgOutUbxMonIoUsb, 0x209101a8, u8,
  /// Output rate of the UBX-MON-MSGPP message on port I2C
  MsgOutUbxMonMsgppI2c, 0x20910196, u8,
  /// Output rate of the UBX-MON-MSGPP message on port SPI
  MsgOutUbxMonMsgppSpi, 0x2091019a, u8,
  /// Output rate of the UBX-MON-MSGPP message on port UART1
  MsgOutUbxMonMsgppUart1, 0x20910197, u8,
  /// Output rate of the UBX-MON-MSGPP message on port UART2
  MsgOutUbxMonMsgppUart2, 0x20910198, u8,
  /// Output rate of the UBX-MON-MSGPP message on port USB
  MsgOutUbxMonMsgppUsb, 0x20910199, u8,
  /// Output rate of the UBX-MON-RF message on port I2C
  MsgOutUbxMonRfI2c, 0x20910359, u8,
  /// Output rate of the UBX-MON-RF message on port SPI
  MsgOutUbxMonRfSpi, 0x2091035d, u8,
  /// Output rate of the UBX-MON-RF message on port UART1
  MsgOutUbxMonRfUart1, 0x2091035a, u8,
  /// Output rate of the UBX-MON-RF message on port UART2
  MsgOutUbxMonRfUart2, 0x2091035b, u8,
  /// Output rate of the UBX-MON-RF message on port USB
  MsgOutUbxMonRfUsb, 0x2091035c, u8,
  /// Output rate of the UBX-MON-RXBUF message on port I2C
  MsgOutUbxMonRxbufI2c, 0x209101a0, u8,
  /// Output rate of the UBX-MON-RXBUF message on port SPI
  MsgOutUbxMonRxbufSpi, 0x209101a4, u8,
  /// Output rate of the UBX-MON-RXBUF message on port UART1
  MsgOutUbxMonRxbufUart1, 0x209101a1, u8,
  /// Output rate of the UBX-MON-RXBUF message on port UART2
  MsgOutUbxMonRxbufUart2, 0x209101a2, u8,
  /// Output rate of the UBX-MON-RXBUF message on port USB
  MsgOutUbxMonRxbufUsb, 0x209101a3, u8,
  /// Output rate of the UBX-MON-RXR message on port I2C
  MsgOutUbxMonRxrI2c, 0x20910187, u8,
  /// Output rate of the UBX-MON-RXR message on port SPI
  MsgOutUbxMonRxrSpi, 0x2091018b, u8,
  /// Output rate of the UBX-MON-RXR message on port UART1
  MsgOutUbxMonRxrUart1, 0x20910188, u8,
  /// Output rate of the UBX-MON-RXR message on port UART2
  MsgOutUbxMonRxrUart2, 0x20910189, u8,
  /// Output rate of the UBX-MON-RXR message on port USB
  MsgOutUbxMonRxrUsb, 0x2091018a, u8,
  /// Output rate of the UBX-MON-TXBUF message on port I2C
  MsgOutUbxMonTxbufI2c, 0x2091019b, u8,
  /// Output rate of the UBX-MON-TXBUF message on port SPI
  MsgOutUbxMonTxbufSpi, 0x2091019f, u8,
  /// Output rate of the UBX-MON-TXBUF message on port UART1
  MsgOutUbxMonTxbufUart1, 0x2091019c, u8,
  /// Output rate of the UBX-MON-TXBUF message on port UART2
  MsgOutUbxMonTxbufUart2, 0x2091019d, u8,
  /// Output rate of the UBX-MON-TXBUF message on port USB
  MsgOutUbxMonTxbufUsb, 0x2091019e, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port I2C
  MsgOutUbxNavClockI2c, 0x20910065, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port SPI
  MsgOutUbxNavClockSpi, 0x20910069, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port UART1
  MsgOutUbxNavClockUart1, 0x20910066, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port UART2
  MsgOutUbxNavClockUart2, 0x20910067, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port USB
  MsgOutUbxNavClockUsb, 0x20910068, u8,
  /// Output rate of the UBX-NAV-DOP message on port I2C
  MsgOutUbxNavDopI2c, 0x20910038, u8,
  /// Output rate of the UBX-NAV-DOP message on port SPI
  MsgOutUbxNavDopSpi, 0x2091003c, u8,
  /// Output rate of the UBX-NAV-DOP message on port UART1
  MsgOutUbxNavDopUart1, 0x20910039, u8,
  /// Output rate of the UBX-NAV-DOP message on port UART2
  MsgOutUbxNavDopUart2, 0x2091003a, u8,
  /// Output rate of the UBX-NAV-DOP message on port USB
  MsgOutUbxNavDopUsb, 0x2091003b, u8,
  /// Output rate of the UBX-NAV-EOE message on port I2C
  MsgOutUbxNavEoeI2c, 0x2091015f, u8,
  /// Output rate of the UBX-NAV-EOE message on port SPI
  MsgOutUbxNavEoeSpi, 0x20910163, u8,
  /// Output rate of the UBX-NAV-EOE message on port UART1
  MsgOutUbxNavEoeUart1, 0x20910160, u8,
  /// Output rate of the UBX-NAV-EOE message on port UART2
  MsgOutUbxNavEoeUart2, 0x20910161, u8,
  /// Output rate of the UBX-NAV-EOE message on port USB
  MsgOutUbxNavEoeUsb, 0x20910162, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port I2C
  MsgOutUbxNavGeofenceI2c, 0x209100a1, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port SPI
  MsgOutUbxNavGeofenceSpi, 0x209100a5, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port UART1
  MsgOutUbxNavGeofenceUart1, 0x209100a2, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port UART2
  MsgOutUbxNavGeofenceUart2, 0x209100a3, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port USB
  MsgOutUbxNavGeofenceUsb, 0x209100a4, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port I2C
  MsgOutUbxNavHpPosEcefI2c, 0x2091002e, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port SPI
  MsgOutUbxNavHpPosEcefSpi, 0x20910032, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port UART1
  MsgOutUbxNavHpPosEcefUart1, 0x2091002f, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port UART2
  MsgOutUbxNavHpPosEcefUart2, 0x20910030, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port USB
  MsgOutUbxNavHpPosEcefUsb, 0x20910031, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port I2C
  MsgOutUbxNavHpPosLlhI2c, 0x20910033, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port SPI
  MsgOutUbxNavHpPosLlhSpi, 0x20910037, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port UART1
  MsgOutUbxNavHpPosLlhUart1, 0x20910034, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port UART2
  MsgOutUbxNavHpPosLlhUart2, 0x20910035, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port USB
  MsgOutUbxNavHpPosLlhUsb, 0x20910036, u8,
  /// Output rate of the UBX-NAV-ODO message on port I2C
  MsgOutUbxNavOdoI2C, 0x2091007e, u8,
  /// Output rate of the UBX-NAV-ODO message on port SPI
  MsgOutUbxNavOdoSpi, 0x20910082, u8,
  /// Output rate of the UBX-NAV-ODO message on port UART1
  MsgOutUbxNavOdoUart1, 0x2091007f, u8,
  /// Output rate of the UBX-NAV-ODO message on port UART2
  MsgOutUbxNavOdoUart2, 0x20910080, u8,
  /// Output rate of the UBX-NAV-ODO message on port USB
  MsgOutUbxNavOdoUsb, 0x20910081, u8,
  /// Output rate of the UBX-NAV-ORB message on port I2C
  MsgOutUbxNavOrbI2c, 0x20910010, u8,
  /// Output rate of the UBX-NAV-ORB message on port SPI
  MsgOutUbxNavOrbSpi, 0x20910014, u8,
  /// Output rate of the UBX-NAV-ORB message on port UART1
  MsgOutUbxNavOrbUart1, 0x20910011, u8,
  /// Output rate of the UBX-NAV-ORB message on port UART2
  MsgOutUbxNavOrbUart2, 0x20910012, u8,
  /// Output rate of the UBX-NAV-ORB message on port USB
  MsgOutUbxNavOrbUsb, 0x20910013, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port I2C
  MsgOutUbxNavPosEcefI2c, 0x20910024, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port SPI
  MsgOutUbxNavPosEcefSpi, 0x20910028, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port UART1
  MsgOutUbxNavPosEcefUart1, 0x20910025, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port UART2
  MsgOutUbxNavPosEcefUart2, 0x20910026, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port USB
  MsgOutUbxNavPosEcefUsb, 0x20910027, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port I2C
  MsgOutUbxNavPosLlhI2c, 0x20910029, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port SPI
  MsgOutUbxNavPosLlhSpi, 0x2091002d, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port UART1
  MsgOutUbxNavPosLlhUart1, 0x2091002a, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port UART2
  MsgOutUbxNavPosLlhUart2, 0x2091002b, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port USB
  MsgOutUbxNavPosLlhUsb, 0x2091002c, u8,
  /// Output rate of the UBX-NAV-PVT message on port I2C
  MsgOutUbxNavPvtI2c, 0x20910006, u8,
  /// Output rate of the UBX-NAV-PVT message on port SPI
  MsgOutUbxNavPvtSpi, 0x2091000a, u8,
  /// Output rate of the UBX-NAV-PVT message on port UART1
  MsgOutUbxNavPvtUart1, 0x20910007, u8,
  /// Output rate of the UBX-NAV-PVT message on port UART2
  MsgOutUbxNavPvtUart2, 0x20910008, u8,
  /// Output rate of the UBX-NAV-PVT message on port USB
  MsgOutUbxNavPvtUsb, 0x20910009, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port I2C
  MsgOutUbxNavRelposNedI2c, 0x2091008d, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port SPI
  MsgOutUbxNavRelposNedSpi, 0x20910091, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port UART1
  MsgOutUbxNavRelposNedUart1, 0x2091008e, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port UART2
  MsgOutUbxNavRelposNedUart2, 0x2091008f, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port USB
  MsgOutUbxNavRelposNedUsb, 0x20910090, u8,
  /// Output rate of the UBX-NAV-SAT message on port I2C
  MsgOutUbxNavSatI2c, 0x20910015, u8,
  /// Output rate of the UBX-NAV-SAT message on port SPI
  MsgOutUbxNavSatSpi, 0x20910019, u8,
  /// Output rate of the UBX-NAV-SAT message on port UART1
  MsgOutUbxNavSatUart1, 0x20910016, u8,
  /// Output rate of the UBX-NAV-SAT message on port UART2
  MsgOutUbxNavSatUart2, 0x20910017, u8,
  /// Output rate of the UBX-NAV-SAT message on port USB
  MsgOutUbxNavSatUsb, 0x20910018, u8,
  /// Output rate of the UBX-NAV-SIG message on port I2C
  MsgOutUbxNavSigI2c, 0x20910345, u8,
  /// Output rate of the UBX-NAV-SIG message on port SPI
  MsgOutUbxNavSigSpi, 0x20910349, u8,
  /// Output rate of the UBX-NAV-SIG message on port UART1
  MsgOutUbxNavSigUart1, 0x20910346, u8,
  /// Output rate of the UBX-NAV-SIG message on port UART2
  MsgOutUbxNavSigUart2, 0x20910347, u8,
  /// Output rate of the UBX-NAV-SIG message on port USB
  MsgOutUbxNavSigUsb, 0x20910348, u8,
  /// Output rate of the UBX-NAV-STATUS message on port I2C
  MsgOutUbxNavStatusI2c, 0x2091001a, u8,
  /// Output rate of the UBX-NAV-STATUS message on port SPI
  MsgOutUbxNavStatusSpi, 0x2091001e, u8,
  /// Output rate of the UBX-NAV-STATUS message on port UART1
  MsgOutUbxNavStatusUart1, 0x2091001b, u8,
  /// Output rate of the UBX-NAV-STATUS message on port UART2
  MsgOutUbxNavStatusUart2, 0x2091001c, u8,
  /// Output rate of the UBX-NAV-STATUS message on port USB
  MsgOutUbxNavStatusUsb, 0x2091001d, u8,
  /// Output rate of the UBX-NAV-SVIN message on port I2C
  MsgOutUbxNavSvinI2c, 0x20910088, u8,
  /// Output rate of the UBX-NAV-SVIN message on port SPI
  MsgOutUbxNavSvinSpi, 0x2091008c, u8,
  /// Output rate of the UBX-NAV-SVIN message on port UART1
  MsgOutUbxNavSvinUart1, 0x20910089, u8,
  /// Output rate of the UBX-NAV-SVIN message on port UART2
  MsgOutUbxNavSvinUart2, 0x2091008a, u8,
  /// Output rate of the UBX-NAV-SVIN message on port USB
  MsgOutUbxNavSvinUsb, 0x2091008b, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port I2C
  MsgOutUbxNavTimeBdsI2c, 0x20910051, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port SPI
  MsgOutUbxNavTimeBdsSpi, 0x20910055, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port UART1
  MsgOutUbxNavTimeBdsUart1, 0x20910052, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port UART2
  MsgOutUbxNavTimeBdsUart2, 0x20910053, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port USB
  MsgOutUbxNavTimeBdsUsb, 0x20910054, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port I2C
  MsgOutUbxNavTimeGalI2c, 0x20910056, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port SPI
  MsgOutUbxNavTimeGalSpi, 0x2091005a, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port UART1
  MsgOutUbxNavTimeGalUart1, 0x20910057, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port UART2
  MsgOutUbxNavTimeGalUart2, 0x20910058, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port USB
  MsgOutUbxNavTimeGalUsb, 0x20910059, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port I2C
  MsgOutUbxNavTimeGloI2c, 0x2091004c, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port SPI
  MsgOutUbxNavTimeGloSpi, 0x20910050, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port UART1
  MsgOutUbxNavTimeGloUart1, 0x2091004d, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port UART2
  MsgOutUbxNavTimeGloUart2, 0x2091004e, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port USB
  MsgOutUbxNavTimeGloUsb, 0x2091004f, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port I2C
  MsgOutUbxNavTimeGpsI2c, 0x20910047, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port SPI
  MsgOutUbxNavTimeGpsSpi, 0x2091004b, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port UART1
  MsgOutUbxNavTimeGpsUart1, 0x20910048, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port UART2
  MsgOutUbxNavTimeGpsUart2, 0x20910049, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port USB
  MsgOutUbxNavTimeGpsUsb, 0x2091004a, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port I2C
  MsgOutUbxNavTimeLsI2c, 0x20910060, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port SPI
  MsgOutUbxNavTimeLsSpi, 0x20910064, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port UART1
  MsgOutUbxNavTimeLsUart1, 0x20910061, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port UART2
  MsgOutUbxNavTimeLsUart2, 0x20910062, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port USB
  MsgOutUbxNavTimeLsUsb, 0x20910063, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port I2C
  MsgOutUbxNavTimeUtcI2c, 0x2091005b, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port SPI
  MsgOutUbxNavTimeUtcSpi, 0x2091005f, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port UART1
  MsgOutUbxNavTimeUtcUart1, 0x2091005c, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port UART2
  MsgOutUbxNavTimeUtcUart2, 0x2091005d, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port USB
  MsgOutUbxNavTimeUtcUsb, 0x2091005e, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port I2C
  MsgOutUbxNavVelEcefI2c, 0x2091003d, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port SPI
  MsgOutUbxNavVelEcefSpi, 0x20910041, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port UART1
  MsgOutUbxNavVelEcefUart1, 0x2091003e, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port UART2
  MsgOutUbxNavVelEcefUart2, 0x2091003f, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port USB
  MsgOutUbxNavVelEcefUsb, 0x20910040, u8,
  /// Output rate of the UBX-NAV-VELNED message on port I2C
  MsgOutUbxNavVelNedI2c, 0x20910042, u8,
  /// Output rate of the UBX-NAV-VELNED message on port SPI
  MsgOutUbxNavVelNedSpi, 0x20910046, u8,
  /// Output rate of the UBX-NAV-VELNED message on port UART1
  MsgOutUbxNavVelNedUart1, 0x20910043, u8,
  /// Output rate of the UBX-NAV-VELNED message on port UART2
  MsgOutUbxNavVelNedUart2, 0x20910044, u8,
  /// Output rate of the UBX-NAV-VELNED message on port USB
  MsgOutUbxNavVelNedUsb, 0x20910045, u8,
  /// Output rate of the UBX-RXM-MEASX message on port I2C
  MsgOutUbxRxmMeasxI2c, 0x20910204, u8,
  /// Output rate of the UBX-RXM-MEASX message on port SPI
  MsgOutUbxRxmMeasxSpi, 0x20910208, u8,
  /// Output rate of the UBX-RXM-MEASX message on port UART1
  MsgOutUbxRxmMeasxUart1, 0x20910205, u8,
  /// Output rate of the UBX-RXM-MEASX message on port UART2
  MsgOutUbxRxmMeasxUart2, 0x20910206, u8,
  /// Output rate of the UBX-RXM-MEASX message on port USB
  MsgOutUbxRxmMeasxUsb, 0x20910207, u8,
  /// Output rate of the UBX-RXM-RAWX message on port I2C
  MsgOutUbxRxmRawxI2c, 0x209102a4, u8,
  /// Output rate of the UBX-RXM-RAWX message on port SPI
  MsgOutUbxRxmRawxSpi, 0x209102a8, u8,
  /// Output rate of the UBX-RXM-RAWX message on port UART1
  MsgOutUbxRxmRawxUart1, 0x209102a5, u8,
  /// Output rate of the UBX-RXM-RAWX message on port UART2
  MsgOutUbxRxmRawxUart2, 0x209102a6, u8,
  /// Output rate of the UBX-RXM-RAWX message on port USB
  MsgOutUbxRxmRawxUsb, 0x209102a7, u8,
  /// Output rate of the UBX-RXM-RLM message on port I2C
  MsgOutUbxRxmRlmI2c, 0x2091025e, u8,
  /// Output rate of the UBX-RXM-RLM message on port SPI
  MsgOutUbxRxmRlmSpi, 0x20910262, u8,
  /// Output rate of the UBX-RXM-RLM message on port UART1
  MsgOutUbxRxmRlmUart1, 0x2091025f, u8,
  /// Output rate of the UBX-RXM-RLM message on port UART2
  MsgOutUbxRxmRlmUart2, 0x20910260, u8,
  /// Output rate of the UBX-RXM-RLM message on port USB
  MsgOutUbxRxmRlmUsb, 0x20910261, u8,
  /// Output rate of the UBX-RXM-RTCM message on port I2C
  MsgOutUbxRxmRtcmI2c, 0x20910268, u8,
  /// Output rate of the UBX-RXM-RTCM message on port SPI
  MsgOutUbxRxmRtcmSpi, 0x2091026c, u8,
  /// Output rate of the UBX-RXM-RTCM message on port UART1
  MsgOutUbxRxmRtcmUart1, 0x20910269, u8,
  /// Output rate of the UBX-RXM-RTCM message on port UART2
  MsgOutUbxRxmRtcmUart2, 0x2091026a, u8,
  /// Output rate of the UBX-RXM-RTCM message on port USB
  MsgOutUbxRxmRtcmUsb, 0x2091026b, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port I2C
  MsgOutUbxRxmSfrbxI2c, 0x20910231, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port SPI
  MsgOutUbxRxmSfrbxSpi, 0x20910235, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port UART1
  MsgOutUbxRxmSfrbxUart1, 0x20910232, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port UART2
  MsgOutUbxRxmSfrbxUart2, 0x20910233, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port USB
  MsgOutUbxRxmSfrbxUsb, 0x20910234, u8,
  /// Output rate of the UBX-TIM-TM2 message on port I2C
  MsgOutUbxTimTm2I2c, 0x20910178, u8,
  /// Output rate of the UBX-TIM-TM2 message on port SPI
  MsgOutUbxTimTm2Spi, 0x2091017c, u8,
  /// Output rate of the UBX-TIM-TM2 message on port UART1
  MsgOutUbxTimTm2Uart1, 0x20910179, u8,
  /// Output rate of the UBX-TIM-TM2 message on port UART2
  MsgOutUbxTimTm2Uart2, 0x2091017a, u8,
  /// Output rate of the UBX-TIM-TM2 message on port USB
  MsgOutUbxTimTm2Usb, 0x2091017b, u8,
  /// Output rate of the UBX-TIM-TP message on port I2C
  MsgOutUbxTimTpI2c, 0x2091017d, u8,
  /// Output rate of the UBX-TIM-TP message on port SPI
  MsgOutUbxTimTpSpi, 0x20910181, u8,
  /// Output rate of the UBX-TIM-TP message on port UART1
  MsgOutUbxTimTpUart1, 0x2091017e, u8,
  /// Output rate of the UBX-TIM-TP message on port UART2
  MsgOutUbxTimTpUart2, 0x2091017f, u8,
  /// Output rate of the UBX-TIM-TP message on port USB
  MsgOutUbxTimTpUsb, 0x20910180, u8,
  /// Output rate of the UBX-TIM-VRFY message on port I2C
  MsgOutUbxTimVrfyI2c, 0x20910092, u8,
  /// Output rate of the UBX-TIM-VRFY message on port SPI
  MsgOutUbxTimVrfySpi, 0x20910096, u8,
  /// Output rate of the UBX-TIM-VRFY message on port UART1
  MsgOutUbxTimVrfyUart1, 0x20910093, u8,
  /// Output rate of the UBX-TIM-VRFY message on port UART2
  MsgOutUbxTimVrfyUart2, 0x20910094, u8,
  /// Output rate of the UBX-TIM-VRFY message on port USB
  MsgOutUbxTimVrfyUsb, 0x20910095, u8,

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

  /// "Undocumented" L5 Health Bit Ignore (see
  /// <https://content.u-blox.com/sites/default/files/documents/GPS-L5-configuration_AppNote_UBX-21038688.pdf>)
  UndocumentedL5Enable,  0x10320001, bool,

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

  /// CFG-TMode* (Time-only mode settings - position fixed)
  /// Receiver mode
  TModeModeDef, 0x20030001,  CfgTModeModes,
  /// Determines whether the Antenna Reference Point (ARP) position is given in ECEF or LAT/LON/HEIGHT?
  TModePosTypeDef, 0x20030002,  TModePosType,
  /// ECEF X coordinate of the ARP position in \[cm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefX, 0x40030003,  i32,
  /// ECEF Y coordinate of the ARP position in \[cm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefY, 0x40030004,  i32,
  /// ECEF Z coordinate of the ARP position in \[cm\] .
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefZ, 0x40030005,  i32,
  /// High-precision ECEF X coordinate of the ARP position, [-99 to +99] in \[mm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefXHp, 0x20030006,  i8,
  /// High-precision ECEF Y coordinate of the ARP position [-99 to +99] in \[mm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefYHp, 0x20030007,  i8,
  /// High-precision ECEF Z coordinate of the ARP position [-99 to +99] in \[mm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=ECEF.
  TModeEcefZHp, 0x20030008,  i8,
  /// Latitude of the ARP position in \[deg\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeLat, 0x40030009,  i32,
  /// Longitude of the ARP position in \[deg\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeLon, 0x4003000a,  i32,
  /// Height of the ARP position in \[cm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeHeight, 0x4003000b,  i32,
  /// High-precision latitude of the ARP position [-99 to +99] in \[deg\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeLatHp, 0x2003000c,  i8,
  /// High-precision longitude of the ARP position [-99 to +99] in \[deg\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeLonHp, 0x2003000d,  i8,
  /// High-precision height of the ARP position [-99 to +99] in \[mm\].
  /// This will only be used if CfgTModeModes=Fixed and TModePOS_TYPE=LLH.
  TModeHeightHp, 0x2003000e,  i8,
  /// Fixed position 3D accuracy in \[mm\]
  TModeFixedPosAcc, 0x4003000f,  u32,
  /// Survey-in minimum duration in \[sec\].
  /// This will only be used if CfgTModeModes=SurveyIn.
  TModeSvInMinDur, 0x40030010,  u32,
  /// Survey-in position accuracy limit in \[mm\]
  /// This will only be used if CfgTModeModes=SurveyIn.
  TModeSvInAccLimit, 0x40030011,  u32,

  /// CFG-NAVSPG -*: Standard Precision Navigation Configuration

  /// Position fix mode
  NavSpgFixMode, 0x20110011, NavFixMode,
  /// Initial fix must be a 3d fix
  NavSpgIniFix3D, 0x10110013, bool,
  /// GPS week rollover number
  ///
  /// GPS week numbers will be set correctly from this week up to 1024 weeks after this week.
  /// Range is from 1 to 4096.
  NavSpgWknRollover, 0x30110017, u16,
  /// Use Precise Point Positioning
  ///
  /// Only available with the PPP product variant.
  NavSpgUsePPP, 0x10110019, bool,
  /// UTC standard to be used
  NavSpgUtcStandard, 0x2011001c, UtcStandardIdentifier,
  /// Dynamic platform model
  NavSpgDynModel, 0x20110021, NavDynamicModel,
  /// Acknowledge assistance input messages
  NavSpgAckAiding, 0x10110025, bool,
  /// Use user geodetic datum parameters
  ///
  /// This must be set together with all CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUseUsrDat, 0x10110061, bool,
  /// Geodetic datum semi-major axis
  ///
  /// Accepted range is from 6,300,000.0 to 6,500,000.0 meters.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatMaja, 0x50110062, f64,
  /// Geodetic datum 1.0 / flattening
  ///
  /// Accepted range is 0.0 to 500.0.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatFlat, 0x50110063, f64,
  /// Geodetic datum X axis shift at the origin
  ///
  /// Accepted range is +/- 5000.0 meters.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatDx, 0x40110064, f32,
  /// Geodetic datum Y axis shift at the origin
  ///
  /// Accepted range is +/- 5000.0 meters.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatDy, 0x40110065, f32,
  /// Geodetic datum Z axis shift at the origin
  ///
  /// Accepted range is +/- 5000.0 meters.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatDz, 0x40110066, f32,
  /// Geodetic datum rotation about the X axis
  ///
  /// Accepted range is +/- 20.0 milli arc seconds.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatRotX, 0x40110067, f32,
  /// Geodetic datum rotation about the Y axis
  ///
  /// Accepted range is +/- 20.0 milli arc seconds.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatRotY, 0x40110068, f32,
  /// Geodetic datum rotation about the Z axis
  ///
  /// Accepted range is +/- 20.0 milli-arc seconds.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatRotZ, 0x40110069, f32,

  /// Geodetic datum scale factor
  ///
  /// Accepted range is 0.0 to 50.0 parts per million.
  /// This will only be used if CFG-NAVSPG-USE_USERDAT is set. It must be set together with all other
  /// CFG-NAVSPG-USERDAT_* parameters.
  NavSpgUsrDatScale, 0x4011006a, f32,
  /// Minimum number of satellites for navigation
  NavSpgInfilMinSvs, 0x201100a1, u8,
  /// Maximum number of satellites for navigation
  NavSpgInfilMaxSvs, 0x201100a2, u8,
  /// Minimum satellite signal level for navigation (dBHz)
  NavSpgInfilMinCno, 0x201100a3, u8,
  /// Minimum elevation for a GNSS satellite to be used in navigation (degrees)
  NavSpgInfilMinElev, 0x201100a4, i8,

  /// Number of satellites required to have C/N0 above CFG-NAVSPG-INFIL_CNOTHRS for a fix to be attempted
  NavSpgInfilNcnoThrs, 0x201100aa, u8,

  /// C/N0 threshold for deciding whether to attempt a fix
  NavSpgInfilCnoThrs, 0x201100ab, u8,

  /// Output filter position DOP mask (threshold)
  NavSpgOutfilPdop, 0x301100b1, u16,

  /// Output filter time DOP mask (threshold)
  NavSpgOutfilTdop, 0x301100b2, u16,

  /// Output filter position accuracy mask (threshold) in meters
  NavSpgOutfilPacc, 0x301100b3, u16,

  /// Output filter time accuracy mask (threshold) in meters
  NavSpgOutfilTacc, 0x301100b4, u16,

  /// Output filter frequency accuracy mask (threshold) in m/s
  NavSpgOutfilFacc, 0x301100b5, u16,

  /// Fixed altitude (mean sea level) for 2D fix mode (0.01 m resolution)
  NavSpgConstrAlt, 0x401100c1, i32,

  /// Fixed altitude variance for 2D mode (0.0001 m² resolution)
  NavSpgConstrAltVar, 0x401100c2, u32,
  /// DGNSS timeout in seconds
  NavSpgConstrDgnssTo, 0x201100c4, u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TpPulse {
    /// Time pulse period
    Period = 0,
    /// Time pulse frequency
    Freq = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TpPulseLength {
    /// Time pulse ratio
    Ratio = 0,
    /// Time pulse length
    Length = 1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TModePosType {
    /// ECEF position
    ECEF = 0,
    /// Lat/Lon/Height position
    LLH = 1,
}
