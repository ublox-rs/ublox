use crate::{
    constants::{UBX_CHECKSUM_LEN, UBX_CLASS_OFFSET, UBX_HEADER_LEN, UBX_LENGTH_OFFSET},
    parser::buffer::DualBuffer,
    ParserError, UnderlyingBuffer,
};

/// UBX [Fletcher-16 checksum](https://en.wikipedia.org/wiki/Fletcher%27s_checksum) calculator supporting both streaming and single-shot validation
#[derive(Default)]
pub(crate) struct UbxChecksumCalc {
    ck_a: u8,
    ck_b: u8,
}

impl UbxChecksumCalc {
    pub(crate) const fn new() -> Self {
        Self { ck_a: 0, ck_b: 0 }
    }

    /// Update checksum with new bytes
    pub(crate) const fn update(&mut self, bytes: &[u8]) {
        let mut i = 0;
        while i < bytes.len() {
            self.update_byte(bytes[i]);
            i += 1;
        }
    }

    /// Update checksum with a single byte
    pub(crate) const fn update_byte(&mut self, byte: u8) {
        self.ck_a = self.ck_a.wrapping_add(byte);
        self.ck_b = self.ck_b.wrapping_add(self.ck_a);
    }

    /// Get the current checksum result
    pub(crate) const fn result(self) -> (u8, u8) {
        (self.ck_a, self.ck_b)
    }

    /// Validate checksum and return result
    pub(crate) const fn validate_result(
        self,
        received_ck_a: u8,
        received_ck_b: u8,
    ) -> Result<(), ParserError> {
        let is_valid = self.is_valid(received_ck_a, received_ck_b);
        let (calculated_ck_a, calculated_ck_b) = self.result();
        if is_valid {
            Ok(())
        } else {
            Err(ParserError::InvalidChecksum {
                expect: u16::from_le_bytes([received_ck_a, received_ck_b]),
                got: u16::from_le_bytes([calculated_ck_a, calculated_ck_b]),
            })
        }
    }

    /// Single-shot validation against buffer contents (convenience method)
    pub(crate) fn validate_buffer<T: UnderlyingBuffer>(
        buf: &DualBuffer<'_, T>,
        pack_len: u16,
    ) -> Result<(), ParserError> {
        let pack_len = pack_len as usize; // `usize` is needed for indexing but constraining the input to `u16` is still important
        let mut calc = Self::new();
        let (class_msg_bytes, payload_and_checksum) =
            buf.peek_raw(UBX_CLASS_OFFSET..(UBX_LENGTH_OFFSET + pack_len + UBX_CHECKSUM_LEN));
        let (received_ck_a, received_ck_b) = (
            buf[UBX_HEADER_LEN + pack_len],
            buf[UBX_HEADER_LEN + pack_len + 1],
        );

        // Calculate checksum over class, message ID, length, and payload
        calc.update(class_msg_bytes);
        calc.update(payload_and_checksum);

        calc.validate_result(received_ck_a, received_ck_b)
    }

    const fn is_valid(&self, received_ck_a: u8, received_ck_b: u8) -> bool {
        self.ck_a == received_ck_a && self.ck_b == received_ck_b
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2},
        FixedBuffer,
    };

    use super::*;

    const PACK_LEN: u8 = 2;
    const VALID_CK_A: u8 = 0x11;
    const VALID_CK_B: u8 = 0x38;
    // UBX-ACK-ACK packet: Class=0x05, ID=0x01, Length=0x0002, Payload=[0x04, 0x05], Checksum=[0x11, 0x38]
    const VALID_UBX_PACKET: [u8; 10] = [
        crate::constants::UBX_SYNC_CHAR_1,
        crate::constants::UBX_SYNC_CHAR_2, // Sync chars (not included in checksum)
        0x05,
        0x01, // Class and Message ID
        PACK_LEN,
        0x00, // Length (2 bytes)
        0x04,
        0x05, // Payload
        VALID_CK_A,
        VALID_CK_B, // Checksum
    ];

    // Helper function to create a valid UBX packet with correct checksum
    const fn create_valid_ubx_packet() -> (u16, [u8; 10]) {
        (PACK_LEN as u16, VALID_UBX_PACKET)
    }

    // Helper function to create an invalid UBX packet with wrong checksum
    const fn create_invalid_ubx_packet() -> (u16, [u8; 10]) {
        let (pack_len, mut packet) = create_valid_ubx_packet();
        // Corrupt the checksum
        let buf_len = packet.len();
        packet[buf_len - 1] = packet[buf_len - 1].wrapping_add(1);
        (pack_len, packet)
    }

    #[test]
    fn test_streaming_checksum_valid() {
        let (_, packet) = create_valid_ubx_packet();
        let mut calc = UbxChecksumCalc::new();

        // Update with class, message ID, length, and payload
        calc.update(&packet[2..8]);

        let (received_ck_a, received_ck_b) = (packet[8], packet[9]);
        assert!(calc.validate_result(received_ck_a, received_ck_b).is_ok());
    }

    #[test]
    fn test_streaming_checksum_invalid() {
        let (_, packet) = create_invalid_ubx_packet();
        let mut calc = UbxChecksumCalc::new();

        // Update with class, message ID, length, and payload
        calc.update(&packet[2..8]);

        let (received_ck_a, received_ck_b) = (packet[8], packet[9]);
        let result = calc.validate_result(received_ck_a, received_ck_b);

        assert!(result.is_err());
        if let Err(ParserError::InvalidChecksum { expect, got }) = result {
            assert_ne!(expect, got);
        }
    }

    #[test]
    fn test_streaming_checksum_incremental() {
        let (_, packet) = create_valid_ubx_packet();
        let mut calc = UbxChecksumCalc::new();

        // Update byte by byte to test incremental calculation
        for byte in &packet[2..8] {
            calc.update_byte(*byte);
        }

        let (received_ck_a, received_ck_b) = (packet[8], packet[9]);

        assert_eq!(calc.validate_result(received_ck_a, received_ck_b), Ok(()));
    }

    #[test]
    fn test_streaming_checksum_chunks() {
        let (_, packet) = create_valid_ubx_packet();
        let mut calc = UbxChecksumCalc::new();

        // Update in chunks
        calc.update(&packet[2..4]); // Class and ID
        calc.update(&packet[4..6]); // Length
        calc.update(&packet[6..8]); // Payload

        let (received_ck_a, received_ck_b) = (packet[8], packet[9]);
        assert_eq!(calc.validate_result(received_ck_a, received_ck_b), Ok(()));
    }

    #[test]
    fn test_buffer_validation_valid() {
        let (pack_len, packet) = create_valid_ubx_packet();
        let mut buf = FixedBuffer::<128>::new();
        let dual_buffer = DualBuffer::new(&mut buf, &packet);

        assert_eq!(
            UbxChecksumCalc::validate_buffer(&dual_buffer, pack_len),
            Ok(())
        );
    }

    #[test]
    fn test_buffer_validation_invalid() {
        let (pack_len, packet) = create_invalid_ubx_packet();
        let mut buf: FixedBuffer<1024> = FixedBuffer::new();

        let dual_buffer = DualBuffer::new(&mut buf, &packet);

        let err = UbxChecksumCalc::validate_buffer(&dual_buffer, pack_len).unwrap_err();
        assert!(matches!(err, ParserError::InvalidChecksum { .. }));
        if let ParserError::InvalidChecksum { expect, got } = err {
            assert_ne!(expect, got);
        }
    }

    #[test]
    fn test_empty_payload_checksum() {
        // Create packet with no payload
        let packet = [
            UBX_SYNC_CHAR_1,
            UBX_SYNC_CHAR_2,
            0x05,
            0x00, // Class and Message ID
            0x00,
            0x00, // Length = 0
        ];

        // Calculate checksum for empty payload
        let mut calc = UbxChecksumCalc::new();
        calc.update(&packet[2..]); // Class, ID, Length only
        let (ck_a, ck_b) = calc.result();

        // Test streaming validation
        let mut calc = UbxChecksumCalc::new();
        calc.update(&packet[2..6]); // Class, ID, Length (no payload)
        assert_eq!(calc.validate_result(ck_a, ck_b), Ok(()));
    }

    #[test]
    fn test_streaming_vs_buffer_consistency() {
        let (pack_len, packet) = create_valid_ubx_packet();

        // Test streaming method
        let mut calc = UbxChecksumCalc::new();
        calc.update(&packet[2..8]); // Class, ID, Length, Payload
        let streaming_result = calc.validate_result(packet[8], packet[9]);

        // Test buffer method
        let mut buf: FixedBuffer<128> = FixedBuffer::new();
        let dual_buffer = DualBuffer::new(&mut buf, &packet);
        let buffer_result = UbxChecksumCalc::validate_buffer(&dual_buffer, pack_len);

        // Both should give same result
        assert_eq!(streaming_result.is_ok(), buffer_result.is_ok());

        if let (Err(streaming_err), Err(buffer_err)) = (&streaming_result, &buffer_result) {
            // Both should have same error details
            match (streaming_err, buffer_err) {
                (
                    ParserError::InvalidChecksum {
                        expect: e1,
                        got: g1,
                    },
                    ParserError::InvalidChecksum {
                        expect: e2,
                        got: g2,
                    },
                ) => {
                    assert_eq!(e1, e2);
                    assert_eq!(g1, g2);
                },
                _ => panic!("Error types should match"),
            }
        }
    }

    // Compute checksum at compile time
    #[allow(dead_code, reason = "constant time evaluated")]
    const fn is_checksum_valid(bytes: &[u8], expected_ck_a: u8, expected_ck_b: u8) -> bool {
        let mut calc = UbxChecksumCalc::new();
        calc.update(bytes);
        calc.is_valid(expected_ck_a, expected_ck_b)
    }

    #[test]
    fn test_const_checksum_computation() {
        // Compile-time assertion
        const _: () = {
            assert!(is_checksum_valid(
                &[
                    0x05, 0x01, // Class and Message ID
                    PACK_LEN, 0x00, // Length (2 bytes)
                    0x04, 0x05, // Payload
                ],
                VALID_CK_A,
                VALID_CK_B
            ));
        };
    }
}
