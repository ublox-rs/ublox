//! Adaptative UBX + RTCM parser

use crate::{
    error::ParserError,
    parser::{ubx::extract_packet_ubx, DualBuffer, UnderlyingBuffer},
    ubx_packets::{PacketRef, MAX_PAYLOAD_LEN, RTCM_SYNC_CHAR, SYNC_CHAR_1, SYNC_CHAR_2},
};

#[cfg(feature = "rtcm")]
use rtcm_rs::{next_msg_frame as find_next_rtcm_frame, MessageFrame as RtcmMessageFrame};

#[derive(Debug, PartialEq, Eq)]
enum NextSync {
    Ubx(usize),
    Rtcm(usize),
    None,
}

/// [AnyPacketRef] allows identifying UBX [PacketRef]s and RTCM packets on the fly.
pub enum AnyPacketRef<'a> {
    Ubx(PacketRef<'a>),

    #[cfg(feature = "rtcm")]
    Rtcm(RtcmMessageFrame<'a>),

    #[cfg(not(feature = "rtcm"))]
    /// Reference to underlying RTCM bytes, "as is".
    Rtcm(RtcmPacketRef<'a>),
}

/// Iterator over data stored in `Parser` buffer. Both UBX and RTCM
/// packets will be identified on the fly.
pub struct UbxRtcmParserIter<'a, T: UnderlyingBuffer> {
    pub(crate) buf: DualBuffer<'a, T>,
}

/// Reference to RTCM content, "as is".
/// When compiled with `rtcm` feature, we can forward this content to
/// the RTCM parser.
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

    let (consumed, msg_frame) = match maybe_data {
        Ok(data) => find_next_rtcm_frame(data),
        Err(e) => {
            return Some(Err(e));
        },
    };

    if consumed > 0 {
        // remove bytes that have been consumed by RTCM identification attempt
    }

    if let Some(msg_frame) = msg_frame {
        // RTCM did find something
        Some(Ok(AnyPacketRef::Rtcm(msg_frame)))
    } else {
        None
    }
}

impl<T: UnderlyingBuffer> UbxRtcmParserIter<'_, T> {
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
                    if pack_len > usize::from(MAX_PAYLOAD_LEN) {
                        self.buf.drain(2);
                        continue;
                    }
                    let maybe_packet = extract_packet_ubx(&mut self.buf, pack_len);
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

#[cfg(feature = "rtcm")]
#[cfg_attr(docsrs, doc(cfg(feature = "rtcm")))]
impl<'a> RtcmPacketRef<'a> {
    /// [RtcmMessageFrame] decoding attempt, from pre-identified RTCM packet content.
    pub fn interpret(&self) -> Option<RtcmMessageFrame> {
        // we're already pointing to the first byte, thanks to previous work.
        // Simply grab one frame from that content
        let (_, msg_frame) = find_next_rtcm_frame(&self.data);
        let msg_frame = msg_frame?;
        Some(msg_frame)
    }
}
