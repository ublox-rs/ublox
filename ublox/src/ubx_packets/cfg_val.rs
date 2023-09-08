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

            // TODO: Replace this with unreachable!() when we upgrade to MSRV 1.57
            // Since it's unreachable we get to pick an arbitrary value
            //_ => unreachable!(),
            _ => StorageSize::OneBit,
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
    ($buf:expr, u64) => {
        u64::from_le_bytes([
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
    ($this:expr, u64) => {{
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
}

macro_rules! cfg_val {
  (
    $(
      $(#[$class_comment:meta])*
      $cfg_item:ident, $cfg_key_id:expr, $cfg_value_type:ident,
    )*
  ) => {
    #[derive(Debug, Clone)]
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
              // TODO: extend all the bytes in one extend() call when we bump MSRV
              for b in bytes.iter() {
                buf.extend(core::iter::once(*b));
              }
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
  UsbinprotUbx,         0x10770001, bool,
  UsbinprotNmea,        0x10770002, bool,
  UsbinprotRtcm3X,      0x10770004, bool,

  // CFG-USBOUTPROT-*
  UsbOutProtUbx,        0x10780001, bool,
  UsbOutProtNmea,       0x10780002, bool,
  UsbOutProtRtcm3x,     0x10780004, bool,

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
  /// Nominal time between GNSS measurements
  /// (e.g. 100ms results in 10Hz measurement rate, 1000ms = 1Hz measurement rate)
  RateMeas,              0x30210001, u16,
  /// Ratio of number of measurements to number of navigation solutions
  RateNav,               0x30210002, u16,
  /// Time system to which measurements are aligned
  RateTimeref,           0x20210003, AlignmentToReferenceTime,

  // CFG-MSGOUT-*
  /// Output rate of the NMEA-GX-DTM message on port I2C
  MsgoutNmeaIdDtmI2c, 0x209100a6, u8,
  /// Output rate of the NMEA-GX-DTM message on port SPI
  MsgoutNmeaIdDtmSpi, 0x209100aa, u8,
  /// Output rate of the NMEA-GX-DTM message on port UART1
  MsgoutNmeaIdDtmUart1, 0x209100a7, u8,
  /// Output rate of the NMEA-GX-DTM message on port UART2
  MsgoutNmeaIdDtmUart2, 0x209100a8, u8,
  /// Output rate of the NMEA-GX-DTM message on port USB
  MsgoutNmeaIdDtmUsb, 0x209100a9, u8,
  /// Output rate of the NMEA-GX-GBS message on port I2C
  MsgoutNmeaIdGbsI2c, 0x209100dd, u8,
  /// Output rate of the NMEA-GX-GBS message on port SPI
  MsgoutNmeaIdGbsSpi, 0x209100e1, u8,
  /// Output rate of the NMEA-GX-GBS message on port UART1
  MsgoutNmeaIdGbsUart1, 0x209100de, u8,
  /// Output rate of the NMEA-GX-GBS message on port UART2
  MsgoutNmeaIdGbsUart2, 0x209100df, u8,
  /// Output rate of the NMEA-GX-GBS message on port USB
  MsgoutNmeaIdGbsUsb, 0x209100e0, u8,
  /// Output rate of the NMEA-GX-GGA message on port I2C
  MsgoutNmeaIdGgaI2c, 0x209100ba, u8,
  /// Output rate of the NMEA-GX-GGA message on port SPI
  MsgoutNmeaIdGgaSpi, 0x209100be, u8,
  /// Output rate of the NMEA-GX-GGA message on port UART1
  MsgoutNmeaIdGgaUart1, 0x209100bb, u8,
  /// Output rate of the NMEA-GX-GGA message on port UART2
  MsgoutNmeaIdGgaUart2, 0x209100bc, u8,
  /// Output rate of the NMEA-GX-GGA message on port USB
  MsgoutNmeaIdGgaUsb, 0x209100bd, u8,
  /// Output rate of the NMEA-GX-GLL message on port I2C
  MsgoutNmeaIdGllI2c, 0x209100c9, u8,
  /// Output rate of the NMEA-GX-GLL message on port SPI
  MsgoutNmeaIdGllSpi, 0x209100cd, u8,
  /// Output rate of the NMEA-GX-GLL message on port UART1
  MsgoutNmeaIdGllUart1, 0x209100ca, u8,
  /// Output rate of the NMEA-GX-GLL message on port UART2
  MsgoutNmeaIdGllUart2, 0x209100cb, u8,
  /// Output rate of the NMEA-GX-GLL message on port USB
  MsgoutNmeaIdGllUsb, 0x209100cc, u8,
  /// Output rate of the NMEA-GX-GNS message on port I2C
  MsgoutNmeaIdGnsI2c, 0x209100b5, u8,
  /// Output rate of the NMEA-GX-GNS message on port SPI
  MsgoutNmeaIdGnsSpi, 0x209100b9, u8,
  /// Output rate of the NMEA-GX-GNS message on port UART1
  MsgoutNmeaIdGnsUart1, 0x209100b6, u8,
  /// Output rate of the NMEA-GX-GNS message on port UART2
  MsgoutNmeaIdGnsUart2, 0x209100b7, u8,
  /// Output rate of the NMEA-GX-GNS message on port USB
  MsgoutNmeaIdGnsUsb, 0x209100b8, u8,
  /// Output rate of the NMEA-GX-GRS message on port I2C
  MsgoutNmeaIdGrsI2c, 0x209100ce, u8,
  /// Output rate of the NMEA-GX-GRS message on port SPI
  MsgoutNmeaIdGrsSpi, 0x209100d2, u8,
  /// Output rate of the NMEA-GX-GRS message on port UART1
  MsgoutNmeaIdGrsUart1, 0x209100cf, u8,
  /// Output rate of the NMEA-GX-GRS message on port UART2
  MsgoutNmeaIdGrsUart2, 0x209100d0, u8,
  /// Output rate of the NMEA-GX-GRS message on port USB
  MsgoutNmeaIdGrsUsb, 0x209100d1, u8,
  /// Output rate of the NMEA-GX-GSA message on port I2C
  MsgoutNmeaIdGsaI2c, 0x209100bf, u8,
  /// Output rate of the NMEA-GX-GSA message on port SPI
  MsgoutNmeaIdGsaSpi, 0x209100c3, u8,
  /// Output rate of the NMEA-GX-GSA message on port UART1
  MsgoutNmeaIdGsaUart1, 0x209100c0, u8,
  /// Output rate of the NMEA-GX-GSA message on port UART2
  MsgoutNmeaIdGsaUart2, 0x209100c1, u8,
  /// Output rate of the NMEA-GX-GSA message on port USB
  MsgoutNmeaIdGsaUsb, 0x209100c2, u8,
  /// Output rate of the NMEA-GX-GST message on port I2C
  MsgoutNmeaIdGstI2c, 0x209100d3, u8,
  /// Output rate of the NMEA-GX-GST message on port SPI
  MsgoutNmeaIdGstSpi, 0x209100d7, u8,
  /// Output rate of the NMEA-GX-GST message on port UART1
  MsgoutNmeaIdGstUart1, 0x209100d4, u8,
  /// Output rate of the NMEA-GX-GST message on port UART2
  MsgoutNmeaIdGstUart2, 0x209100d5, u8,
  /// Output rate of the NMEA-GX-GST message on port USB
  MsgoutNmeaIdGstUsb, 0x209100d6, u8,
  /// Output rate of the NMEA-GX-GSV message on port I2C
  MsgoutNmeaIdGsvI2c, 0x209100c4, u8,
  /// Output rate of the NMEA-GX-GSV message on port SPI
  MsgoutNmeaIdGsvSpi, 0x209100c8, u8,
  /// Output rate of the NMEA-GX-GSV message on port UART1
  MsgoutNmeaIdGsvUart1, 0x209100c5, u8,
  /// Output rate of the NMEA-GX-GSV message on port UART2
  MsgoutNmeaIdGsvUart2, 0x209100c6, u8,
  /// Output rate of the NMEA-GX-GSV message on port USB
  MsgoutNmeaIdGsvUsb, 0x209100c7, u8,
  /// Output rate of the NMEA-GX-RMC message on port I2C
  MsgoutNmeaIdRmcI2c, 0x209100ab, u8,
  /// Output rate of the NMEA-GX-RMC message on port SPI
  MsgoutNmeaIdRmcSpi, 0x209100af, u8,
  /// Output rate of the NMEA-GX-RMC message on port UART1
  MsgoutNmeaIdRmcUart1, 0x209100ac, u8,
  /// Output rate of the NMEA-GX-RMC message on port UART2
  MsgoutNmeaIdRmcUart2, 0x209100ad, u8,
  /// Output rate of the NMEA-GX-RMC message on port USB
  MsgoutNmeaIdRmcUsb, 0x209100ae, u8,
  /// Output rate of the NMEA-GX-VLW message on port I2C
  MsgoutNmeaIdVlwI2c, 0x209100e7, u8,
  /// Output rate of the NMEA-GX-VLW message on port SPI
  MsgoutNmeaIdVlwSpi, 0x209100eb, u8,
  /// Output rate of the NMEA-GX-VLW message on port UART1
  MsgoutNmeaIdVlwUart1, 0x209100e8, u8,
  /// Output rate of the NMEA-GX-VLW message on port UART2
  MsgoutNmeaIdVlwUart2, 0x209100e9, u8,
  /// Output rate of the NMEA-GX-VLW message on port USB
  MsgoutNmeaIdVlwUsb, 0x209100ea, u8,
  /// Output rate of the NMEA-GX-VTG message on port I2C
  MsgoutNmeaIdVtgI2c, 0x209100b0, u8,
  /// Output rate of the NMEA-GX-VTG message on port SPI
  MsgoutNmeaIdVtgSpi, 0x209100b4, u8,
  /// Output rate of the NMEA-GX-VTG message on port UART1
  MsgoutNmeaIdVtgUart1, 0x209100b1, u8,
  /// Output rate of the NMEA-GX-VTG message on port UART2
  MsgoutNmeaIdVtgUart2, 0x209100b2, u8,
  /// Output rate of the NMEA-GX-VTG message on port USB
  MsgoutNmeaIdVtgUsb, 0x209100b3, u8,
  /// Output rate of the NMEA-GX-ZDA message on port I2C
  MsgoutNmeaIdZdaI2c, 0x209100d8, u8,
  /// Output rate of the NMEA-GX-ZDA message on port SPI
  MsgoutNmeaIdZdaSpi, 0x209100dc, u8,
  /// Output rate of the NMEA-GX-ZDA message on port UART1
  MsgoutNmeaIdZdaUart1, 0x209100d9, u8,
  /// Output rate of the NMEA-GX-ZDA message on port UART2
  MsgoutNmeaIdZdaUart2, 0x209100da, u8,
  /// Output rate of the NMEA-GX-ZDA message on port USB
  MsgoutNmeaIdZdaUsb, 0x209100db, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port I2C
  MsgoutPubxIdPolypI2c, 0x209100ec, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port SPI
  MsgoutPubxIdPolypSpi, 0x209100f0, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port UART1
  MsgoutPubxIdPolypUart1, 0x209100ed, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port UART2
  MsgoutPubxIdPolypUart2, 0x209100ee, u8,
  /// Output rate of the NMEA-GX-PUBX00 message on port USB
  MsgoutPubxIdPolypUsb, 0x209100ef, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port I2C
  MsgoutPubxIdPolysI2c, 0x209100f1, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port SPI
  MsgoutPubxIdPolysSpi, 0x209100f5, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port UART1
  MsgoutPubxIdPolysUart1, 0x209100f2, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port UART2
  MsgoutPubxIdPolysUart2, 0x209100f3, u8,
  /// Output rate of the NMEA-GX-PUBX03 message on port USB
  MsgoutPubxIdPolysUsb, 0x209100f4, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port I2C
  MsgoutPubxIdPolytI2c, 0x209100f6, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port SPI
  MsgoutPubxIdPolytSpi, 0x209100fa, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port UART1
  MsgoutPubxIdPolytUart1, 0x209100f7, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port UART2
  MsgoutPubxIdPolytUart2, 0x209100f8, u8,
  /// Output rate of the NMEA-GX-PUBX04 message on port USB
  MsgoutPubxIdPolytUsb, 0x209100f9, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port I2C
  MsgoutRtcm3xType1005I2c, 0x209102bd, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port SPI
  MsgoutRtcm3xType1005Spi, 0x209102c1, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port UART1
  MsgoutRtcm3xType1005Uart1, 0x209102be, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port UART2
  MsgoutRtcm3xType1005Uart2, 0x209102bf, u8,
  /// Output rate of the RTCM-3XTYPE1005 message on port USB
  MsgoutRtcm3xType1005Usb, 0x209102c0, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port I2C
  MsgoutRtcm3xType1074I2c, 0x2091035e, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port SPI
  MsgoutRtcm3xType1074Spi, 0x20910362, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port UART1
  MsgoutRtcm3xType1074Uart1, 0x2091035f, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port UART2
  MsgoutRtcm3xType1074Uart2, 0x20910360, u8,
  /// Output rate of the RTCM-3XTYPE1074 message on port USB
  MsgoutRtcm3xType1074Usb, 0x20910361, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port I2C
  MsgoutRtcm3xType1077I2c, 0x209102cc, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port SPI
  MsgoutRtcm3xType1077Spi, 0x209102d0, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port UART1
  MsgoutRtcm3xType1077Uart1, 0x209102cd, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port UART2
  MsgoutRtcm3xType1077Uart2, 0x209102ce, u8,
  /// Output rate of the RTCM-3XTYPE1077 message on port USB
  MsgoutRtcm3xType1077Usb, 0x209102cf, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port I2C
  MsgoutRtcm3xType1084I2c, 0x20910363, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port SPI
  MsgoutRtcm3xType1084Spi, 0x20910367, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port UART1
  MsgoutRtcm3xType1084Uart1, 0x20910364, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port UART2
  MsgoutRtcm3xType1084Uart2, 0x20910365, u8,
  /// Output rate of the RTCM-3XTYPE1084 message on port USB
  MsgoutRtcm3xType1084Usb, 0x20910366, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port I2C
  MsgoutRtcm3xType1087I2c, 0x209102d1, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port SPI
  MsgoutRtcm3xType1087Spi, 0x209102d5, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port UART1
  MsgoutRtcm3xType1087Uart1, 0x209102d2, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port UART2
  MsgoutRtcm3xType1087Uart2, 0x209102d3, u8,
  /// Output rate of the RTCM-3XTYPE1087 message on port USB
  MsgoutRtcm3xType1087Usb, 0x209102d4, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port I2C
  MsgoutRtcm3xType1094I2c, 0x20910368, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port SPI
  MsgoutRtcm3xType1094Spi, 0x2091036c, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port UART1
  MsgoutRtcm3xType1094Uart1, 0x20910369, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port UART2
  MsgoutRtcm3xType1094Uart2, 0x2091036a, u8,
  /// Output rate of the RTCM-3XTYPE1094 message on port USB
  MsgoutRtcm3xType1094Usb, 0x2091036b, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port I2C
  MsgoutRtcm3xType1097I2c, 0x20910318, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port SPI
  MsgoutRtcm3xType1097Spi, 0x2091031c, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port UART1
  MsgoutRtcm3xType1097Uart1, 0x20910319, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port UART2
  MsgoutRtcm3xType1097Uart2, 0x2091031a, u8,
  /// Output rate of the RTCM-3XTYPE1097 message on port USB
  MsgoutRtcm3xType1097Usb, 0x2091031b, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port I2C
  MsgoutRtcm3xType1124I2c, 0x2091036d, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port SPI
  MsgoutRtcm3xType1124Spi, 0x20910371, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port UART1
  MsgoutRtcm3xType1124Uart1, 0x2091036e, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port UART2
  MsgoutRtcm3xType1124Uart2, 0x2091036f, u8,
  /// Output rate of the RTCM-3XTYPE1124 message on port USB
  MsgoutRtcm3xType1124Usb, 0x20910370, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port I2C
  MsgoutRtcm3xType1127I2c, 0x209102d6, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port SPI
  MsgoutRtcm3xType1127Spi, 0x209102da, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port UART1
  MsgoutRtcm3xType1127Uart1, 0x209102d7, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port UART2
  MsgoutRtcm3xType1127Uart2, 0x209102d8, u8,
  /// Output rate of the RTCM-3XTYPE1127 message on port USB
  MsgoutRtcm3xType1127Usb, 0x209102d9, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port I2C
  MsgoutRtcm3xType1230I2c, 0x20910303, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port SPI
  MsgoutRtcm3xType1230Spi, 0x20910307, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port UART1
  MsgoutRtcm3xType1230Uart1, 0x20910304, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port UART2
  MsgoutRtcm3xType1230Uart2, 0x20910305, u8,
  /// Output rate of the RTCM-3XTYPE1230 message on port USB
  MsgoutRtcm3xType1230Usb, 0x20910306, u8,
  /// Output rate of the UBX-LOG-INFO message on port I2C
  MsgoutUbxLogInfoI2c, 0x20910259, u8,
  /// Output rate of the UBX-LOG-INFO message on port SPI
  MsgoutUbxLogInfoSpi, 0x2091025d, u8,
  /// Output rate of the UBX-LOG-INFO message on port UART1
  MsgoutUbxLogInfoUart1, 0x2091025a, u8,
  /// Output rate of the UBX-LOG-INFO message on port UART2
  MsgoutUbxLogInfoUart2, 0x2091025b, u8,
  /// Output rate of the UBX-LOG-INFO message on port USB
  MsgoutUbxLogInfoUsb, 0x2091025c, u8,
  /// Output rate of the UBX-MONCOMMS message on port I2C
  MsgoutUbxMonCommsI2c, 0x2091034f, u8,
  /// Output rate of the UBX-MONCOMMS message on port SPI
  MsgoutUbxMonCommsSpi, 0x20910353, u8,
  /// Output rate of the UBX-MONCOMMS message on port UART1
  MsgoutUbxMonCommsUart1, 0x20910350, u8,
  /// Output rate of the UBX-MONCOMMS message on port UART2
  MsgoutUbxMonCommsUart2, 0x20910351, u8,
  /// Output rate of the UBX-MONCOMMS message on port USB
  MsgoutUbxMonCommsUsb, 0x20910352, u8,
  /// Output rate of the UBX-MON-HW2 message on port I2C
  MsgoutUbxMonHw2I2c, 0x209101b9, u8,
  /// Output rate of the UBX-MON-HW2 message on port SPI
  MsgoutUbxMonHw2Spi, 0x209101bd, u8,
  /// Output rate of the UBX-MON-HW2 message on port UART1
  MsgoutUbxMonHw2Uart1, 0x209101ba, u8,
  /// Output rate of the UBX-MON-HW2 message on port UART2
  MsgoutUbxMonHw2Uart2, 0x209101bb, u8,
  /// Output rate of the UBX-MON-HW2 message on port USB
  MsgoutUbxMonHw2Usb, 0x209101bc, u8,
  /// Output rate of the UBX-MON-HW3 message on port I2C
  MsgoutUbxMonHw3I2c, 0x20910354, u8,
  /// Output rate of the UBX-MON-HW3 message on port SPI
  MsgoutUbxMonHw3Spi, 0x20910358, u8,
  /// Output rate of the UBX-MON-HW3 message on port UART1
  MsgoutUbxMonHw3Uart1, 0x20910355, u8,
  /// Output rate of the UBX-MON-HW3 message on port UART2
  MsgoutUbxMonHw3Uart2, 0x20910356, u8,
  /// Output rate of the UBX-MON-HW3 message on port USB
  MsgoutUbxMonHw3Usb, 0x20910357, u8,
  /// Output rate of the UBX-MON-HW message on port I2C
  MsgoutUbxMonHwI2c, 0x209101b4, u8,
  /// Output rate of the UBX-MON-HW message on port SPI
  MsgoutUbxMonHwSpi, 0x209101b8, u8,
  /// Output rate of the UBX-MON-HW message on port UART1
  MsgoutUbxMonHwUart1, 0x209101b5, u8,
  /// Output rate of the UBX-MON-HW message on port UART2
  MsgoutUbxMonHwUart2, 0x209101b6, u8,
  /// Output rate of the UBX-MON-HW message on port USB
  MsgoutUbxMonHwUsb, 0x209101b7, u8,
  /// Output rate of the UBX-MON-IO message on port I2C
  MsgoutUbxMonIoI2c, 0x209101a5, u8,
  /// Output rate of the UBX-MON-IO message on port SPI
  MsgoutUbxMonIoSpi, 0x209101a9, u8,
  /// Output rate of the UBX-MON-IO message on port UART1
  MsgoutUbxMonIoUart1, 0x209101a6, u8,
  /// Output rate of the UBX-MON-IO message on port UART2
  MsgoutUbxMonIoUart2, 0x209101a7, u8,
  /// Output rate of the UBX-MON-IO message on port USB
  MsgoutUbxMonIoUsb, 0x209101a8, u8,
  /// Output rate of the UBX-MON-MSGPP message on port I2C
  MsgoutUbxMonMsgPpI2c, 0x20910196, u8,
  /// Output rate of the UBX-MON-MSGPP message on port SPI
  MsgoutUbxMonMsgPpSpi, 0x2091019a, u8,
  /// Output rate of the UBX-MON-MSGPP message on port UART1
  MsgoutUbxMonMsgPpUart1, 0x20910197, u8,
  /// Output rate of the UBX-MON-MSGPP message on port UART2
  MsgoutUbxMonMsgPpUart2, 0x20910198, u8,
  /// Output rate of the UBX-MON-MSGPP message on port USB
  MsgoutUbxMonMsgPpUsb, 0x20910199, u8,
  /// Output rate of the UBX-MON-RF message on port I2C
  MsgoutUbxMonRfI2c, 0x20910359, u8,
  /// Output rate of the UBX-MON-RF message on port SPI
  MsgoutUbxMonRfSpi, 0x2091035d, u8,
  /// Output rate of the UBX-MON-RF message on port UART1
  MsgoutUbxMonRfUart1, 0x2091035a, u8,
  /// Output rate of the UBX-MON-RF message on port UART2
  MsgoutUbxMonRfUart2, 0x2091035b, u8,
  /// Output rate of the UBX-MON-RF message on port USB
  MsgoutUbxMonRfUsb, 0x2091035c, u8,
  /// Output rate of the UBX-MON-RXBUF message on port I2C
  MsgoutUbxMonRxbufI2c, 0x209101a0, u8,
  /// Output rate of the UBX-MON-RXBUF message on port SPI
  MsgoutUbxMonRxbufSpi, 0x209101a4, u8,
  /// Output rate of the UBX-MON-RXBUF message on port UART1
  MsgoutUbxMonRxbufUart1, 0x209101a1, u8,
  /// Output rate of the UBX-MON-RXBUF message on port UART2
  MsgoutUbxMonRxbufUart2, 0x209101a2, u8,
  /// Output rate of the UBX-MON-RXBUF message on port USB
  MsgoutUbxMonRxbufUsb, 0x209101a3, u8,
  /// Output rate of the UBX-MON-RXR message on port I2C
  MsgoutUbxMonRxrI2c, 0x20910187, u8,
  /// Output rate of the UBX-MON-RXR message on port SPI
  MsgoutUbxMonRxrSpi, 0x2091018b, u8,
  /// Output rate of the UBX-MON-RXR message on port UART1
  MsgoutUbxMonRxrUart1, 0x20910188, u8,
  /// Output rate of the UBX-MON-RXR message on port UART2
  MsgoutUbxMonRxrUart2, 0x20910189, u8,
  /// Output rate of the UBX-MON-RXR message on port USB
  MsgoutUbxMonRxrUsb, 0x2091018a, u8,
  /// Output rate of the UBX-MON-TXBUF message on port I2C
  MsgoutUbxMonTxbufI2c, 0x2091019b, u8,
  /// Output rate of the UBX-MON-TXBUF message on port SPI
  MsgoutUbxMonTxbufSpi, 0x2091019f, u8,
  /// Output rate of the UBX-MON-TXBUF message on port UART1
  MsgoutUbxMonTxbufUart1, 0x2091019c, u8,
  /// Output rate of the UBX-MON-TXBUF message on port UART2
  MsgoutUbxMonTxbufUart2, 0x2091019d, u8,
  /// Output rate of the UBX-MON-TXBUF message on port USB
  MsgoutUbxMonTxbufUsb, 0x2091019e, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port I2C
  MsgoutUbxNavClockI2c, 0x20910065, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port SPI
  MsgoutUbxNavClockSpi, 0x20910069, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port UART1
  MsgoutUbxNavClockUart1, 0x20910066, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port UART2
  MsgoutUbxNavClockUart2, 0x20910067, u8,
  /// Output rate of the UBX-NAV-CLOCK message on port USB
  MsgoutUbxNavClockUsb, 0x20910068, u8,
  /// Output rate of the UBX-NAV-DOP message on port I2C
  MsgoutUbxNavDopI2c, 0x20910038, u8,
  /// Output rate of the UBX-NAV-DOP message on port SPI
  MsgoutUbxNavDopSpi, 0x2091003c, u8,
  /// Output rate of the UBX-NAV-DOP message on port UART1
  MsgoutUbxNavDopUart1, 0x20910039, u8,
  /// Output rate of the UBX-NAV-DOP message on port UART2
  MsgoutUbxNavDopUart2, 0x2091003a, u8,
  /// Output rate of the UBX-NAV-DOP message on port USB
  MsgoutUbxNavDopUsb, 0x2091003b, u8,
  /// Output rate of the UBX-NAV-EOE message on port I2C
  MsgoutUbxNavEoeI2c, 0x2091015f, u8,
  /// Output rate of the UBX-NAV-EOE message on port SPI
  MsgoutUbxNavEoeSpi, 0x20910163, u8,
  /// Output rate of the UBX-NAV-EOE message on port UART1
  MsgoutUbxNavEoeUart1, 0x20910160, u8,
  /// Output rate of the UBX-NAV-EOE message on port UART2
  MsgoutUbxNavEoeUart2, 0x20910161, u8,
  /// Output rate of the UBX-NAV-EOE message on port USB
  MsgoutUbxNavEoeUsb, 0x20910162, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port I2C
  MsgoutUbxNavGeofenceI2c, 0x209100a1, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port SPI
  MsgoutUbxNavGeofenceSpi, 0x209100a5, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port UART1
  MsgoutUbxNavGeofenceUart1, 0x209100a2, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port UART2
  MsgoutUbxNavGeofenceUart2, 0x209100a3, u8,
  /// Output rate of the UBX-NAVGEOFENCE message on port USB
  MsgoutUbxNavGeofenceUsb, 0x209100a4, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port I2C
  MsgoutUbxNavHpPosEcefI2c, 0x2091002e, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port SPI
  MsgoutUbxNavHpPosEcefSpi, 0x20910032, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port UART1
  MsgoutUbxNavHpPosEcefUart1, 0x2091002f, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port UART2
  MsgoutUbxNavHpPosEcefUart2, 0x20910030, u8,
  /// Output rate of the UBX-NAVHPPOSECEF message on port USB
  MsgoutUbxNavHpPosEcefUsb, 0x20910031, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port I2C
  MsgoutUbxNavHpPosllhI2c, 0x20910033, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port SPI
  MsgoutUbxNavHpPosllhSpi, 0x20910037, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port UART1
  MsgoutUbxNavHpPosllhUart1, 0x20910034, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port UART2
  MsgoutUbxNavHpPosllhUart2, 0x20910035, u8,
  /// Output rate of the UBX-NAVHPPOSLLH message on port USB
  MsgoutUbxNavHpPosllhUsb, 0x20910036, u8,
  /// Output rate of the UBX-NAV-ODO message on port I2C
  MsgoutUbxNavOdoI2c, 0x2091007e, u8,
  /// Output rate of the UBX-NAV-ODO message on port SPI
  MsgoutUbxNavOdoSpi, 0x20910082, u8,
  /// Output rate of the UBX-NAV-ODO message on port UART1
  MsgoutUbxNavOdoUart1, 0x2091007f, u8,
  /// Output rate of the UBX-NAV-ODO message on port UART2
  MsgoutUbxNavOdoUart2, 0x20910080, u8,
  /// Output rate of the UBX-NAV-ODO message on port USB
  MsgoutUbxNavOdoUsb, 0x20910081, u8,
  /// Output rate of the UBX-NAV-ORB message on port I2C
  MsgoutUbxNavOrbI2c, 0x20910010, u8,
  /// Output rate of the UBX-NAV-ORB message on port SPI
  MsgoutUbxNavOrbSpi, 0x20910014, u8,
  /// Output rate of the UBX-NAV-ORB message on port UART1
  MsgoutUbxNavOrbUart1, 0x20910011, u8,
  /// Output rate of the UBX-NAV-ORB message on port UART2
  MsgoutUbxNavOrbUart2, 0x20910012, u8,
  /// Output rate of the UBX-NAV-ORB message on port USB
  MsgoutUbxNavOrbUsb, 0x20910013, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port I2C
  MsgoutUbxNavPosEcefI2c, 0x20910024, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port SPI
  MsgoutUbxNavPosEcefSpi, 0x20910028, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port UART1
  MsgoutUbxNavPosEcefUart1, 0x20910025, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port UART2
  MsgoutUbxNavPosEcefUart2, 0x20910026, u8,
  /// Output rate of the UBX-NAV-POSECEF message on port USB
  MsgoutUbxNavPosEcefUsb, 0x20910027, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port I2C
  MsgoutUbxNavPosLlhI2c, 0x20910029, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port SPI
  MsgoutUbxNavPosLlhSpi, 0x2091002d, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port UART1
  MsgoutUbxNavPosLlhUart1, 0x2091002a, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port UART2
  MsgoutUbxNavPosLlhUart2, 0x2091002b, u8,
  /// Output rate of the UBX-NAV-POSLLH message on port USB
  MsgoutUbxNavPosLlhUsb, 0x2091002c, u8,
  /// Output rate of the UBX-NAV-PVT message on port I2C
  MsgoutUbxNavPvtI2c, 0x20910006, u8,
  /// Output rate of the UBX-NAV-PVT message on port SPI
  MsgoutUbxNavPvtSpi, 0x2091000a, u8,
  /// Output rate of the UBX-NAV-PVT message on port UART1
  MsgoutUbxNavPvtUart1, 0x20910007, u8,
  /// Output rate of the UBX-NAV-PVT message on port UART2
  MsgoutUbxNavPvtUart2, 0x20910008, u8,
  /// Output rate of the UBX-NAV-PVT message on port USB
  MsgoutUbxNavPvtUsb, 0x20910009, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port I2C
  MsgoutUbxNavRelposnedI2c, 0x2091008d, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port SPI
  MsgoutUbxNavRelposnedSpi, 0x20910091, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port UART1
  MsgoutUbxNavRelposnedUart1, 0x2091008e, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port UART2
  MsgoutUbxNavRelposnedUart2, 0x2091008f, u8,
  /// Output rate of the UBX-NAVRELPOSNED message on port USB
  MsgoutUbxNavRelposnedUsb, 0x20910090, u8,
  /// Output rate of the UBX-NAV-SAT message on port I2C
  MsgoutUbxNavSatI2c, 0x20910015, u8,
  /// Output rate of the UBX-NAV-SAT message on port SPI
  MsgoutUbxNavSatSpi, 0x20910019, u8,
  /// Output rate of the UBX-NAV-SAT message on port UART1
  MsgoutUbxNavSatUart1, 0x20910016, u8,
  /// Output rate of the UBX-NAV-SAT message on port UART2
  MsgoutUbxNavSatUart2, 0x20910017, u8,
  /// Output rate of the UBX-NAV-SAT message on port USB
  MsgoutUbxNavSatUsb, 0x20910018, u8,
  /// Output rate of the UBX-NAV-SIG message on port I2C
  MsgoutUbxNavSigI2c, 0x20910345, u8,
  /// Output rate of the UBX-NAV-SIG message on port SPI
  MsgoutUbxNavSigSpi, 0x20910349, u8,
  /// Output rate of the UBX-NAV-SIG message on port UART1
  MsgoutUbxNavSigUart1, 0x20910346, u8,
  /// Output rate of the UBX-NAV-SIG message on port UART2
  MsgoutUbxNavSigUart2, 0x20910347, u8,
  /// Output rate of the UBX-NAV-SIG message on port USB
  MsgoutUbxNavSigUsb, 0x20910348, u8,
  /// Output rate of the UBX-NAV-STATUS message on port I2C
  MsgoutUbxNavStatusI2c, 0x2091001a, u8,
  /// Output rate of the UBX-NAV-STATUS message on port SPI
  MsgoutUbxNavStatusSpi, 0x2091001e, u8,
  /// Output rate of the UBX-NAV-STATUS message on port UART1
  MsgoutUbxNavStatusUart1, 0x2091001b, u8,
  /// Output rate of the UBX-NAV-STATUS message on port UART2
  MsgoutUbxNavStatusUart2, 0x2091001c, u8,
  /// Output rate of the UBX-NAV-STATUS message on port USB
  MsgoutUbxNavStatusUsb, 0x2091001d, u8,
  /// Output rate of the UBX-NAV-SVIN message on port I2C
  MsgoutUbxNavSvinI2c, 0x20910088, u8,
  /// Output rate of the UBX-NAV-SVIN message on port SPI
  MsgoutUbxNavSvinSpi, 0x2091008c, u8,
  /// Output rate of the UBX-NAV-SVIN message on port UART1
  MsgoutUbxNavSvinUart1, 0x20910089, u8,
  /// Output rate of the UBX-NAV-SVIN message on port UART2
  MsgoutUbxNavSvinUart2, 0x2091008a, u8,
  /// Output rate of the UBX-NAV-SVIN message on port USB
  MsgoutUbxNavSvinUsb, 0x2091008b, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port I2C
  MsgoutUbxNavTimeBdsI2c, 0x20910051, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port SPI
  MsgoutUbxNavTimeBdsSpi, 0x20910055, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port UART1
  MsgoutUbxNavTimeBdsUart1, 0x20910052, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port UART2
  MsgoutUbxNavTimeBdsUart2, 0x20910053, u8,
  /// Output rate of the UBX-NAV-TIMEBDS message on port USB
  MsgoutUbxNavTimeBdsUsb, 0x20910054, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port I2C
  MsgoutUbxNavTimeGalI2c, 0x20910056, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port SPI
  MsgoutUbxNavTimeGalSpi, 0x2091005a, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port UART1
  MsgoutUbxNavTimeGalUart1, 0x20910057, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port UART2
  MsgoutUbxNavTimeGalUart2, 0x20910058, u8,
  /// Output rate of the UBX-NAVTIMEGAL message on port USB
  MsgoutUbxNavTimeGalUsb, 0x20910059, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port I2C
  MsgoutUbxNavTimeGloI2c, 0x2091004c, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port SPI
  MsgoutUbxNavTimeGloSpi, 0x20910050, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port UART1
  MsgoutUbxNavTimeGloUart1, 0x2091004d, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port UART2
  MsgoutUbxNavTimeGloUart2, 0x2091004e, u8,
  /// Output rate of the UBX-NAVTIMEGLO message on port USB
  MsgoutUbxNavTimeGloUsb, 0x2091004f, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port I2C
  MsgoutUbxNavTimeGpsI2c, 0x20910047, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port SPI
  MsgoutUbxNavTimeGpsSpi, 0x2091004b, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port UART1
  MsgoutUbxNavTimeGpsUart1, 0x20910048, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port UART2
  MsgoutUbxNavTimeGpsUart2, 0x20910049, u8,
  /// Output rate of the UBX-NAV-TIMEGPS message on port USB
  MsgoutUbxNavTimeGpsUsb, 0x2091004a, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port I2C
  MsgoutUbxNavTimeLsI2c, 0x20910060, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port SPI
  MsgoutUbxNavTimeLsSpi, 0x20910064, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port UART1
  MsgoutUbxNavTimeLsUart1, 0x20910061, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port UART2
  MsgoutUbxNavTimeLsUart2, 0x20910062, u8,
  /// Output rate of the UBX-NAV-TIMELS message on port USB
  MsgoutUbxNavTimeLsUsb, 0x20910063, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port I2C
  MsgoutUbxNavTimeUtcI2c, 0x2091005b, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port SPI
  MsgoutUbxNavTimeUtcSpi, 0x2091005f, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port UART1
  MsgoutUbxNavTimeUtcUart1, 0x2091005c, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port UART2
  MsgoutUbxNavTimeUtcUart2, 0x2091005d, u8,
  /// Output rate of the UBX-NAVTIMEUTC message on port USB
  MsgoutUbxNavTimeUtcUsb, 0x2091005e, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port I2C
  MsgoutUbxNavVelEcefI2c, 0x2091003d, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port SPI
  MsgoutUbxNavVelEcefSpi, 0x20910041, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port UART1
  MsgoutUbxNavVelEcefUart1, 0x2091003e, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port UART2
  MsgoutUbxNavVelEcefUart2, 0x2091003f, u8,
  /// Output rate of the UBX-NAV-VELECEF message on port USB
  MsgoutUbxNavVelEcefUsb, 0x20910040, u8,
  /// Output rate of the UBX-NAV-VELNED message on port I2C
  MsgoutUbxNavVelNedI2c, 0x20910042, u8,
  /// Output rate of the UBX-NAV-VELNED message on port SPI
  MsgoutUbxNavVelNedSpi, 0x20910046, u8,
  /// Output rate of the UBX-NAV-VELNED message on port UART1
  MsgoutUbxNavVelNedUart1, 0x20910043, u8,
  /// Output rate of the UBX-NAV-VELNED message on port UART2
  MsgoutUbxNavVelNedUart2, 0x20910044, u8,
  /// Output rate of the UBX-NAV-VELNED message on port USB
  MsgoutUbxNavVelNedUsb, 0x20910045, u8,
  /// Output rate of the UBX-RXM-MEASX message on port I2C
  MsgoutUbxRxmMeasxI2c, 0x20910204, u8,
  /// Output rate of the UBX-RXM-MEASX message on port SPI
  MsgoutUbxRxmMeasxSpi, 0x20910208, u8,
  /// Output rate of the UBX-RXM-MEASX message on port UART1
  MsgoutUbxRxmMeasxUart1, 0x20910205, u8,
  /// Output rate of the UBX-RXM-MEASX message on port UART2
  MsgoutUbxRxmMeasxUart2, 0x20910206, u8,
  /// Output rate of the UBX-RXM-MEASX message on port USB
  MsgoutUbxRxmMeasxUsb, 0x20910207, u8,
  /// Output rate of the UBX-RXM-RAWX message on port I2C
  MsgoutUbxRxmRawxI2c, 0x209102a4, u8,
  /// Output rate of the UBX-RXM-RAWX message on port SPI
  MsgoutUbxRxmRawxSpi, 0x209102a8, u8,
  /// Output rate of the UBX-RXM-RAWX message on port UART1
  MsgoutUbxRxmRawxUart1, 0x209102a5, u8,
  /// Output rate of the UBX-RXM-RAWX message on port UART2
  MsgoutUbxRxmRawxUart2, 0x209102a6, u8,
  /// Output rate of the UBX-RXM-RAWX message on port USB
  MsgoutUbxRxmRawxUsb, 0x209102a7, u8,
  /// Output rate of the UBX-RXM-RLM message on port I2C
  MsgoutUbxRxmRlmI2c, 0x2091025e, u8,
  /// Output rate of the UBX-RXM-RLM message on port SPI
  MsgoutUbxRxmRlmSpi, 0x20910262, u8,
  /// Output rate of the UBX-RXM-RLM message on port UART1
  MsgoutUbxRxmRlmUart1, 0x2091025f, u8,
  /// Output rate of the UBX-RXM-RLM message on port UART2
  MsgoutUbxRxmRlmUart2, 0x20910260, u8,
  /// Output rate of the UBX-RXM-RLM message on port USB
  MsgoutUbxRxmRlmUsb, 0x20910261, u8,
  /// Output rate of the UBX-RXM-RTCM message on port I2C
  MsgoutUbxRxmRtcmI2c, 0x20910268, u8,
  /// Output rate of the UBX-RXM-RTCM message on port SPI
  MsgoutUbxRxmRtcmSpi, 0x2091026c, u8,
  /// Output rate of the UBX-RXM-RTCM message on port UART1
  MsgoutUbxRxmRtcmUart1, 0x20910269, u8,
  /// Output rate of the UBX-RXM-RTCM message on port UART2
  MsgoutUbxRxmRtcmUart2, 0x2091026a, u8,
  /// Output rate of the UBX-RXM-RTCM message on port USB
  MsgoutUbxRxmRtcmUsb, 0x2091026b, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port I2C
  MsgoutUbxRxmSfrbxI2c, 0x20910231, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port SPI
  MsgoutUbxRxmSfrbxSpi, 0x20910235, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port UART1
  MsgoutUbxRxmSfrbxUart1, 0x20910232, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port UART2
  MsgoutUbxRxmSfrbxUart2, 0x20910233, u8,
  /// Output rate of the UBX-RXM-SFRBX message on port USB
  MsgoutUbxRxmSfrbxUsb, 0x20910234, u8,
  /// Output rate of the UBX-TIM-TM2 message on port I2C
  MsgoutUbxTimTm2I2c, 0x20910178, u8,
  /// Output rate of the UBX-TIM-TM2 message on port SPI
  MsgoutUbxTimTm2Spi, 0x2091017c, u8,
  /// Output rate of the UBX-TIM-TM2 message on port UART1
  MsgoutUbxTimTm2Uart1, 0x20910179, u8,
  /// Output rate of the UBX-TIM-TM2 message on port UART2
  MsgoutUbxTimTm2Uart2, 0x2091017a, u8,
  /// Output rate of the UBX-TIM-TM2 message on port USB
  MsgoutUbxTimTm2Usb, 0x2091017b, u8,
  /// Output rate of the UBX-TIM-TP message on port I2C
  MsgoutUbxTimTpI2c, 0x2091017d, u8,
  /// Output rate of the UBX-TIM-TP message on port SPI
  MsgoutUbxTimTpSpi, 0x20910181, u8,
  /// Output rate of the UBX-TIM-TP message on port UART1
  MsgoutUbxTimTpUart1, 0x2091017e, u8,
  /// Output rate of the UBX-TIM-TP message on port UART2
  MsgoutUbxTimTpUart2, 0x2091017f, u8,
  /// Output rate of the UBX-TIM-TP message on port USB
  MsgoutUbxTimTpUsb, 0x20910180, u8,
  /// Output rate of the UBX-TIM-VRFY message on port I2C
  MsgoutUbxTimVrfyI2c, 0x20910092, u8,
  /// Output rate of the UBX-TIM-VRFY message on port SPI
  MsgoutUbxTimVrfySpi, 0x20910096, u8,
  /// Output rate of the UBX-TIM-VRFY message on port UART1
  MsgoutUbxTimVrfyUart1, 0x20910093, u8,
  /// Output rate of the UBX-TIM-VRFY message on port UART2
  MsgoutUbxTimVrfyUart2, 0x20910094, u8,
  /// Output rate of the UBX-TIM-VRFY message on port USB
  MsgoutUbxTimVrfyUsb, 0x20910095, u8,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpPulse {
    /// Time pulse period
    Period = 0,
    /// Time pulse frequency
    Freq = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpPulseLength {
    /// Time pulse ratio
    Ratio = 0,
    /// Time pulse length
    Length = 1,
}
