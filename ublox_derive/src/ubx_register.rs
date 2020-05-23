//use crate::types::BitFlagsMacro;
use crate::types::{
    PackDesc, PackField, PacketFlag, PayloadLen, RecvPackets, UbxEnumRestHandling, UbxExtendEnum,
    UbxTypeFromFn, UbxTypeIntoFn,
};
use crate::input::UbxBitfield;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::{collections::HashSet, convert::TryFrom};
use syn::{parse_quote, Ident, Type};
use syn::{Attribute, Fields};

pub fn generate_ubx_register(
    underlying_type: Ident,
    struct_name: Ident,
    attrs: Vec<Attribute>,
    fields: Fields,
) -> TokenStream {
    println!("Generating ubx_register!");
    println!("{:?}", struct_name);
    println!("{:?}", attrs);
    let mut getters = vec!();
    let mut setters = vec!();
    let mut with = vec!();
    for f in fields.iter() {
        //println!("{:?}", f);
        let mut bits = None;//UbxBitfield{ hi: 0, lo: 0 };
        for attr in f.attrs.iter() {
            //println!("{:?} {:?}", f.ident, attr.path.get_ident());
            let ident = match attr.path.get_ident() {
                Some(x) => { x },
                None => {
                    return quote!{ compile_error!("Only attributes that are a single identifier are supported!"); };
                }
            };
            if *ident == "ubx_field" {
                //println!("Found field!");
                let flags: UbxBitfield = attr.parse_args().expect("foobar");
                //println!("{:?}", flags);
                //break flags;
                bits = Some(flags);
            } else if *ident == "doc" {
                println!("Docs: {:?}", attr);
            }
        };
        //let bits = bits.expect("Could not find a ubx_field attribute!");
        let bits = match bits {
            Some(x) => { x }
            None => {
                return quote!{ compile_error!("Could not find a ubx_field attribute!"); };
            }
        };
        /*println!("{:?}", bits);
        println!("{:?}", f.ident);
        println!("{:?}", f.ty);*/
        let getter_name = format_ident!("get_{}", f.ident.as_ref().expect("Struct field must have a name"));
        let setter_name = format_ident!("set_{}", f.ident.as_ref().expect("Struct field must have a name"));
        let with_name = format_ident!("with_{}", f.ident.as_ref().expect("Struct field must have a name"));
        let field_type = &f.ty;
        let lo = bits.lo;
        let mask: usize = (1 << (bits.hi - bits.lo + 1)) - 1;
        let is_bool = {
            if let syn::Type::Path(path) = field_type {
                format!("{}", path.path.get_ident().unwrap()) == "bool"
            } else {
                false
            }
        };
        // TODO: We shouldn't cast to usize
        // TODO: We should find a way to merge these
        if is_bool {
            // TODO: Check that hi == lo
            getters.push(quote!{
                pub fn #getter_name(&self) -> #field_type {
                    ((self.0 as usize >> #lo) & #mask) != 0
                }
            });
            setters.push(quote!{
                pub fn #setter_name(&mut self, value: #field_type) {
                    // @TODO: Actually implement this
                    self.0 |= 1;
                }
            });
            with.push(quote!{
                pub fn #with_name(mut self, value: #field_type) -> Self {
                    self.#setter_name(value);
                    self
                }
            });
        } else {
            getters.push(quote!{
                pub fn #getter_name(&self) -> #field_type {
                    ((self.0 as usize >> #lo) & #mask).try_into().unwrap()
                }
            });
            setters.push(quote!{
                pub fn #setter_name(&mut self, value: #field_type) {
                    // @TODO: Actually implement this
                    self.0 |= 1;
                }
            });
            with.push(quote!{
                pub fn #with_name(mut self, value: #field_type) -> Self {
                    self.#setter_name(value);
                    self
                }
            });
        }
    }
    quote!{
        //use std::convert::TryInto;

        //struct #struct_name<'a>(&'a [u8; 1]);
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub struct #struct_name(#underlying_type);

        impl #struct_name {
            /*pub fn get_field1(&self) -> bool {
                (self.0 & 0x1) != 0
            }

            pub fn get_field2(&self) -> u8 {
                (self.0 >> 1) & 0x7F
            }*/

            #(#getters)*
            #(#with)*
            #(#setters)*

            const fn from(x: #underlying_type) -> Self {
                #struct_name(x)
            }

            fn into_raw(self) -> #underlying_type {
                self.0
            }

            // TODO: Evaluate callsites to determine if this is needed
            fn from_bits_truncate(bits: #underlying_type) -> #struct_name {
                Self(bits)
            }
        }

        // TODO: This may not make sense for every packet
        impl std::default::Default for #struct_name {
            fn default() -> Self { Self(0) }
        }
    }
}
