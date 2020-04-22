use super::*;
use quote::quote;
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    sync::Arc,
};
use syn::Error;
use which::which;

#[test]
fn test_ubx_packet_recv_simple() {
    let src_code = quote! {
        #[ubx_packet_recv]
        #[ubx(class = 1, id = 2, fixed_payload_len = 16)]
        #[doc = "Some comment"]
        struct Test {
            itow: u32,
            #[doc = "this is lat"]
            #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
            lat: i32,
            #[doc = "this is a"]
            a: u8,
            reserved1: [u8; 5],
            #[ubx(map_type = Flags, may_fail)]
            flags: u8,
            b: i8,
        }
    };
    let src_code = src_code.to_string();

    let code: syn::ItemStruct = syn::parse_str(&src_code)
        .unwrap_or_else(|err| panic_on_parse_error("test_ubx_packet_recv", &src_code, &err));
    let tokens = generate_code_for_recv_packet(code.ident, code.attrs, code.fields)
        .unwrap_or_else(|err| panic_on_parse_error("test_ubx_packet_recv", &src_code, &err));

    run_compare_test(
        tokens,
        quote! {
            #[doc = "Some comment"]
            pub struct Test;

            impl UbxPacketMeta for Test {
                const CLASS: u8 = 1u8;
                const ID: u8 = 2u8;
                const FIXED_PAYLOAD_LEN: Option<u16> = Some(16u16);
                const MAX_PAYLOAD_LEN: u16 = 16u16;
            }

            #[doc = "Some comment"]
            #[doc = "It is just reference to internal parser's buffer"]
            pub struct TestRef<'a>(&'a [u8]);
            impl<'a> TestRef<'a> {
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
                #[doc = "this is lat"]
                #[inline]
                pub fn lat_degrees(&self) -> f64 {
                    let val = <i32>::from_le_bytes([
                        self.0[4usize],
                        self.0[5usize],
                        self.0[6usize],
                        self.0[7usize]]
                    );
                    let val = <f64>::from(val);
                    val * 1e-7
                }
                #[doc = "this is a"]
                #[inline]
                pub fn a(&self) -> u8 {
                    self.0[8usize]
                }
                #[doc = ""]
                #[inline]
                pub fn reserved1(&self) -> [u8; 5] {
                    [
                        self.0[9usize],
                        self.0[10usize],
                        self.0[11usize],
                        self.0[12usize],
                        self.0[13usize],
                    ]
                }
                #[doc = ""]
                #[inline]
                pub fn flags(&self) -> Flags {
                    let val = self.0[14usize];
                    <Flags>::from_unchecked(val)
                }
                #[doc = ""]
                #[inline]
                pub fn b(&self) -> i8 {
                    <i8>::from_le_bytes([self.0[15usize]])
                }

                fn validate(payload: &[u8]) -> Result<(), ParserError> {
                    let expect = 16usize;
                    let got = payload.len();
                    if got ==  expect {
                        let val = payload[14usize];
                        if !<Flags>::is_valid(val) {
                            return Err(ParserError::InvalidField{packet: "Test", field: stringify!(flags)});
                        }
                        Ok(())
                    } else {
                        Err(ParserError::InvalidPacketLen{packet: "Test", expect, got})
                    }
                }
            }
        },
    );
}

#[test]
fn test_ubx_packet_recv_dyn_len() {
    let src_code = quote! {
        #[ubx_packet_recv]
        #[ubx(class = 1, id = 2, max_payload_len = 38)]
        struct Test {
            #[ubx(map_type = &str, get_as_ref, from = unpack_str)]
            f1: [u8; 8],
            rest: [u8; 0],
        }
    };
    let src_code = src_code.to_string();

    let code: syn::ItemStruct = syn::parse_str(&src_code).unwrap_or_else(|err| {
        panic_on_parse_error("test_ubx_packet_recv_dyn_len", &src_code, &err)
    });
    let tokens =
        generate_code_for_recv_packet(code.ident, code.attrs, code.fields).unwrap_or_else(|err| {
            panic_on_parse_error("test_ubx_packet_recv_dyn_len", &src_code, &err)
        });

    run_compare_test(
        tokens,
        quote! {
            #[doc = ""]
            pub struct Test;

            impl UbxPacketMeta for Test {
                const CLASS: u8 = 1u8;
                const ID: u8 = 2u8;
                const FIXED_PAYLOAD_LEN: Option<u16> = None;
                const MAX_PAYLOAD_LEN: u16 = 38u16;
            }

            #[doc = ""]
            #[doc = "It is just reference to internal parser's buffer"]
            pub struct TestRef<'a>(&'a [u8]);
            impl<'a> TestRef<'a> {
                #[doc = ""]
                #[inline]
                pub fn f1(&self) -> &str {
                    let val = &self.0[0usize..(0usize + 8usize)];
                    unpack_str(val)
                }

                #[doc = ""]
                #[inline]
                pub fn rest(&self) -> &[u8] {
                    &self.0[8usize..]
                }

                fn validate(payload: &[u8]) -> Result<(), ParserError> {
                    let min = 8usize;
                    let got = payload.len();
                    if got >= min {
                        Ok(())
                    } else {
                        Err(ParserError::InvalidPacketLen{packet: "Test", expect: min, got})
                    }
                }
            }
        },
    );
}

#[test]
fn test_ubx_packet_send() {
    let src_code = quote! {
        #[ubx_packet_send]
        #[ubx(class = 1, id = 2, fixed_payload_len = 9, flags = "default_for_builder")]
        #[doc = "Some comment"]
        struct Test {
            itow: u32,
            #[doc = "this is lat"]
            #[ubx(map_type = f64, scale = 1e-7, alias = lat_degrees)]
            lat: i32,
            #[doc = "this is a"]
            a: u8,
        }
    };
    let src_code = src_code.to_string();

    let code: syn::ItemStruct = syn::parse_str(&src_code)
        .unwrap_or_else(|err| panic_on_parse_error("test_ubx_packet_send", &src_code, &err));
    let tokens = generate_code_for_send_packet(code.ident, code.attrs, code.fields)
        .unwrap_or_else(|err| panic_on_parse_error("test_ubx_packet_send", &src_code, &err));

    run_compare_test(
        tokens,
        quote! {
            #[doc = "Some comment"]
            pub struct Test;

            impl UbxPacketMeta for Test {
                const CLASS: u8 = 1u8;
                const ID: u8 = 2u8;
                const FIXED_PAYLOAD_LEN: Option<u16> = Some(9u16);
                const MAX_PAYLOAD_LEN: u16 = 9u16;
            }

            #[doc = "Some comment"]
            #[doc = "Struct that is used as \"builder\" for packet"]
            #[derive(Default)]
            pub struct TestBuilder {
                #[doc = ""]
                pub itow: u32,
                #[doc = "this is lat"]
                pub lat_degrees: f64,
                #[doc = "this is a"]
                pub a: u8,
            }
            impl TestBuilder {
                pub const PACKET_LEN: usize = 17usize;

                #[inline]
                pub fn into_packet_bytes(self) -> [u8; Self::PACKET_LEN] {
                    let mut ret = [0u8; Self::PACKET_LEN];
                    ret[0] = SYNC_CHAR_1;
                    ret[1] = SYNC_CHAR_2;
                    ret[2] = Test::CLASS;
                    ret[3] = Test::ID;
                    let pack_len_bytes = 9u16.to_le_bytes();
                    ret[4] = pack_len_bytes[0];
                    ret[5] = pack_len_bytes[1];
                    let bytes = self.itow.to_le_bytes();
                    ret[6usize] = bytes[0usize];
                    ret[7usize] = bytes[1usize];
                    ret[8usize] = bytes[2usize];
                    ret[9usize] = bytes[3usize];
                    let bytes = ScaleBack::<f64>(1. / 1e-7)
                        .as_i32(self.lat_degrees)
                        .to_le_bytes();
                    ret[10usize] = bytes[0usize];
                    ret[11usize] = bytes[1usize];
                    ret[12usize] = bytes[2usize];
                    ret[13usize] = bytes[3usize];
                    let bytes = self.a.to_le_bytes();
                    ret[14usize] = bytes[0usize];
                    let (ck_a, ck_b) = ubx_checksum(&ret[2..17usize - 2]);
                    ret[17usize - 2] = ck_a;
                    ret[17usize - 1] = ck_b;
                    ret
                }
            }
            impl From<TestBuilder> for [u8; 17usize] {
                fn from(x: TestBuilder) -> Self {
                    x.into_packet_bytes()
                }
            }
            impl UbxPacketCreator for TestBuilder {
                #[inline]
                fn create_packet<T: MemWriter>(self, out: &mut T) -> Result<(), MemWriterError<T::Error>> {
                    out.reserve_allocate(17usize)?;
                    let len_bytes = 9u16.to_le_bytes();
                    let header = [
                        SYNC_CHAR_1,
                        SYNC_CHAR_2,
                        Test::CLASS,
                        Test::ID,
                        len_bytes[0],
                        len_bytes[1],
                    ];
                    out.write(&header)?;
                    let mut checksum_calc = UbxChecksumCalc::default();
                    checksum_calc.update(&header[2..]);
                    let bytes = self.itow.to_le_bytes();
                    out.write(&bytes)?;
                    checksum_calc.update(&bytes);
                    let bytes = ScaleBack::<f64>(1. / 1e-7)
                        .as_i32(self.lat_degrees)
                        .to_le_bytes();
                    out.write(&bytes)?;
                    checksum_calc.update(&bytes);
                    let bytes = self.a.to_le_bytes();
                    out.write(&bytes)?;
                    checksum_calc.update(&bytes);
                    let (ck_a, ck_b) = checksum_calc.result();
                    out.write(&[ck_a, ck_b])?;
                    Ok(())
                }
            }
        },
    );
}

#[test]
fn test_upgrade_enum() {
    let src_code = quote! {
        #[doc = "GPS fix Type"]
        #[ubx_extend]
        #[ubx(from, rest_reserved)]
        #[repr(u8)]
        #[derive(Debug, Copy, Clone)]
        enum GpsFix {
            NoFix = 0,
            DeadReckoningOnly = 1,
            Fix2D = 2,
            Fix3D = 3,
            GPSPlusDeadReckoning = 4,
            TimeOnlyFix = 5,
        }
    };
    let src_code = src_code.to_string();

    let code: syn::ItemEnum = syn::parse_str(&src_code)
        .unwrap_or_else(|err| panic_on_parse_error("test_upgrade_enum", &src_code, &err));
    let tokens = extend_enum(code.ident, code.attrs, code.variants)
        .unwrap_or_else(|err| panic_on_parse_error("test_upgrade_enum", &src_code, &err));

    let mut reserved_fields = Vec::with_capacity(256);
    let mut rev_reserved_fields = Vec::with_capacity(256);
    for i in 6..=255 {
        let val = i as u8;
        let ident = quote::format_ident!("Reserved{}", val);
        reserved_fields.push(quote! { #ident = #val });
        rev_reserved_fields.push(quote! { #val => GpsFix::#ident });
    }

    run_compare_test(
        tokens,
        quote! {
            #[doc = "GPS fix Type"]
            #[repr(u8)]
            #[derive(Debug, Copy, Clone)]
            pub enum GpsFix {
                NoFix = 0u8,
                DeadReckoningOnly = 1u8,
                Fix2D = 2u8,
                Fix3D = 3u8,
                GPSPlusDeadReckoning = 4u8,
                TimeOnlyFix = 5u8,
                #(#reserved_fields),*
            }
            impl GpsFix {
                fn from(x: u8) -> Self {
                    match x {
                        0u8 => GpsFix::NoFix,
                        1u8 => GpsFix::DeadReckoningOnly,
                        2u8 => GpsFix::Fix2D,
                        3u8 => GpsFix::Fix3D,
                        4u8 => GpsFix::GPSPlusDeadReckoning,
                        5u8 => GpsFix::TimeOnlyFix,
                        #(#rev_reserved_fields),*
                    }
                }
            }
        },
    );
}

#[test]
fn test_define_recv_packets() {
    let src_code = quote! {
        enum PacketRef {
            _ = UnknownPacketRef,
            Pack1,
            Pack2
        }
    };
    let src_code = src_code.to_string();
    let tokens: TokenStream = syn::parse_str(&src_code)
        .unwrap_or_else(|err| panic_on_parse_error("test_define_recv_packets", &src_code, &err));
    let output = do_define_recv_packets(tokens)
        .unwrap_or_else(|err| panic_on_parse_error("test_define_recv_packets", &src_code, &err));
    run_compare_test(
        output,
        quote! {
            #[doc = "All possible packets enum"]
            pub enum PacketRef<'a> {
                Pack1(Pack1Ref<'a>),
                Pack2(Pack2Ref<'a>),
                Unknown(UnknownPacketRef<'a>)
            }

            impl<'a> PacketRef<'a> {
                pub fn class_and_msg_id(&self) -> (u8, u8) {
                    match *self {
                        PacketRef::Pack1(_) => (Pack1::CLASS, Pack1::ID),
                        PacketRef::Pack2(_) => (Pack2::CLASS, Pack2::ID),
                        PacketRef::Unknown(ref pack) => (pack.class, pack.msg_id),
                    }
                }
            }

            pub(crate) fn match_packet(
                class: u8,
                msg_id: u8,
                payload: &[u8],
            ) -> Result<PacketRef, ParserError> {
                match (class, msg_id) {
                    (Pack1::CLASS, Pack1::ID) if <Pack1Ref>::validate(payload).is_ok() => {
                        Ok(PacketRef::Pack1(Pack1Ref(payload)))
                    }
                    (Pack2::CLASS, Pack2::ID) if <Pack2Ref>::validate(payload).is_ok() => {
                        Ok(PacketRef::Pack2(Pack2Ref(payload)))
                    }
                    _ => Ok(PacketRef::Unknown(UnknownPacketRef {
                        payload,
                        class,
                        msg_id,
                    })),
                }
            }

            const fn max_u16(a: u16, b: u16) -> u16 {
                [a, b][(a < b) as usize]
            }
            pub(crate) const MAX_PAYLOAD_LEN: u16 =
                    max_u16(Pack2::MAX_PAYLOAD_LEN, max_u16(Pack1::MAX_PAYLOAD_LEN, 0u16));
        },
    );
}

#[test]
fn test_extend_bitflags() {
    let src_code = quote! {
        #[ubx_extend_bitflags]
        #[ubx(from, rest_reserved)]
        bitflags! {
            #[doc = "Navigation Status Flags"]
            pub struct Test: u8 {
                #[doc = "position and velocity valid and within DOP and ACC Masks"]
                const F1 = 1;
                #[doc = "DGPS used"]
                const F2 = 2;
                #[doc = "Week Number valid"]
                const F3 = 4;
                #[doc = "Time of Week valid"]
                const F4 = 8;
            }
        }
    };
    let src_code = src_code.to_string();

    let mac: syn::ItemMacro = syn::parse_str(&src_code)
        .unwrap_or_else(|err| panic_on_parse_error("test_extend_bitflags", &src_code, &err));
    let tokens = extend_bitflags(mac)
        .unwrap_or_else(|err| panic_on_parse_error("test_extend_bitflags", &src_code, &err));
    run_compare_test(
        tokens,
        quote! {
            bitflags! {
                #[doc = "Navigation Status Flags"]
                pub struct Test: u8 {
                    #[doc = "position and velocity valid and within DOP and ACC Masks"]
                    const F1 = (1 as u8);
                    #[doc = "DGPS used"]
                    const F2 = ((1 as u8) << 1u32);
                    #[doc = "Week Number valid"]
                    const F3 = ((1 as u8) << 2u32);
                    #[doc = "Time of Week valid"]
                    const F4 = ((1 as u8) << 3u32);
                    const RESERVED4 = ((1 as u8) << 4u32);
                    const RESERVED5 = ((1 as u8) << 5u32);
                    const RESERVED6 = ((1 as u8) << 6u32);
                    const RESERVED7 = ((1 as u8) << 7u32);
                }
            }
            impl Test {
                const fn from(x: u8) -> Self {
                    Self::from_bits_truncate(x)
                }
            }
        },
    );
}

fn run_compare_test(output: TokenStream, expect_output: TokenStream) {
    let output = output.to_string();
    let output = String::from_utf8(rustfmt_cnt(output.into_bytes()).unwrap()).unwrap();
    let expect_output = expect_output.to_string();
    let expect_output =
        String::from_utf8(rustfmt_cnt(expect_output.into_bytes()).unwrap()).unwrap();

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

fn rustfmt_cnt(source: Vec<u8>) -> io::Result<Vec<u8>> {
    let rustfmt =
        which("rustfmt").map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

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

fn panic_on_parse_error(name: &str, src_cnt: &str, err: &Error) -> ! {
    use std::fmt::Write;

    let span = err.span();
    let start = span.start();
    let end = span.end();

    let mut code_problem = String::new();
    let nlines = end.line - start.line + 1;
    for (i, line) in src_cnt
        .lines()
        .skip(start.line - 1)
        .take(nlines)
        .enumerate()
    {
        code_problem.push_str(&line);
        code_problem.push('\n');
        if i == 0 && start.column > 0 {
            write!(&mut code_problem, "{:1$}", ' ', start.column).expect("write to String failed");
        }
        let code_problem_len = if i == 0 {
            if i == nlines - 1 {
                end.column - start.column
            } else {
                line.len() - start.column - 1
            }
        } else if i != nlines - 1 {
            line.len()
        } else {
            end.column
        };
        writeln!(&mut code_problem, "{:^^1$}", '^', code_problem_len).expect("Not enought memory");
        if i == end.line {
            break;
        }
    }

    panic!(
        "parsing of {name} failed\nerror: {err}\n{code_problem}\nAt {name}:{line_s}:{col_s}",
        name = name,
        err = err,
        code_problem = code_problem,
        line_s = start.line,
        col_s = start.column,
    );
}
