use ublox::{PacketRef, Parser, RxmSfrbxInterpreted};

use gnss_protos::GpsQzssSubframe;

/**
 * UBX-RXM-SFRBX GPS Eph#1 L1/CA parsing and interpretation
 */
#[test]
#[cfg(feature = "ubx_proto23")]
fn sfrbx_gps_eph1() {
    let bytes = [
        0xb5,
        0x62,
        0x02,
        0x13, // UBX
        12 + 36,
        0, // LENGTH
        0,
        1,
        2,
        3, // SFRBX (1)
        1,
        5,
        6,
        7, // SFRBX (2)
        0x1B,
        0x3E,
        0xC1,
        0x22, // 1
        0x73,
        0xC9,
        0x27,
        0x15, // 2
        0x04,
        0x00,
        0xE4,
        0x13, // 3
        0x31,
        0x5D,
        0x4F,
        0x10, // 4
        0xD7,
        0xE6,
        0x44,
        0x97, // 5
        0x83,
        0x57,
        0x75,
        0x07, // 6
        0xB5,
        0x80,
        0x0C,
        0x33, // 7
        0xA1,
        0x42,
        0x50,
        0x92, // 8
        0x84,
        0x16,
        0x00,
        0x80, // 9
        0x33,
        0x30,
        0x2C,
        0x31, // 10
        0xFB, // CK_A
        0xE6, // CK_B
    ];

    let mut test_passed = false;
    let mut parser = Parser::default();
    let mut it = parser.consume_ubx(&bytes);

    while let Some(pack) = it.next() {
        match pack {
            Ok(PacketRef::RxmSfrbx(packet)) => {
                assert_eq!(packet.gnss_id(), 0);
                assert_eq!(packet.sv_id(), 1);
                assert_eq!(packet.reserved1(), 2);
                assert_eq!(packet.freq_id(), 3);
                assert_eq!(packet.num_words(), 1);
                assert_eq!(packet.reserved2(), 5);
                assert_eq!(packet.version(), 6);
                assert_eq!(packet.reserved3(), 7);

                // GPS interpretation
                let interpreted = packet
                    .interpret()
                    .unwrap_or_else(|| panic!("UBX-SFRBX (GPS/QZSS) interpretation failed!"));

                match interpreted {
                    RxmSfrbxInterpreted::GpsQzss(frame) => match frame.subframe {
                        GpsQzssSubframe::Ephemeris1(subframe) => {
                            assert_eq!(subframe.af2, 0.0);
                            assert!((subframe.af1 - 1.023181539495e-011).abs() < 1e-14);
                            assert!((subframe.af0 - -4.524961113930e-004).abs() < 1.0e-11);
                            assert_eq!(subframe.week, 318);
                            assert_eq!(subframe.toc, 266_400);
                            assert_eq!(subframe.health, 0);
                        },
                        _ => panic!("UBX-SFRBX (GPS/QZSS) invalid subframe interpretation!"),
                    },
                    _ => panic!("UBX-SFRBX (GPS/QZSS) incorrect interpretation"),
                }
                test_passed = true;
            },
            Ok(_) => panic!("found invalid packet"),
            Err(e) => panic!("UBX-SFRBX parsing failed with {}", e),
        }
    }
    assert!(test_passed, "UBX-SFRBX test failed");
}

/**
 * UBX-RXM-SFRBX GPS Eph#2 L1/CA parsing and interpretation
 */
#[test]
#[cfg(feature = "ubx_proto23")]
fn sfrbx_gps_eph2() {
    let bytes = [
        0xb5,
        0x62,
        0x02,
        0x13, // UBX
        12 + 36,
        0, // LENGTH
        0,
        1,
        2,
        3, // SFRBX (1)
        1,
        5,
        6,
        7, // SFRBX (2)
        0x1B,
        0x3E,
        0xC1,
        0x22, // 1
        0x1B,
        0xEA,
        0x27,
        0x15, // 2
        0x65,
        0xF1,
        0x7F,
        0x12, // 3
        0x7C,
        0x1F,
        0x68,
        0x8C, // 3
        0x15,
        0x34,
        0x49,
        0x02, // 4
        0x1E,
        0x81,
        0xF8,
        0xBF, // 5
        0x14,
        0x81,
        0x1B,
        0x99, // 6
        0x6E,
        0x68,
        0x3E,
        0x04, // 7
        0x21,
        0x72,
        0x34,
        0x83, // 8
        0x7B,
        0x9F,
        0x42,
        0x90, // 9
        0xD2, // CK_A
        0x9A, // CK_B
    ];

    let mut test_passed = false;
    let mut parser = Parser::default();
    let mut it = parser.consume_ubx(&bytes);

    while let Some(pack) = it.next() {
        match pack {
            Ok(PacketRef::RxmSfrbx(packet)) => {
                assert_eq!(packet.gnss_id(), 0);
                assert_eq!(packet.sv_id(), 1);
                assert_eq!(packet.reserved1(), 2);
                assert_eq!(packet.freq_id(), 3);
                assert_eq!(packet.num_words(), 1);
                assert_eq!(packet.reserved2(), 5);
                assert_eq!(packet.version(), 6);
                assert_eq!(packet.reserved3(), 7);

                // GPS interpretation
                let interpreted = packet
                    .interpret()
                    .unwrap_or_else(|| panic!("UBX-SFRBX (GPS/QZSS) interpretation failed!"));

                match interpreted {
                    RxmSfrbxInterpreted::GpsQzss(frame) => match frame.subframe {
                        GpsQzssSubframe::Ephemeris2(subframe) => {
                            assert_eq!(subframe.toe, 266_400);
                            assert_eq!(subframe.crs, -1.843750000000e+000);
                            assert!((subframe.sqrt_a - 5.153602432251e+003).abs() < 1e-9);
                            assert!((subframe.m0 - 9.768415465951e-001).abs() < 1e-9);
                            assert!((subframe.cuc - -5.587935447693e-008).abs() < 1e-9);
                            assert!((subframe.e - 8.578718174249e-003).abs() < 1e-9);
                            assert!((subframe.cus - 8.093193173409e-006).abs() < 1e-9);
                            assert!((subframe.cuc - -5.587935447693e-008).abs() < 1e-6);
                            assert!((subframe.dn - 1.444277586415e-009).abs() < 1e-9);
                            assert_eq!(subframe.fit_int_flag, false);
                        },
                        _ => panic!("UBX-SFRBX (GPS/QZSS) invalid subframe interpretation!"),
                    },
                    _ => panic!("UBX-SFRBX (GPS/QZSS) incorrect interpretation"),
                }
                test_passed = true;
            },
            Ok(_) => panic!("found invalid packet"),
            Err(e) => panic!("UBX-SFRBX parsing failed with {}", e),
        }
    }
    assert!(test_passed, "UBX-SFRBX test failed");
}

/**
 * UBX-RXM-SFRBX GPS Eph#3 L1/CA parsing and interpretation
 */
#[test]
#[cfg(feature = "ubx_proto23")]
fn sfrbx_gps_eph3() {
    let bytes = [
        0xb5,
        0x62,
        0x02,
        0x13, // UBX
        12 + 36,
        0, // LENGTH
        0,
        1,
        2,
        3, // SFRBX (1)
        1,
        5,
        6,
        7, // SFRBX (2)
        0x1B,
        0x3E,
        0xC1,
        0x22, // 1
        0xDB,
        0x0B,
        0x28,
        0x15, // 2
        0x34,
        0xEA,
        0x0A,
        0x00, // 3
        0xEE,
        0xFF,
        0x3C,
        0x03, // 4
        0xEB,
        0xC9,
        0xE5,
        0xBF, // 5
        0x4E,
        0xB6,
        0x6F,
        0x13, // 6
        0x2C,
        0xAB,
        0xF4,
        0x86, // 7
        0x44,
        0xEB,
        0x71,
        0x06, // 8
        0x02,
        0xF6,
        0xEA,
        0x3F, // 9
        0x13,
        0x52,
        0x45,
        0x92, // 10
        0x43, // CK_A
        0x6D, // CK_B
    ];

    let mut test_passed = false;
    let mut parser = Parser::default();
    let mut it = parser.consume_ubx(&bytes);

    while let Some(pack) = it.next() {
        match pack {
            Ok(PacketRef::RxmSfrbx(packet)) => {
                assert_eq!(packet.gnss_id(), 0);
                assert_eq!(packet.sv_id(), 1);
                assert_eq!(packet.reserved1(), 2);
                assert_eq!(packet.freq_id(), 3);
                assert_eq!(packet.num_words(), 1);
                assert_eq!(packet.reserved2(), 5);
                assert_eq!(packet.version(), 6);
                assert_eq!(packet.reserved3(), 7);

                // GPS interpretation
                let interpreted = packet
                    .interpret()
                    .unwrap_or_else(|| panic!("UBX-SFRBX (GPS/QZSS) interpretation failed!"));

                match interpreted {
                    RxmSfrbxInterpreted::GpsQzss(frame) => match frame.subframe {
                        GpsQzssSubframe::Ephemeris3(subframe) => {
                            assert!((subframe.cic - 8.009374141693e-008).abs() < 1e-9);
                            assert!((subframe.cis - -1.955777406693e-007).abs() < 1e-9);
                            assert!((subframe.crc - 2.225625000000e+002).abs() < 1e-9);
                            assert!((subframe.i0 - 3.070601043291e-001).abs() < 1e-9);
                            assert!((subframe.idot - 1.548414729768e-010).abs() < 1e-9);
                            assert!((subframe.omega0 - -6.871047024615e-001).abs() < 1e-9);
                            assert!((subframe.omega_dot - -2.449269231874e-009).abs() < 1e-9);
                            assert!((subframe.omega - -6.554632573389e-001).abs() < 1e-9);
                        },
                        _ => panic!("UBX-SFRBX (GPS/QZSS) invalid subframe interpretation!"),
                    },
                    _ => panic!("UBX-SFRBX (GPS/QZSS) incorrect interpretation"),
                }
                test_passed = true;
            },
            Ok(_) => panic!("found invalid packet"),
            Err(e) => panic!("UBX-SFRBX parsing failed with {}", e),
        }
    }
    assert!(test_passed, "UBX-SFRBX test failed");
}
