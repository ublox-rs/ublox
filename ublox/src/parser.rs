#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    constants::{
        NMEA_END_CHARS_LEN, NMEA_END_CHAR_1, NMEA_END_CHAR_2, NMEA_MAX_SENTENCE_LENGTH,
        NMEA_MIN_BUFFER_SIZE, NMEA_SYNC_CHAR, RTCM_HEADER_SIZE, RTCM_LENGTH_MASK, RTCM_SYNC_CHAR,
        UBX_CHECKSUM_LEN, UBX_CLASS_OFFSET, UBX_HEADER_LEN, UBX_LENGTH_OFFSET, UBX_MSG_ID_OFFSET,
        UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2, UBX_SYNC_SIZE,
    },
    error::ParserError,
    UbxPacket, UbxProtocol,
};

use core::marker::PhantomData;

// Pick the oldest enabled protocol as the default
#[cfg(feature = "ubx_proto14")]
/// The default protocol for types that are generic over protocols, the type of the [DefaultProtocol] depends on which protocol feature(s) are enabled
pub type DefaultProtocol = crate::proto14::Proto14;

#[cfg(all(not(feature = "ubx_proto14"), feature = "ubx_proto23"))]
/// The default protocol for types that are generic over protocols, the type of the [DefaultProtocol] depends on which protocol feature(s) are enabled
pub type DefaultProtocol = crate::proto23::Proto23;

#[cfg(all(
    not(feature = "ubx_proto14"),
    not(feature = "ubx_proto23"),
    feature = "ubx_proto27"
))]
/// The default protocol for types that are generic over protocols, the type of the [DefaultProtocol] depends on which protocol feature(s) are enabled
pub type DefaultProtocol = crate::proto27::Proto27;

#[cfg(all(
    not(feature = "ubx_proto14"),
    not(feature = "ubx_proto23"),
    not(feature = "ubx_proto27"),
    feature = "ubx_proto31"
))]
/// The default protocol for types that are generic over protocols, the type of the [DefaultProtocol] depends on which protocol feature(s) are enabled
pub type DefaultProtocol = crate::proto31::Proto31;

mod buffer;
use buffer::DualBuffer;
pub use buffer::{FixedBuffer, FixedLinearBuffer, UnderlyingBuffer};

mod checksum;

/// A compile-time builder for constructing UBX protocol parsers with different buffer types and protocols.
///
/// Unlike typical builders, `ParserBuilder` performs all configuration at compile time through
/// the type system rather than storing configuration in fields.
///
/// # Examples
///
/// ## Basic parser with default settings
///
/// ```rust
/// # use ublox::ParserBuilder;
///
/// // Creates a parser with Vec<u8> buffer and default protocol
/// # #[cfg(feature = "alloc")]
/// let mut parser = ParserBuilder::new().with_vec_buffer();
/// ```
///
/// ## Parser with fixed-size buffer (no_std compatible)
///
/// ```rust
/// # use ublox::ParserBuilder;
///
/// // Creates a parser with 1024-byte fixed buffer
/// let mut parser = ParserBuilder::new().with_fixed_buffer::<1024>();
/// ```
///
/// ## Parser with specific protocol version
///
/// ```rust
/// # #[cfg(all(feature = "alloc", feature = "ubx_proto23"))]
/// # {
/// # use ublox::{ParserBuilder, proto23::Proto23};
///
/// // Specify protocol version and buffer type
/// let mut parser = ParserBuilder::new()
///     .with_protocol::<Proto23>()
///     .with_vec_buffer();
/// # }
/// ```
///
/// ## Parser with custom buffer implementation
///
/// ```rust
/// # use ublox::{ParserBuilder, FixedLinearBuffer};
///
/// let mut my_array = [0u8; 512];
/// let custom_buffer = FixedLinearBuffer::new(&mut my_array);
///
/// let mut parser = ParserBuilder::new()
///     .with_buffer(custom_buffer);
/// ```
pub struct ParserBuilder<P: UbxProtocol = DefaultProtocol> {
    _phantom: PhantomData<P>,
}

impl Default for ParserBuilder<DefaultProtocol> {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserBuilder<DefaultProtocol> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<P: UbxProtocol> ParserBuilder<P> {
    /// Specify a protocol version
    pub const fn with_protocol<NewP: UbxProtocol>(self) -> ParserBuilder<NewP> {
        ParserBuilder {
            _phantom: PhantomData,
        }
    }

    /// Build a parser with a `Vec<u8>` buffer
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn with_vec_buffer(self) -> Parser<Vec<u8>, P> {
        Parser::new(Vec::new())
    }

    /// Build a parser with a fixed-size buffer (for no_std or when you want bounded memory usage)
    pub const fn with_fixed_buffer<const N: usize>(self) -> Parser<FixedBuffer<N>, P> {
        Parser::with_fixed_buffer()
    }

    /// Build a parser with a custom buffer implementation
    pub const fn with_buffer<T: UnderlyingBuffer>(self, buffer: T) -> Parser<T, P> {
        Parser::new(buffer)
    }
}

/// Streaming parser for UBX protocol with buffer.
///
/// The easiest way to construct a [Parser] is with the [ParserBuilder].
///
/// The default constructor will build a parser containing a Vec, but you can pass your own underlying buffer by passing it
/// to Parser::new().
///
/// If you pass your own buffer, it should be able to store at _least_ 4 bytes. In practice,
/// you won't be able to do anything useful unless it's at least 36 bytes long (the size
/// of a NavPosLlh packet).
pub struct Parser<T, P: UbxProtocol = DefaultProtocol>
where
    T: UnderlyingBuffer,
{
    buf: T,
    _phantom: PhantomData<P>,
}

impl<const N: usize> Parser<FixedBuffer<N>, DefaultProtocol> {
    /// Creates a new parser with a fixed-size buffer and the default protocol.
    /// Use this for no_std environments where you want a compile-time known buffer size.
    pub const fn new_fixed() -> Self {
        Self::with_fixed_buffer()
    }
}

impl<const N: usize, P: UbxProtocol> Parser<FixedBuffer<N>, P> {
    /// Creates a new parser with an owned, fixed-size internal buffer of size N.
    pub const fn with_fixed_buffer() -> Self {
        Self::new(FixedBuffer::new())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Parser<Vec<u8>, DefaultProtocol> {
    /// Creates a new parser with a `Vec<u8>` buffer and the default protocol.
    /// This is the simplest way to create a parser for most use cases.
    pub fn default_proto() -> Self {
        Self {
            buf: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T: UnderlyingBuffer, P: UbxProtocol> Parser<T, P> {
    pub const fn new(underlying: T) -> Self {
        Self {
            buf: underlying,
            _phantom: PhantomData,
        }
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns the number of elements in the buffer
    pub fn buffer_len(&self) -> usize {
        self.buf.len()
    }

    /// Returns the total number of elements the buffer can hold
    pub fn buffer_capacity(&self) -> usize {
        self.buf.max_capacity()
    }

    /// Appends `new_data` to the internal buffer and returns and iterator over the buffer
    /// that will yield [UbxPackets](UbxPacket) on demand.
    pub fn consume_ubx<'a>(&'a mut self, new_data: &'a [u8]) -> UbxParserIter<'a, T, P> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == UBX_SYNC_CHAR_1 {
                buf.drain(i);
                break;
            }
        }

        UbxParserIter {
            buf,
            _phantom: PhantomData,
        }
    }

    /// Appends `new_data` to the internal buffer and returns and iterator over the buffer
    /// that will yield [UbxPackets or RtcmPackets](AnyPacketRef) on demand.
    pub fn consume_ubx_rtcm<'a>(&'a mut self, new_data: &'a [u8]) -> UbxRtcmParserIter<'a, T, P> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == UBX_SYNC_CHAR_1 || buf[i] == RTCM_SYNC_CHAR {
                buf.drain(i);
                break;
            }
        }

        UbxRtcmParserIter {
            buf,
            _phantom: PhantomData,
        }
    }

    /// Appends `new_data` to the internal buffer and returns and iterator over the buffer
    /// that will yield [UbxPackets, RtcmPackets, or NmeaPackets](AnyPacketRef) on demand.
    pub fn consume_ubx_rtcm_nmea<'a>(
        &'a mut self,
        new_data: &'a [u8],
    ) -> UbxRtcmNmeaParserIter<'a, T, P> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == UBX_SYNC_CHAR_1 || buf[i] == RTCM_SYNC_CHAR || buf[i] == NMEA_SYNC_CHAR {
                buf.drain(i);
                break;
            }
        }

        UbxRtcmNmeaParserIter {
            buf,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum NextSync {
    Ubx(usize),
    Rtcm(usize),
    Nmea(usize),
    None,
}

#[derive(Debug)]
pub enum AnyPacketRef<'a> {
    Ubx(UbxPacket<'a>),
    Rtcm(RtcmPacketRef<'a>),
    Nmea(NmeaPacketRef<'a>),
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxParserIter<'a, T: UnderlyingBuffer, P: UbxProtocol = DefaultProtocol> {
    buf: DualBuffer<'a, T>,
    _phantom: PhantomData<P>,
}

fn extract_packet_ubx<'b, T: UnderlyingBuffer, P: UbxProtocol>(
    buf: &'b mut DualBuffer<'_, T>,
    pack_len: u16,
) -> Option<Result<UbxPacket<'b>, ParserError>> {
    if !buf.can_drain_and_take(UBX_HEADER_LEN, usize::from(pack_len) + UBX_CHECKSUM_LEN) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            buf.drain(UBX_SYNC_SIZE);
            return Some(Err(ParserError::OutOfMemory {
                required_size: usize::from(pack_len) + UBX_CHECKSUM_LEN,
            }));
        }
        return None;
    }
    if let Err(checksum_error) = checksum::UbxChecksumCalc::validate_buffer(buf, pack_len) {
        buf.drain(UBX_SYNC_SIZE);
        return Some(Err(checksum_error));
    }

    let class_id = buf[UBX_CLASS_OFFSET];
    let msg_id = buf[UBX_MSG_ID_OFFSET];
    buf.drain(UBX_HEADER_LEN);
    let msg_data = match buf.take(usize::from(pack_len) + UBX_CHECKSUM_LEN) {
        Ok(x) => x,
        Err(e) => {
            return Some(Err(e));
        },
    };
    let specific_packet_result = P::match_packet(
        class_id,
        msg_id,
        &msg_data[..msg_data.len() - UBX_CHECKSUM_LEN],
    );
    Some(specific_packet_result.map(|p| p.into()))
}

impl<T: UnderlyingBuffer, P: UbxProtocol> UbxParserIter<'_, T, P> {
    fn find_sync(&self) -> Option<usize> {
        (0..self.buf.len()).find(|&i| self.buf[i] == UBX_SYNC_CHAR_1)
    }

    #[allow(
        clippy::should_implement_trait,
        reason = "This is a lending iterator, which is not in std"
    )]
    /// Parse and return the next [UbxPacket] in the buffer, or `None` if the buffer cannot yield
    /// another full [UbxPacket]
    pub fn next(&mut self) -> Option<Result<UbxPacket<'_>, ParserError>> {
        while self.buf.len() > 0 {
            let pos = match self.find_sync() {
                Some(x) => x,
                None => {
                    self.buf.clear();
                    return None;
                },
            };
            self.buf.drain(pos);

            if self.buf.len() < UBX_SYNC_SIZE {
                return None;
            }
            if self.buf[1] != UBX_SYNC_CHAR_2 {
                self.buf.drain(1);
                continue;
            }

            if self.buf.len() < UBX_HEADER_LEN {
                return None;
            }

            let pack_len =
                u16::from_le_bytes([self.buf[UBX_LENGTH_OFFSET], self.buf[UBX_LENGTH_OFFSET + 1]]);
            if pack_len > P::MAX_PAYLOAD_LEN {
                self.buf.drain(UBX_SYNC_SIZE);
                continue;
            }
            return extract_packet_ubx::<T, P>(&mut self.buf, pack_len);
        }
        None
    }
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxRtcmParserIter<'a, T: UnderlyingBuffer, P: UbxProtocol = DefaultProtocol> {
    buf: DualBuffer<'a, T>,
    _phantom: PhantomData<P>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RtcmPacketRef<'a> {
    pub data: &'a [u8],
}

fn extract_packet_rtcm<'a, 'b, T: UnderlyingBuffer>(
    buf: &'b mut DualBuffer<'a, T>,
    pack_len: u16,
) -> Option<Result<AnyPacketRef<'b>, ParserError>> {
    let pack_len = pack_len as usize; // `usize` is needed for indexing but constraining the input to `u16` is still important
    if !buf.can_drain_and_take(0, pack_len + RTCM_HEADER_SIZE) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            // Drain only the RTCM sync char to allow for finding another RTCM packet
            buf.drain(1);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len + RTCM_HEADER_SIZE,
            }));
        }
        return None;
    }

    let maybe_data = buf.take(pack_len + RTCM_HEADER_SIZE);
    match maybe_data {
        Ok(data) => Some(Ok(AnyPacketRef::Rtcm(RtcmPacketRef::<'b> { data }))),
        Err(e) => Some(Err(e)),
    }
}

impl<T: UnderlyingBuffer, P: UbxProtocol> UbxRtcmParserIter<'_, T, P> {
    fn find_sync(&self) -> NextSync {
        for i in 0..self.buf.len() {
            if self.buf[i] == UBX_SYNC_CHAR_1 {
                return NextSync::Ubx(i);
            }
            if self.buf[i] == RTCM_SYNC_CHAR {
                return NextSync::Rtcm(i);
            }
        }
        NextSync::None
    }

    #[allow(
        clippy::should_implement_trait,
        reason = "This is a lending iterator, which is not in std"
    )]
    /// Parse and return the next [UbxPacket or RtcmPacket](AnyPacketRef) in the buffer, or `None` if the buffer cannot yield
    /// another full packet
    pub fn next(&mut self) -> Option<Result<AnyPacketRef<'_>, ParserError>> {
        while self.buf.len() > 0 {
            match self.find_sync() {
                NextSync::Ubx(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < UBX_SYNC_SIZE {
                        return None;
                    }
                    if self.buf[1] != UBX_SYNC_CHAR_2 {
                        self.buf.drain(1);
                        continue;
                    }

                    if self.buf.len() < UBX_HEADER_LEN {
                        return None;
                    }

                    let pack_len = u16::from_le_bytes([
                        self.buf[UBX_LENGTH_OFFSET],
                        self.buf[UBX_LENGTH_OFFSET + 1],
                    ]);
                    if pack_len > P::MAX_PAYLOAD_LEN {
                        self.buf.drain(UBX_SYNC_SIZE);
                        continue;
                    }
                    let maybe_packet = extract_packet_ubx::<T, P>(&mut self.buf, pack_len);
                    match maybe_packet {
                        Some(Ok(packet)) => return Some(Ok(AnyPacketRef::Ubx(packet))),
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    }
                },
                NextSync::Rtcm(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < RTCM_HEADER_SIZE {
                        return None;
                    }
                    // next 2 bytes contain 6 bits reserved + 10 bits length, big endian
                    let pack_len =
                        u16::from_be_bytes([self.buf[1], self.buf[2]]) & RTCM_LENGTH_MASK;

                    return extract_packet_rtcm(&mut self.buf, pack_len);
                },
                NextSync::Nmea(_) | NextSync::None => {
                    self.buf.clear();
                    return None;
                },
            };
        }
        None
    }
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxRtcmNmeaParserIter<'a, T: UnderlyingBuffer, P: UbxProtocol = DefaultProtocol> {
    buf: DualBuffer<'a, T>,
    _phantom: PhantomData<P>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NmeaPacketRef<'a> {
    pub data: &'a [u8],
}

fn extract_packet_nmea<'a, 'b, T: UnderlyingBuffer>(
    buf: &'b mut DualBuffer<'a, T>,
    pack_len: u16,
) -> Option<Result<AnyPacketRef<'b>, ParserError>> {
    let pack_len = pack_len as usize; // `usize` is needed for indexing but constraining the input to `u16` is still important
    if !buf.can_drain_and_take(0, pack_len) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            // Drain only the NMEA sync char to allow for finding another NMEA sentence
            buf.drain(1);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len,
            }));
        }
        return None;
    }

    let maybe_data = buf.take(pack_len);
    match maybe_data {
        Ok(data) => Some(Ok(AnyPacketRef::Nmea(NmeaPacketRef::<'b> { data }))),
        Err(e) => Some(Err(e)),
    }
}

impl<T: UnderlyingBuffer, P: UbxProtocol> UbxRtcmNmeaParserIter<'_, T, P> {
    /// Find the next sync char in the buffer, starting at `min_idx`
    fn find_sync(&self, min_idx: usize) -> NextSync {
        for i in min_idx..self.buf.len() {
            match self.buf[i] {
                UBX_SYNC_CHAR_1 => return NextSync::Ubx(i),
                RTCM_SYNC_CHAR => return NextSync::Rtcm(i),
                NMEA_SYNC_CHAR => return NextSync::Nmea(i),
                _ => (),
            }
        }
        NextSync::None
    }

    #[allow(
        clippy::should_implement_trait,
        reason = "This is a lending iterator, which is not in std"
    )]
    /// Parse and return the next [UbxPacket, RtcmPacket, or NmeaPacket](AnyPacketRef) in the buffer, or `None` if the buffer cannot yield
    /// another full packet
    pub fn next(&mut self) -> Option<Result<AnyPacketRef<'_>, ParserError>> {
        while self.buf.len() > 0 {
            match self.find_sync(0) {
                NextSync::Ubx(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < UBX_SYNC_SIZE {
                        return None;
                    }
                    if self.buf[1] != UBX_SYNC_CHAR_2 {
                        self.buf.drain(1);
                        continue;
                    }

                    if self.buf.len() < UBX_HEADER_LEN {
                        return None;
                    }

                    let pack_len = u16::from_le_bytes([
                        self.buf[UBX_LENGTH_OFFSET],
                        self.buf[UBX_LENGTH_OFFSET + 1],
                    ]);
                    if pack_len > P::MAX_PAYLOAD_LEN {
                        self.buf.drain(UBX_SYNC_SIZE);
                        continue;
                    }
                    let maybe_packet = extract_packet_ubx::<T, P>(&mut self.buf, pack_len);
                    match maybe_packet {
                        Some(Ok(packet)) => return Some(Ok(AnyPacketRef::Ubx(packet))),
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    }
                },
                NextSync::Rtcm(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < RTCM_HEADER_SIZE {
                        return None;
                    }
                    // next 2 bytes contain 6 bits reserved + 10 bits length, big endian
                    let pack_len =
                        u16::from_be_bytes([self.buf[1], self.buf[2]]) & RTCM_LENGTH_MASK;

                    return extract_packet_rtcm(&mut self.buf, pack_len);
                },
                NextSync::Nmea(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < NMEA_MIN_BUFFER_SIZE {
                        return None;
                    }
                    // try to determine packet length by searching for NMEA end chars
                    let mut pack_len: Option<u16> = None;
                    for i in 0..self.buf.len() - 1 {
                        if self.buf[i] == NMEA_END_CHAR_1 && self.buf[i + 1] == NMEA_END_CHAR_2 {
                            // including sync and both end chars
                            pack_len = Some((i + NMEA_END_CHARS_LEN) as u16);
                            break;
                        }
                    }

                    // try to extract the packet if its length was found,
                    // otherwise check if NMEA string has to be discarded
                    return if let Some(len) = pack_len {
                        extract_packet_nmea(&mut self.buf, len)
                    } else {
                        if self.find_sync(1) != NextSync::None {
                            // found another packet before the end of the NMEA sentence,
                            // drain NMEA sync char
                            self.buf.drain(1);
                        } else if self.buf.len() > NMEA_MAX_SENTENCE_LENGTH {
                            // maximum NMEA length exceeded, clear buffer
                            self.buf.clear();
                        }
                        None
                    };
                },
                NextSync::None => {
                    self.buf.clear();
                    return None;
                },
            };
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ubx_packets::packets::cfg_nav5::*;
    use crate::ubx_packets::*;

    #[cfg(all(feature = "alloc", feature = "ubx_proto23", feature = "ubx_proto31"))]
    #[test]
    fn build_parser() {
        use crate::{proto23::Proto23, proto31::Proto31};
        // Default protocol with Vec buffer
        let parser = ParserBuilder::new().with_vec_buffer();
        assert_eq!(parser.buffer_capacity(), usize::MAX);
        assert_eq!(parser.buffer_len(), 0);

        // Different protocol
        const BUF_SZ_0: usize = 2048;
        let parser = ParserBuilder::new()
            .with_protocol::<Proto23>()
            .with_fixed_buffer::<BUF_SZ_0>();
        assert_eq!(parser.buffer_capacity(), BUF_SZ_0);
        assert_eq!(parser.buffer_len(), 0);

        // Custom buffer
        const BUF_SZ_1: usize = 512;
        let mut my_buffer = [0; BUF_SZ_1];
        let buffer = FixedLinearBuffer::new(&mut my_buffer);
        let parser = ParserBuilder::new()
            .with_protocol::<Proto31>()
            .with_buffer(buffer);

        assert_eq!(parser.buffer_capacity(), BUF_SZ_1);
        assert_eq!(parser.buffer_len(), 0);
    }

    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn parser_oom_processes_multiple_small_packets() {
        use crate::proto23::{PacketRef, Proto23};

        let packet = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        let mut bytes = vec![];
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);
        bytes.extend_from_slice(&packet);

        let mut buffer = [0; 10];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser: Parser<FixedLinearBuffer<'_>, Proto23> = Parser::new(buffer);

        let mut it = parser.consume_ubx(&bytes);
        for _ in 0..5 {
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto23(PacketRef::AckAck(_))))
            ));
        }
        assert!(it.next().is_none());
    }

    const BYTES_GARBAGE: [u8; 11] = [0xb5, 0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn parser_handle_garbage_first_byte_default() {
        let mut parser = Parser::default_proto();
        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(it.next().is_some());
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto14")]
    #[test]
    fn parser_handle_garbage_first_byte_proto14() {
        use crate::proto14::{PacketRef, Proto14};

        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser: Parser<FixedLinearBuffer<'_>, Proto14> = Parser::new(buffer);
        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto14(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto23")]
    #[test]
    fn parser_handle_garbage_first_byte_proto23() {
        use crate::proto23::{PacketRef, Proto23};
        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser: Parser<FixedLinearBuffer<'_>, Proto23> = Parser::new(buffer);

        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto23(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto27")]
    #[test]
    fn parser_handle_garbage_first_byte_proto27() {
        use crate::proto27::{PacketRef, Proto27};

        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser: Parser<FixedLinearBuffer<'_>, Proto27> = Parser::new(buffer);

        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto27(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto31")]
    #[test]
    fn parser_handle_garbage_first_byte_proto31() {
        use crate::proto31::{PacketRef, Proto31};

        let mut buffer = [0; 12];
        let mut parser: Parser<_, Proto31> = Parser::new(FixedLinearBuffer::new(&mut buffer));

        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto31(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    fn test_util_cfg_nav5_bytes() -> [u8; 44] {
        CfgNav5Builder {
            mask: CfgNav5Params::DYN,
            dyn_model: NavDynamicModel::AirborneWithLess1gAcceleration,
            fix_mode: NavFixMode::Only3D,
            fixed_alt: 100.17,
            fixed_alt_var: 0.0017,
            min_elev_degrees: 17,
            pdop: 1.7,
            tdop: 1.7,
            pacc: 17,
            tacc: 17,
            static_hold_thresh: 2.17,
            dgps_time_out: 17,
            cno_thresh_num_svs: 17,
            cno_thresh: 17,
            static_hold_max_dist: 0x1717,
            utc_standard: UtcStandardIdentifier::UtcChina,
            ..CfgNav5Builder::default()
        }
        .into_packet_bytes()
    }

    #[cfg(feature = "ubx_proto14")]
    #[test]
    fn parser_oom_clears_buffer_proto14() {
        use crate::proto14::{PacketRef, Proto14};

        let bytes = test_util_cfg_nav5_bytes();

        let mut buffer = [0; 12];
        let mut parser = Parser::<_, Proto14>::new(FixedLinearBuffer::new(&mut buffer));

        {
            let mut it = parser.consume_ubx(&bytes[0..8]);
            assert!(it.next().is_none());
        }

        {
            let mut it = parser.consume_ubx(&bytes[8..]);
            assert!(
                matches!(it.next(), Some(Err(ParserError::OutOfMemory { required_size })) if required_size == bytes.len() - 6)
            );
            assert!(it.next().is_none());
        }

        // Should now be empty, and we can parse a small packet
        let bytes = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto14(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto23")]
    #[test]
    fn parser_oom_clears_buffer_proto23() {
        use crate::proto23::{PacketRef, Proto23};
        let bytes = test_util_cfg_nav5_bytes();

        let mut buffer = [0; 12];
        let mut parser = Parser::<_, Proto23>::new(FixedLinearBuffer::new(&mut buffer));

        {
            let mut it = parser.consume_ubx(&bytes[0..8]);
            assert!(it.next().is_none());
        }

        {
            let mut it = parser.consume_ubx(&bytes[8..]);
            assert!(
                matches!(it.next(), Some(Err(ParserError::OutOfMemory { required_size })) if required_size == bytes.len() - 6)
            );
            assert!(it.next().is_none());
        }

        // Should now be empty, and we can parse a small packet
        let bytes = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto23(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto27")]
    #[test]
    fn parser_oom_clears_buffer_proto27() {
        use crate::proto27::{PacketRef, Proto27};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto27>()
            .with_fixed_buffer::<12>();

        {
            let mut it = parser.consume_ubx(&bytes[0..8]);
            assert!(it.next().is_none());
        }

        {
            let mut it = parser.consume_ubx(&bytes[8..]);
            assert!(
                matches!(it.next(), Some(Err(ParserError::OutOfMemory { required_size })) if required_size == bytes.len() - 6)
            );
            assert!(it.next().is_none());
        }

        // Should now be empty, and we can parse a small packet
        let bytes = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto27(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto31")]
    #[test]
    fn parser_oom_clears_buffer_proto31() {
        use crate::proto31::{PacketRef, Proto31};

        let bytes = test_util_cfg_nav5_bytes();
        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto31>()
            .with_fixed_buffer::<12>();

        {
            let mut it = parser.consume_ubx(&bytes[0..8]);
            assert!(it.next().is_none());
        }

        {
            let mut it = parser.consume_ubx(&bytes[8..]);
            assert!(
                matches!(it.next(), Some(Err(ParserError::OutOfMemory { required_size })) if required_size == bytes.len() - 6)
            );
            assert!(it.next().is_none());
        }

        // Should now be empty, and we can parse a small packet
        let bytes = [0xb5, 0x62, 0x5, 0x1, 0x2, 0x0, 0x4, 0x5, 0x11, 0x38];

        {
            let mut it = parser.consume_ubx(&bytes);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto31(PacketRef::AckAck(_))))
            ));
            assert!(it.next().is_none());
        }
    }

    #[cfg(feature = "ubx_proto14")]
    #[test]
    fn parser_accepts_packet_array_underlying_proto14() {
        use crate::proto14::{PacketRef, Proto14};

        let bytes = test_util_cfg_nav5_bytes();
        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto14>()
            .with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto14(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[cfg(feature = "ubx_proto23")]
    #[test]
    fn parser_accepts_packet_array_underlying_proto23() {
        use crate::proto23::{PacketRef, Proto23};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<FixedBuffer<1024>, Proto23>::with_fixed_buffer();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto23(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[cfg(feature = "ubx_proto27")]
    #[test]
    fn parser_accepts_packet_array_underlying_proto27() {
        use crate::proto27::{PacketRef, Proto27};
        let bytes = test_util_cfg_nav5_bytes();
        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto27>()
            .with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto27(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[cfg(feature = "ubx_proto31")]
    #[test]
    fn parser_accepts_packet_array_underlying_proto31() {
        use crate::proto31::{PacketRef, Proto31};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto31>()
            .with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto31(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto14"))]
    fn parser_accepts_packet_vec_underlying_proto14() {
        use crate::proto14::{PacketRef, Proto14};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<_, Proto14>::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto14(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto23"))]
    fn parser_accepts_packet_vec_underlying_proto23() {
        use crate::proto23::{PacketRef, Proto23};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<_, Proto23>::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto23(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto27"))]
    fn parser_accepts_packet_vec_underlying_proto27() {
        use crate::proto27::{PacketRef, Proto27};

        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<_, Proto27>::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto27(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto31"))]
    fn parser_accepts_packet_vec_underlying_proto31() {
        use crate::proto31::{PacketRef, Proto31};

        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<_, Proto31>::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto31(PacketRef::CfgNav5(_))))
        ));
        assert!(it.next().is_none());
    }

    #[cfg(feature = "std")]
    fn test_util_multiple_cfg_nav5_packets_bytes() -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(
            &CfgNav5Builder {
                pacc: 21,
                ..CfgNav5Builder::default()
            }
            .into_packet_bytes(),
        );
        data.extend_from_slice(
            &CfgNav5Builder {
                pacc: 18,
                ..CfgNav5Builder::default()
            }
            .into_packet_bytes(),
        );
        data
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto14"))]
    fn parser_accepts_multiple_packets_proto14() {
        use crate::proto14::{PacketRef, Proto14};
        let data = test_util_multiple_cfg_nav5_packets_bytes();
        let mut parser = Parser::<_, Proto14>::default();
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(UbxPacket::Proto14(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto14(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            },
            _ => {
                panic!()
            },
        }
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto23"))]
    fn parser_accepts_multiple_packets_proto23() {
        use crate::proto23::{PacketRef, Proto23};
        let data = test_util_multiple_cfg_nav5_packets_bytes();
        let mut parser = Parser::<_, Proto23>::default();
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(UbxPacket::Proto23(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto23(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            },
            _ => {
                panic!()
            },
        }
        assert!(it.next().is_none());
    }
    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto27"))]
    fn parser_accepts_multiple_packets_proto27() {
        use crate::proto27::{PacketRef, Proto27};

        let data = test_util_multiple_cfg_nav5_packets_bytes();
        let mut parser = Parser::<_, Proto27>::default();
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(UbxPacket::Proto27(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto27(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            },
            _ => {
                panic!()
            },
        }
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(all(feature = "std", feature = "ubx_proto31"))]
    fn parser_accepts_multiple_packets_proto31() {
        use crate::proto31::{PacketRef, Proto31};
        let data = test_util_multiple_cfg_nav5_packets_bytes();
        let mut parser = Parser::<_, Proto31>::default();
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(UbxPacket::Proto31(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto31(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 18);
            },
            _ => {
                panic!()
            },
        }
        assert!(it.next().is_none());
    }

    const ACK_ACK_BYTES: [u8; 10] = [
        UBX_SYNC_CHAR_1,
        UBX_SYNC_CHAR_2,
        0x05,
        0x01,
        0x02,
        0x00,
        0x04,
        0x05,
        0x11,
        0x38,
    ];

    #[cfg(feature = "ubx_proto14")]
    #[test]
    fn test_ack_ack_payload_len_proto14() {
        use crate::proto14::{PacketRef, Proto14};

        const ACK_ACK_PAYLOAD_LEN: usize = 2;

        let mut parser = crate::Parser::<FixedBuffer<1024>, Proto14>::with_fixed_buffer();
        let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
        match it.next() {
            Some(Ok(crate::UbxPacket::Proto14(PacketRef::AckAck(pack)))) => {
                assert_eq!(ACK_ACK_PAYLOAD_LEN, pack.payload_len());
            },
            _ => panic!(),
        }
    }

    #[cfg(feature = "ubx_proto23")]
    #[test]
    fn test_ack_ack_payload_len_proto23() {
        use crate::proto23::{PacketRef, Proto23};

        const ACK_ACK_PAYLOAD_LEN: usize = 2;

        let mut parser = crate::Parser::<FixedBuffer<1024>, Proto23>::with_fixed_buffer();
        let mut it = parser.consume_ubx(&ACK_ACK_BYTES);
        match it.next() {
            Some(Ok(crate::UbxPacket::Proto23(PacketRef::AckAck(pack)))) => {
                assert_eq!(ACK_ACK_PAYLOAD_LEN, pack.payload_len());
            },
            _ => panic!(),
        }
    }

    // NAV-PVT payload lengths differ across protocols: proto14=84, proto23/27/31=92
    const NAV_PVT_CLASS: u8 = 0x01;
    const NAV_PVT_ID: u8 = 0x07;
    #[cfg(feature = "ubx_proto14")]
    const NAV_PVT_PROTO14_LEN: usize = 84;
    #[cfg(feature = "ubx_proto23")]
    const NAV_PVT_PROTO23_LEN: usize = 92;

    fn build_nav_pvt_frame(frame: &mut [u8]) {
        frame[0] = UBX_SYNC_CHAR_1;
        frame[1] = UBX_SYNC_CHAR_2;
        frame[2] = NAV_PVT_CLASS;
        frame[3] = NAV_PVT_ID;
        let payload_len = (frame.len() - UBX_HEADER_LEN - UBX_CHECKSUM_LEN) as u16;
        let len_bytes = payload_len.to_le_bytes();
        frame[4] = len_bytes[0];
        frame[5] = len_bytes[1];
        let (ck_a, ck_b) = crate::ubx_packets::ubx_checksum(
            &frame[UBX_CLASS_OFFSET..(frame.len() - UBX_CHECKSUM_LEN)],
        );
        frame[frame.len() - 2] = ck_a;
        frame[frame.len() - 1] = ck_b;
    }

    #[cfg(feature = "ubx_proto14")]
    #[test]
    fn test_nav_pvt_payload_len_proto14() {
        use crate::proto14::{PacketRef, Proto14};

        const PACKET_LEN: usize = NAV_PVT_PROTO14_LEN + UBX_HEADER_LEN + UBX_CHECKSUM_LEN;
        let mut packet = [0; PACKET_LEN];

        build_nav_pvt_frame(&mut packet);
        let mut parser = crate::Parser::<FixedBuffer<1024>, Proto14>::with_fixed_buffer();
        let mut it = parser.consume_ubx(&packet);
        match it.next() {
            Some(Ok(crate::UbxPacket::Proto14(PacketRef::NavPvt(p)))) => {
                assert_eq!(NAV_PVT_PROTO14_LEN, p.payload_len())
            },
            _ => panic!(),
        }
    }

    #[cfg(feature = "ubx_proto23")]
    #[test]
    fn test_nav_pvt_payload_len_proto23() {
        use crate::proto23::{PacketRef, Proto23};

        const PACKET_LEN: usize = NAV_PVT_PROTO23_LEN + UBX_HEADER_LEN + UBX_CHECKSUM_LEN;
        let mut packet = [0; PACKET_LEN];

        build_nav_pvt_frame(&mut packet);
        let mut parser = crate::Parser::<FixedBuffer<1024>, Proto23>::with_fixed_buffer();
        let mut it = parser.consume_ubx(&packet);
        match it.next() {
            Some(Ok(crate::UbxPacket::Proto23(PacketRef::NavPvt(p)))) => {
                assert_eq!(NAV_PVT_PROTO23_LEN, p.payload_len())
            },
            _ => panic!(),
        }
    }
}
