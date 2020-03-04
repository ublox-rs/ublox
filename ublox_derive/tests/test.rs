use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    sync::Arc,
};
use ublox_derive::{expand_ubx_packets_code_in_str, panic_on_parse_error};

#[test]
fn test_nav_pos_llh() {
    run_compare_test(
        quote! {
            #[ubx_packet_recv]
            #[ubx(class = 1, id = 2, fixed_payload_len = 12)]
            #[doc = "Geodetic Position Solution"]
            struct NavPosLLH {
                itow: u32,
                #[ubx(map_type = f64, scale = 1e-7, alias = lon_degrees)]
                lon: i32,
                #[doc = "Horizontal Accuracy Estimate"]
                horizontal_accuracy: u32,
            }
        },
        quote! {
            #[doc = "Geodetic Position Solution"]
            pub struct NavPosLLH;

            impl UbxPacket for NavPosLLH {
                const CLASS: u8 = 1u8;
                const ID: u8 = 2u8;
                const FIXED_PAYLOAD_LENGTH: Option<u16> = Some(12u16);
            }

            #[doc = "Geodetic Position Solution"]
            #[doc = "It is just reference to internal parser's buffer"]
            pub struct NavPosLLHRef<'a>(&'a [u8]);
            impl<'a> NavPosLLHRef<'a> {
                #[doc = ""]
                #[inline]
                pub fn itow(&self) -> u32 {
                    <u32>::from_le_bytes([
                        self.0[0usize],
                        self.0[1usize],
                        self.0[2usize],
                        self.0[3usize]]
                    )
                }
                #[doc = ""]
                #[inline]
                pub fn lon_degrees(&self) -> f64 {
                    let val = <i32>::from_le_bytes([
                        self.0[4usize],
                        self.0[5usize],
                        self.0[6usize],
                        self.0[7usize]]
                    );
                    let val = <f64>::from(val);
                    val * 1e-7
                }

                #[doc = "Horizontal Accuracy Estimate"]
                #[inline]
                pub fn horizontal_accuracy(&self) -> u32 {
                    <u32>::from_le_bytes([
                        self.0[8usize],
                        self.0[9usize],
                        self.0[10usize],
                        self.0[11usize]
                    ])
                }
            }

            #[doc = "All possible packets enum"]
            pub enum PacketRef<'a> {
                NavPosLLH(NavPosLLHRef<'a>),
                Unknown(UnknownPacketRef<'a>),
            }
            fn match_packet(class: u8, msg_id: u8, payload: &[u8]) -> Option<Result<PacketRef, ParserError>> {
                match (class, msg_id) {
                    (NavPosLLH::CLASS, NavPosLLH::ID) => {
                        if 12usize != payload.len() {
                            return Some(Err(ParserError::InvalidPacketLen("NavPosLLH")));
                        }
                        Some(Ok(PacketRef::NavPosLLH(NavPosLLHRef(payload))))
                    }
                    _ => Some(Ok(PacketRef::Unknown(UnknownPacketRef {
                        payload,
                        class,
                        msg_id,
                    }))),
                }
            }
        },
        Flags::Equal,
    );
}

#[test]
fn test_nav_status() {
    run_compare_test(
        quote! {
            #[ubx_type]
            #[ubx(from, to, rest_reserved)]
            #[repr(u8)]
            #[derive(Debug, Copy, Clone)]
            enum GpsFix {
                NoFix = 0,
                DeadReckoningOnly = 1,
                Fix2D = 2,
                Fix3D = 3,
                GPS = 4,
            }

            #[ubx_type]
            #[ubx(from_unchecked, to, rest_error)]
            #[derive(Copy, Clone)]
            #[repr(u8)]
            enum DGPSCorrectionStatus {
                None = 0,
                PrPrrCorrected = 1,
            }


            #[ubx_packet_recv]
            #[ubx(class = 1, id = 3, fixed_payload_len = 6)]
            struct Status {
                itow: u32,
                #[ubx(map_type = GpsFix)]
                gps_fix: u8,
                #[ubx(map_type = DGPSCorrectionStatus)]
                dgps_status: u8,
            }
        },
        quote! {
            #[doc = ""]
            pub struct Status;
            impl UbxPacket for Status {
                const CLASS: u8 = 1u8;
                const ID: u8 = 3u8;
                const FIXED_PAYLOAD_LENGTH: Option<u16> = Some(6u16);
            }

            #[doc = ""]
            #[doc = "It is just reference to internal parser's buffer"]
            pub struct StatusRef<'a>(&'a [u8]);
            impl<'a> StatusRef<'a> {
                #[doc = ""]
                #[inline]
                pub fn itow(&self) -> u32 {
                    <u32>::from_le_bytes([
                        self.0[0usize],
                        self.0[1usize],
                        self.0[2usize],
                        self.0[3usize],
                    ])
                }
                #[doc = ""]
                #[inline]
                pub fn gps_fix(&self) -> GpsFix {
                    let val = self.0[4usize];
                    <GpsFix>::from(val)
                }
                #[doc = ""]
                #[inline]
                pub fn dgps_status(&self) -> DGPSCorrectionStatus {
                    let val = self.0[5usize];
                    <DGPSCorrectionStatus>::from_unchecked(val)
                }
            }
        },
        Flags::Contains,
    );
}

enum Flags {
    Equal,
    Contains,
}

fn run_compare_test(input: TokenStream, expect_output: TokenStream, flags: Flags) {
    let src = input.to_string();
    let res = match expand_ubx_packets_code_in_str(&src) {
        Ok(x) => x,
        Err(err) => panic_on_parse_error((std::path::Path::new(""), &src), &err),
    };
    let output = String::from_utf8(rustfmt_cnt(res.into_bytes()).unwrap()).unwrap();

    let expect_output = expect_output.into_token_stream().to_string();
    let expect_output =
        String::from_utf8(rustfmt_cnt(expect_output.into_bytes()).unwrap()).unwrap();

    match flags {
        Flags::Equal => {
            if expect_output != output {
                for (e, g) in expect_output.lines().zip(output.lines()) {
                    if e != g {
                        println!("first mismatch:\ne {}\ng {}", e, g);
                        break;
                    }
                }
                panic!("Expect:\n{}\nGot:\n{}\n", expect_output, output);
            }
        }
        Flags::Contains => {
            if !output.contains(&expect_output) {
                panic!(
                    "Output doesn't contain Expect\n
Expect:\n{}\nGot:\n{}\n",
                    expect_output, output
                );
            }
        }
    }
}

fn rustfmt_cnt(source: Vec<u8>) -> io::Result<Vec<u8>> {
    let rustfmt = which::which("rustfmt")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

    let mut cmd = Command::new(&*rustfmt);

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let src_len = source.len();
    let src = Arc::new(source);
    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(src.as_slice());
        src
    });

    let mut output = Vec::with_capacity(src_len);
    io::copy(&mut child_stdout, &mut output)?;
    let status = child.wait()?;
    let src = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
         anything that could panic",
    );
    let src =
        Arc::try_unwrap(src).expect("Internal error: rusftfmt_cnt should only one Arc refernce");
    match status.code() {
        Some(0) => Ok(output),
        Some(2) => Err(io::Error::new(
            io::ErrorKind::Other,
            "Rustfmt parsing errors.".to_string(),
        )),
        Some(3) => {
            println!("warning=Rustfmt could not format some lines.");
            Ok(src)
        }
        _ => {
            println!("warning=Internal rustfmt error");
            Ok(src)
        }
    }
}
