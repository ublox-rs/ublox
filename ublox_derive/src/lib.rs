mod error;
mod file_cache;
mod input;
mod output;
mod types;

pub use error::panic_on_parse_error;
use quote::ToTokens;
use std::{collections::HashMap, path::Path};
use types::HowCodeForPackage;

/// process `src` and save result of macro expansion to `dst`
///
/// # Panics
/// Panics on error
pub fn expand_ubx_packets_code_in_file<S, D>(src: S, dst: D)
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let src_cnt = std::fs::read_to_string(src.as_ref()).unwrap_or_else(|err| {
        panic!(
            "Error during read for file {}: {}",
            src.as_ref().display(),
            err
        )
    });
    let data = match expand_ubx_packets_code_in_str(&src_cnt) {
        Ok(x) => x,
        Err(ref err) => panic_on_parse_error((src.as_ref(), &src_cnt), err),
    };
    let mut file = file_cache::FileWriteCache::new(dst.as_ref());
    file.replace_content(data.into_bytes());
    file.update_file_if_necessary().unwrap_or_else(|err| {
        panic!(
            "Error during write to file {}: {}",
            dst.as_ref().display(),
            err
        );
    });
}

pub fn expand_ubx_packets_code_in_str(src_cnt: &str) -> syn::Result<String> {
    let mut ret = String::new();
    let syn_file = syn::parse_file(src_cnt)?;
    let mut packets = Vec::with_capacity(100);
    let mut ubx_types = HashMap::new();
    for item in syn_file.items {
        match item {
            syn::Item::Struct(s) => {
                let mut send = false;
                let mut recv = false;
                for a in &s.attrs {
                    if !send {
                        send = a.path.is_ident("ubx_packet_send");
                    }
                    if !recv {
                        recv = a.path.is_ident("ubx_packet_recv");
                    }
                }
                if send || recv {
                    let mode = if send && recv {
                        HowCodeForPackage::SendRecv
                    } else if send {
                        HowCodeForPackage::SendOnly
                    } else if recv {
                        HowCodeForPackage::RecvOnly
                    } else {
                        unreachable!();
                    };
                    packets.push((s, mode));
                } else {
                    ret.push_str(&s.into_token_stream().to_string());
                }
            }
            syn::Item::Enum(e) => {
                if e.attrs.iter().any(|x| x.path.is_ident("ubx_type")) {
                    let en = input::parse_ubx_enum_type(e)?;
                    let code = output::generate_code_for_ubx_enum(&en);
                    ubx_types.insert(en.name.to_string(), en);
                    ret.push_str(&code);
                } else {
                    ret.push_str(&e.into_token_stream().to_string());
                }
            }
            _ => {
                ret.push_str(&item.into_token_stream().to_string());
            }
        }
    }

    let mut all_packs = Vec::with_capacity(packets.len());

    for (pack_desc, mode) in packets {
        let pack = input::parse_packet_description(pack_desc)?;
        let code = output::generate_code_for_packet(&pack, &ubx_types, mode);
        all_packs.push((pack, mode));
        ret.push_str(&code);
    }

    let code = output::generate_code_for_packet_parser(&all_packs, &ubx_types);
    ret.push_str(&code);

    Ok(ret)
}
