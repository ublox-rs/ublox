//! Shared helpers for the UBX packet fuzz tests.
//!
//! Keep this module minimal: only protocol-level primitives (checksum, frame
//! envelope) and generic proptest strategies belong here. Per-packet payload
//! structs and their `*_payload_strategy()` / `to_bytes()` helpers stay in
//! their own test files so that each packet's correctness remains
//! independently auditable.

#![allow(dead_code)] // each test binary uses only a subset of these helpers

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::constants::{UBX_SYNC_CHAR_1, UBX_SYNC_CHAR_2};

/// Calculates the 8-bit Fletcher-16 checksum used by U-Blox.
///
/// `data` is the checksummed region of the frame: class, id, length and
/// payload (everything between the sync chars and the checksum bytes).
pub fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

/// Assembles a complete, valid UBX frame from a class/id and payload:
/// `[sync1, sync2, class, id, len_lo, len_hi, payload..., ck_a, ck_b]`.
///
/// The length field is the payload length as a little-endian `u16`, and the
/// checksum is computed over `class..=payload` per [`calculate_checksum`].
pub fn build_ubx_frame(class: u8, id: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame_core = Vec::with_capacity(4 + payload.len());
    frame_core.push(class);
    frame_core.push(id);
    frame_core
        .write_u16::<LittleEndian>(payload.len() as u16)
        .unwrap();
    frame_core.extend_from_slice(payload);

    let (ck_a, ck_b) = calculate_checksum(&frame_core);

    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.push(UBX_SYNC_CHAR_1);
    frame.push(UBX_SYNC_CHAR_2);
    frame.extend_from_slice(&frame_core);
    frame.push(ck_a);
    frame.push(ck_b);
    frame
}

/// A proptest strategy that generates only finite `f32` values (no NaN/inf).
pub fn finite_f32() -> impl Strategy<Value = f32> {
    any::<u32>().prop_filter_map("finite f32", |bits| {
        let f = f32::from_bits(bits);
        if f.is_finite() {
            Some(f)
        } else {
            None
        }
    })
}
