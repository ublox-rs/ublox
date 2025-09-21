#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    error::ParserError,
    ubx_packets::{RTCM_SYNC_CHAR, SYNC_CHAR_1, SYNC_CHAR_2},
    UbxPacket, UbxProtocol,
};

use core::marker::PhantomData;

// Pick the oldest enabled protocol as the default
#[cfg(feature = "ubx_proto14")]
/// The default protocol for types that are generic over protocols, the type of the [DefaultProtocol] depends on which protocol feature(s) are enabled
pub type DefaultProtocol = crate::proto17::Proto17;

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
/// use ublox::ParserBuilder;
///
/// // Creates a parser with Vec<u8> buffer and default protocol
/// let mut parser = ParserBuilder::new().with_vec_buffer();
/// ```
///
/// ## Parser with fixed-size buffer (no_std compatible)
///
/// ```rust
/// use ublox::ParserBuilder;
///
/// // Creates a parser with 1024-byte fixed buffer
/// let mut parser = ParserBuilder::new().with_fixed_buffer::<1024>();
/// ```
///
/// ## Parser with specific protocol version
///
/// ```rust
/// use ublox::{ParserBuilder, proto23::Proto23};
///
/// // Specify protocol version and buffer type
/// let mut parser = ParserBuilder::new()
///     .with_protocol::<Proto23>()
///     .with_vec_buffer();
/// ```
///
/// ## Parser with custom buffer implementation
///
/// ```rust
/// use ublox::{ParserBuilder, FixedLinearBuffer};
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
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<P: UbxProtocol> ParserBuilder<P> {
    /// Specify a protocol version
    pub fn with_protocol<NewP: UbxProtocol>(self) -> ParserBuilder<NewP> {
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
    pub fn with_fixed_buffer<const N: usize>(self) -> Parser<FixedBuffer<N>, P> {
        Parser::with_fixed_buffer()
    }

    /// Build a parser with a custom buffer implementation
    pub fn with_buffer<T: UnderlyingBuffer>(self, buffer: T) -> Parser<T, P> {
        Parser::new(buffer)
    }
}

/// Streaming parser for UBX protocol with buffer. The default constructor will build
/// a parser containing a Vec, but you can pass your own underlying buffer by passing it
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
    pub fn new_fixed() -> Self {
        Self::with_fixed_buffer()
    }
}

impl<const N: usize, P: UbxProtocol> Parser<FixedBuffer<N>, P> {
    /// Creates a new parser with an owned, fixed-size internal buffer of size N.
    pub fn with_fixed_buffer() -> Self {
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
    pub fn new(underlying: T) -> Self {
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

    pub fn consume_ubx<'a>(&'a mut self, new_data: &'a [u8]) -> UbxParserIter<'a, T, P> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == SYNC_CHAR_1 {
                buf.drain(i);
                break;
            }
        }

        UbxParserIter {
            buf,
            _phantom: PhantomData,
        }
    }

    pub fn consume_ubx_rtcm<'a>(&'a mut self, new_data: &'a [u8]) -> UbxRtcmParserIter<'a, T, P> {
        let mut buf = DualBuffer::new(&mut self.buf, new_data);

        for i in 0..buf.len() {
            if buf[i] == SYNC_CHAR_1 || buf[i] == RTCM_SYNC_CHAR {
                buf.drain(i);
                break;
            }
        }

        UbxRtcmParserIter {
            buf,
            _phantom: PhantomData,
        }
    }
}

/// For ubx checksum on the fly
#[derive(Default)]
struct UbxChecksumCalc {
    ck_a: u8,
    ck_b: u8,
}

impl UbxChecksumCalc {
    fn new() -> Self {
        Self { ck_a: 0, ck_b: 0 }
    }

    fn update(&mut self, bytes: &[u8]) {
        let mut a = self.ck_a;
        let mut b = self.ck_b;
        for byte in bytes.iter() {
            a = a.overflowing_add(*byte).0;
            b = b.overflowing_add(a).0;
        }
        self.ck_a = a;
        self.ck_b = b;
    }

    fn result(self) -> (u8, u8) {
        (self.ck_a, self.ck_b)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum NextSync {
    Ubx(usize),
    Rtcm(usize),
    None,
}

#[derive(Debug)]
pub enum AnyPacketRef<'a> {
    Ubx(UbxPacket<'a>),
    Rtcm(RtcmPacketRef<'a>),
}

/// Iterator over data stored in `Parser` buffer
pub struct UbxParserIter<'a, T: UnderlyingBuffer, P: UbxProtocol = DefaultProtocol> {
    buf: DualBuffer<'a, T>,
    _phantom: PhantomData<P>,
}

fn extract_packet_ubx<'b, T: UnderlyingBuffer, P: UbxProtocol>(
    buf: &'b mut DualBuffer<'_, T>,
    pack_len: usize,
) -> Option<Result<UbxPacket<'b>, ParserError>> {
    if !buf.can_drain_and_take(6, pack_len + 2) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            buf.drain(2);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len + 2,
            }));
        }
        return None;
    }
    let mut checksummer = UbxChecksumCalc::new();
    let (a, b) = buf.peek_raw(2..(4 + pack_len + 2));
    checksummer.update(a);
    checksummer.update(b);
    let (ck_a, ck_b) = checksummer.result();

    let (expect_ck_a, expect_ck_b) = (buf[6 + pack_len], buf[6 + pack_len + 1]);
    if (ck_a, ck_b) != (expect_ck_a, expect_ck_b) {
        buf.drain(2);
        return Some(Err(ParserError::InvalidChecksum {
            expect: u16::from_le_bytes([expect_ck_a, expect_ck_b]),
            got: u16::from_le_bytes([ck_a, ck_b]),
        }));
    }
    let class_id = buf[2];
    let msg_id = buf[3];
    buf.drain(6);
    let msg_data = match buf.take(pack_len + 2) {
        Ok(x) => x,
        Err(e) => {
            return Some(Err(e));
        },
    };
    let specific_packet_result = P::match_packet(class_id, msg_id, &msg_data[..msg_data.len() - 2]);
    Some(specific_packet_result.map(|p| p.into()))
}

impl<T: UnderlyingBuffer, P: UbxProtocol> UbxParserIter<'_, T, P> {
    fn find_sync(&self) -> Option<usize> {
        (0..self.buf.len()).find(|&i| self.buf[i] == SYNC_CHAR_1)
    }

    #[allow(clippy::should_implement_trait)]
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of `<https://github.com/rust-lang/rust/issues/44265>`
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

            if self.buf.len() < 2 {
                return None;
            }
            if self.buf[1] != SYNC_CHAR_2 {
                self.buf.drain(1);
                continue;
            }

            if self.buf.len() < 6 {
                return None;
            }

            let pack_len: usize = u16::from_le_bytes([self.buf[4], self.buf[5]]).into();
            if pack_len > P::MAX_PAYLOAD_LEN {
                self.buf.drain(2);
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
    pack_len: usize,
) -> Option<Result<AnyPacketRef<'b>, ParserError>> {
    if !buf.can_drain_and_take(0, pack_len + 3) {
        if buf.potential_lost_bytes() > 0 {
            // We ran out of space, drop this packet and move on
            // TODO: shouldn't we drain pack_len + 3?
            buf.drain(2);
            return Some(Err(ParserError::OutOfMemory {
                required_size: pack_len + 2,
            }));
        }
        return None;
    }

    let maybe_data = buf.take(pack_len + 3);
    match maybe_data {
        Ok(data) => Some(Ok(AnyPacketRef::Rtcm(RtcmPacketRef::<'b> { data }))),
        Err(e) => Some(Err(e)),
    }
}

impl<T: UnderlyingBuffer, P: UbxProtocol> UbxRtcmParserIter<'_, T, P> {
    fn find_sync(&self) -> NextSync {
        for i in 0..self.buf.len() {
            if self.buf[i] == SYNC_CHAR_1 {
                return NextSync::Ubx(i);
            }
            if self.buf[i] == RTCM_SYNC_CHAR {
                return NextSync::Rtcm(i);
            }
        }
        NextSync::None
    }

    #[allow(clippy::should_implement_trait)]
    /// Analog of `core::iter::Iterator::next`, should be switched to
    /// trait implementation after merge of https://github.com/rust-lang/rust/issues/44265
    pub fn next(&mut self) -> Option<Result<AnyPacketRef<'_>, ParserError>> {
        while self.buf.len() > 0 {
            match self.find_sync() {
                NextSync::Ubx(pos) => {
                    self.buf.drain(pos);

                    if self.buf.len() < 2 {
                        return None;
                    }
                    if self.buf[1] != SYNC_CHAR_2 {
                        self.buf.drain(1);
                        continue;
                    }

                    if self.buf.len() < 6 {
                        return None;
                    }

                    let pack_len: usize = u16::from_le_bytes([self.buf[4], self.buf[5]]).into();
                    if pack_len > P::MAX_PAYLOAD_LEN {
                        self.buf.drain(2);
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

                    // need to read 3 bytes total for sync char (1) + length (2)
                    if self.buf.len() < 3 {
                        return None;
                    }
                    // next 2 bytes contain 6 bits reserved + 10 bits length, big endian
                    let pack_len: usize =
                        (u16::from_be_bytes([self.buf[1], self.buf[2]]) & 0x03ff).into();

                    return extract_packet_rtcm(&mut self.buf, pack_len);
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
        use crate::proto17::{PacketRef, Proto17};

        let mut buffer = [0; 12];
        let buffer = FixedLinearBuffer::new(&mut buffer);
        let mut parser: Parser<FixedLinearBuffer<'_>, Proto17> = Parser::new(buffer);
        {
            let mut it = parser.consume_ubx(&BYTES_GARBAGE);
            assert!(matches!(
                it.next(),
                Some(Ok(UbxPacket::Proto17(PacketRef::AckAck(_))))
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
        use crate::proto17::{PacketRef, Proto17};

        let bytes = test_util_cfg_nav5_bytes();

        let mut buffer = [0; 12];
        let mut parser = Parser::<_, Proto17>::new(FixedLinearBuffer::new(&mut buffer));

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
                Some(Ok(UbxPacket::Proto17(PacketRef::AckAck(_))))
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
        use crate::proto17::{PacketRef, Proto17};

        let bytes = test_util_cfg_nav5_bytes();
        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto17>()
            .with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto17(PacketRef::CfgNav5(_))))
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
        use crate::proto17::{PacketRef, Proto17};
        let bytes = test_util_cfg_nav5_bytes();

        let mut parser = Parser::<_, Proto17>::default();
        let mut it = parser.consume_ubx(&bytes);
        assert!(matches!(
            it.next(),
            Some(Ok(UbxPacket::Proto17(PacketRef::CfgNav5(_))))
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
        use crate::proto17::{PacketRef, Proto17};
        let data = test_util_multiple_cfg_nav5_packets_bytes();
        let mut parser = Parser::<_, Proto17>::default();
        let mut it = parser.consume_ubx(&data);
        match it.next() {
            Some(Ok(UbxPacket::Proto17(PacketRef::CfgNav5(packet)))) => {
                // We're good
                assert_eq!(packet.pacc(), 21);
            },
            _ => {
                panic!()
            },
        }
        match it.next() {
            Some(Ok(UbxPacket::Proto17(PacketRef::CfgNav5(packet)))) => {
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
}
