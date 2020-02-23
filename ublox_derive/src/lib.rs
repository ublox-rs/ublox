extern crate proc_macro;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, Data, Fields, FieldsNamed, Ident, Attribute};
use syn::parse::Parser;
use syn::spanned::Spanned;
use quote::{quote, quote_spanned, format_ident};
use std::convert::TryInto;
use inflector::Inflector;
use proc_macro_error::{abort, proc_macro_error};
use syn::parse::Parse;
use itertools::Itertools;

#[derive(Clone, Debug)]
struct Bitrange {
    lsb: usize,
    msb: usize,
    ident: Ident,
    enum_type: Option<Ident>,
    field_type: syn::TypePath,
}

#[derive(Clone, Debug)]
struct Bitfield {
    num_bits: usize,
    ranges: Vec<Bitrange>,
}

impl Bitfield {
    fn new(num_bits: usize) -> Bitfield {
        Bitfield {
            num_bits: num_bits,
            ranges: vec!(),
        }
    }

    fn add_range(&mut self, range: Bitrange) {
        self.ranges.push(range);
    }
}

#[derive(Debug)]
enum Member {
    Bitfield(Bitfield),
    Primitive(syn::Field),
}

enum UbxAttribute {
    UbxBitRange((usize, usize)),
    UbxBitField(usize),
    UbxEnum(Ident),
}

fn parse_attribute(attr: &Attribute) -> Option<UbxAttribute> {
    //println!("{:#?}", attr);
    let parser = syn::punctuated::Punctuated::<TokenStream, syn::token::Comma>::parse_separated_nonempty;

    let name = attr.path.get_ident().unwrap();
    let arguments = attr.parse_args_with(parser).unwrap();

    match name.to_string().as_str() {
        "ubx_bitfield" => {
            if arguments.len() != 1 {
                panic!("Incorrect number of arguments to ubx_bitfield");
            }
            let arg = syn::parse2(arguments[0].clone()).unwrap();
            match &arg {
                syn::Lit::Int(litint) => {
                    Some(UbxAttribute::UbxBitField(litint.base10_parse().unwrap()))
                }
                _ => { abort!(&arguments[0], "Only int literals allowed!"); }
            }
        }
        "ubx_bitrange" => {
            if arguments.len() != 1 {
                panic!("Incorrect number of arguments to ubx_bitrange");
            }
            let parser = syn::punctuated::Punctuated::<syn::Lit, syn::token::Colon>::parse_separated_nonempty;
            let bits = parser.parse2(arguments[0].clone()).unwrap();
            if bits.len() != 2 {
                panic!("Bit slice may only contain 2 elements in ubx_bitrange");
            }
            let msb: usize = match &bits[0] {
                syn::Lit::Int(litint) => {
                    litint.base10_parse().unwrap()
                }
                _ => { abort!(&bits[0], "Only int literals allowed!"); }
            };
            let lsb: usize = match &bits[1] {
                syn::Lit::Int(litint) => {
                    litint.base10_parse().unwrap()
                }
                _ => { abort!(&bits[1], "Only int literals allowed!"); }
            };
            Some(UbxAttribute::UbxBitRange((msb, lsb)))
        }
        "ubx_enum" => {
            if arguments.len() != 1 {
                panic!("Incorrect number of arguments to ubx_enum");
            }
            Some(UbxAttribute::UbxEnum(match arguments[0].clone().into_iter().next().unwrap() {
                proc_macro2::TokenTree::Ident(ident) => ident,
                _ => { abort!(arguments[0], "Must specify an identifier for ubx_enum"); }
            }))
        }
        _ => { None }
    }
}

fn find_struct_segments(fields: &FieldsNamed) -> Vec<Member> {
    let mut segments = vec!();
    let mut current_bitfield: Option<Bitfield> = None;
    for field in fields.named.iter() {
        let tags: Vec<_> = field.attrs.iter().map(parse_attribute).collect();
        let bitfield: Vec<_> = tags.iter().filter_map(|x| {
            match x {
                Some(UbxAttribute::UbxBitField(size)) => Some(size),
                _ => None,
            }
        }).collect();
        let has_bitfield = bitfield.len() > 0;

        let bitrange: Vec<_> = tags.iter().filter_map(|x| {
            match x {
                Some(UbxAttribute::UbxBitRange((msb, lsb))) => Some((msb, lsb)),
                _ => None,
            }
        }).collect();
        let has_bitrange = bitrange.len() > 0;

        let enum_type: Vec<_> = tags.iter().filter_map(|x| {
            match x {
                Some(UbxAttribute::UbxEnum(e)) => Some(e),
                _ => None,
            }
        }).collect();
        let enum_type = if enum_type.len() > 0 {
            Some(enum_type[0].clone())
        } else {
            None
        };

        if has_bitfield {
            if let Some(field) = current_bitfield {
                segments.push(Member::Bitfield(field));
            }
            current_bitfield = Some(Bitfield::new(*bitfield[0]));
        }

        if has_bitrange {
            let (msb, lsb) = bitrange[0];
            let bitrange = Bitrange{
                lsb: *lsb,
                msb: *msb,
                ident: field.ident.as_ref().unwrap().clone(),
                enum_type: enum_type,
                field_type: match &field.ty {
                    syn::Type::Path(path) => {
                        path.clone()
                    }
                    _ => {
                        abort!(field, "Only path types allowed for bitmap ranges");
                    }
                }
            };
            match &mut current_bitfield {
                Some(bitfield) => {
                    bitfield.add_range(bitrange);
                }
                None => {
                    abort!(field, "Must have an active bitfield to specify a bitrange!");
                }
            }
        } else {
            if let Some(bitfield) = current_bitfield {
                segments.push(Member::Bitfield(bitfield));
            }
            current_bitfield = None;

            segments.push(Member::Primitive(field.clone()));
        }
    }
    if let Some(field) = current_bitfield {
        segments.push(Member::Bitfield(field));
    }
    //println!("{:#?}", segments);
    segments
}

struct Accessor {
    getter: TokenStream,
    setter: TokenStream,
    trait_getter: TokenStream,
    trait_setter: TokenStream,
}

//fn build_bitrange_accessors(offset: &TokenStream, bitfield: &Bitfield, bitrange: &Bitrange) -> (TokenStream, TokenStream, TokenStream) {
fn build_bitrange_accessors(offset: &TokenStream, bitfield: &Bitfield, bitrange: &Bitrange) -> Accessor {
    let underlying_fn_name = format_ident!("get_{}_underlying", bitrange.ident);
    let underlying_set_fn_name = format_ident!("set_{}_underlying", bitrange.ident);
    let underlying_type = format_ident!("u{}", bitfield.num_bits);
    let getter_fn_name = format_ident!("get_{}", bitrange.ident);
    let setter_fn_name = format_ident!("set_{}", bitrange.ident);
    let span = bitrange.ident.span();
    let return_type = if let Some(ident) = &bitrange.enum_type {
        let field_type = &ident;
        parse_quote!{ Option<#field_type> }
    } else {
        bitrange.field_type.clone()
    };
    let field_type = if let Some(ident) = &bitrange.enum_type {
        parse_quote!{ #ident }
    } else {
        bitrange.field_type.clone()
    };
    let msb = bitrange.msb;
    let lsb = bitrange.lsb;
    let shifter_fn_name = format_ident!("shift_{}", bitrange.ident);
    let type_cvt_name = format_ident!("type_cvt_{}", bitrange.ident);
    let type_uncvt_name = format_ident!("type_uncvt_{}", bitrange.ident);
    let type_cvt = if let Some(enumtype) = &bitrange.enum_type {
        let fromname = format_ident!("from_{}", underlying_type);
        quote_spanned! {
            span =>
                #enumtype::#fromname(value)
        }
    } else if return_type.path.get_ident().unwrap().to_string() == "bool" {
        quote_spanned! {
            span =>
                value != 0
        }
    } else {
        quote_spanned! {
            span =>
                value.try_into().unwrap()
        }
    };
    let type_uncvt = if let Some(enumtype) = &bitrange.enum_type {
        let toname = format_ident!("to_{}", underlying_type);
        quote_spanned! {
            span =>
                value.#toname().unwrap()
        }
    } else if return_type.path.get_ident().unwrap().to_string() == "bool" {
        quote_spanned! {
            span =>
                if value { 1 } else { 0 }
        }
    } else {
        quote_spanned! {
            span =>
                value.try_into().unwrap()
        }
    };
    Accessor {
        getter: quote_spanned! {
            span =>
            fn #underlying_fn_name(&self) -> #underlying_type {
                #underlying_type::from_le_bytes(self.data[#offset..#offset + std::mem::size_of::<#underlying_type>()].try_into().unwrap())
            }

            fn #type_cvt_name(value: #underlying_type) -> #return_type {
                #type_cvt
            }

            fn #shifter_fn_name(&self) -> #underlying_type {
                let underlying = self.#underlying_fn_name();
                (underlying >> #lsb) & ((1 << (#msb - #lsb + 1)) - 1)
            }

            fn #getter_fn_name(&self) -> #return_type {
                Self::#type_cvt_name(self.#shifter_fn_name())
            }
        },
        trait_getter: quote_spanned! {
            span =>
            fn #getter_fn_name(&self) -> #return_type;
        },
        setter: quote_spanned! {
            span =>
            fn #underlying_set_fn_name(&mut self, value: #underlying_type) {
                let bytes = value.to_le_bytes();
                self.data[#offset..#offset + std::mem::size_of::<#underlying_type>()].clone_from_slice(&bytes);
            }

            fn #type_uncvt_name(value: #field_type) -> #underlying_type {
                #type_uncvt
            }

            fn #setter_fn_name(&mut self, value: #field_type) {
                let original = self.#underlying_fn_name();
                let new_field_value = Self::#type_uncvt_name(value);
                let mask = ((1 << (#msb - #lsb + 1)) - 1) << #lsb;
                let newval = (original & !mask) | (new_field_value << #lsb);
                //println!("{} {} {} {}", original, new_field_value, mask, newval);
                self.#underlying_set_fn_name(newval);
            }
        },
        trait_setter: quote_spanned! {
            span =>
            fn #setter_fn_name(&mut self, value: #field_type);
        },
    }
}

fn process_struct(struct_name: &Ident, fields: &FieldsNamed) -> TokenStream {
    let segments = find_struct_segments(fields);

    let fs: Vec<_> = fields.named.iter().map(|f| {
        for attr in f.attrs.iter() {
            parse_attribute(&attr);
        }
    }).collect();

    let sizes: Vec<_> = segments.iter().map(|f| {
        match f {
            Member::Bitfield(bitfield) => {
                let nbits = bitfield.num_bits;
                quote_spanned! {
                    bitfield.ranges[0].ident.span() =>
                        (#nbits / 8)
                }
            }
            Member::Primitive(f) => {
                let ftype = &f.ty;
                quote_spanned! {
                    f.span() =>
                        std::mem::size_of::<#ftype>()
                }
            }
        }
    }).collect();

    let offsets: Vec<_> = sizes.iter().scan(quote! { 0 }, |state, size| {
        let orig_state = state.clone();
        *state = quote! {
            #state + #size
        };
        Some(orig_state)
    }).collect();

    let accessors: Vec<Accessor> = segments.iter().zip(offsets).map(|(f, o)| {
        match f {
            Member::Bitfield(bitfield) => {
                let accessors: Vec<Accessor> = bitfield.ranges.iter().map(|range| {
                //bitfield.ranges.iter().map(|range| {
                    build_bitrange_accessors(&o, &bitfield, &range)
                }).collect();
                accessors
                /*((quote_spanned! {
                    bitfield.ranges[0].ident.span() =>
                        #(#getters)*
                }, quote_spanned! {
                    bitfield.ranges[0].ident.span() =>
                        #(#trait_getters)*
                }),
                 quote_spanned! {
                     bitfield.ranges[0].ident.span() =>
                         #(#setters)*
                 })*/
            }
            Member::Primitive(f) => {
                let enum_attrs: Vec<_> = f.attrs.iter().filter_map(|attr| {
                    parse_attribute(attr)
                }).filter_map(|attr| {
                    match attr {
                        UbxAttribute::UbxEnum(ident) => Some(ident),
                        _ => None,
                    }
                }).collect();

                let membername = f.ident.as_ref().unwrap();
                let get_fname = format_ident!("get_{}", membername);
                let set_fname = format_ident!("set_{}", membername);
                let ftype = &f.ty;
                if enum_attrs.len() > 0 {
                    let enumtype = &enum_attrs[0];
                    let fromname = format_ident!("from_{}", match ftype {
                        syn::Type::Path(path) => { path.path.get_ident().unwrap() }
                        _ => { abort!(f, "Must specify a primitive int field type"); }
                    });
                    let enum_cvt = format_ident!("to_{}", match ftype {
                        syn::Type::Path(path) => { path.path.get_ident().unwrap() }
                        _ => { abort!(f, "Must specify a primitive int field type"); }
                    });
                    vec![Accessor {
                        getter: quote_spanned! {
                            f.span() =>
                            fn #get_fname(&self) -> Option<#enumtype> {
                                let x = #ftype::from_le_bytes(self.data[#o..#o + std::mem::size_of::<#ftype>()].try_into().unwrap());
                                #enumtype::#fromname(x)
                            }
                        },
                        trait_getter: quote_spanned! {
                            f.span() =>
                            fn #get_fname(&self) -> Option<#enumtype>;
                        },
                        setter: quote_spanned! {
                            f.span() =>
                            fn #set_fname(&mut self, value: #enumtype) {
                                let value = value.#enum_cvt();
                                let bytes = value.to_le_bytes();
                                self.data[#o..#o + std::mem::size_of::<#ftype>()].clone_from_slice(&bytes);
                            }
                        },
                        trait_setter: quote_spanned! {
                            f.span() =>
                            fn #set_fname(&mut self, value: #enumtype);
                        }
                    }]
                } else {
                    vec![Accessor {
                        getter: quote_spanned! {
                            f.span() =>
                            fn #get_fname(&self) -> #ftype {
                                #ftype::from_le_bytes(self.data[#o..#o + std::mem::size_of::<#ftype>()].try_into().unwrap())
                            }
                        },
                        trait_getter: quote_spanned! {
                            f.span() =>
                            fn #get_fname(&self) -> #ftype;
                        },
                        setter: quote_spanned! {
                            f.span() =>
                            fn #set_fname(&mut self, value: #ftype) {
                                let bytes = value.to_le_bytes();
                                self.data[#o..#o + std::mem::size_of::<#ftype>()].clone_from_slice(&bytes);
                            }
                        },
                        trait_setter: quote_spanned! {
                            f.span() =>
                            fn #set_fname(&mut self, value: #ftype);
                        }
                    }]
                }
            }
        }
    }).flatten().collect();
    //let (getters, trait_getters) = getters.iter().unzip();

    let mut getters = vec!();
    let mut trait_getters = vec!();
    let mut setters = vec!();
    let mut trait_setters = vec!();
    for accessor in accessors.iter() {
        getters.push(&accessor.getter);
        trait_getters.push(&accessor.trait_getter);
        setters.push(&accessor.setter);
        trait_setters.push(&accessor.trait_setter);
    }

    let getter_trait_name = format_ident!("{}Getter", struct_name);
    let setter_trait_name = format_ident!("{}Setter", struct_name);
    let ref_struct_name = format_ident!("{}Ref", struct_name);
    let mut_ref_struct_name = format_ident!("{}MutRef", struct_name);

    quote! {
        pub trait #getter_trait_name {
            #(#trait_getters)*
        }

        pub trait #setter_trait_name {
            #(#trait_setters)*
        }

        struct #ref_struct_name<'a> {
            data: &'a [u8; 0 #(+ #sizes)*],
        }

        impl<'a> #ref_struct_name<'a> {
            pub fn new(data: &'a [u8; 0 #(+ #sizes)*]) -> #ref_struct_name {
                #ref_struct_name {
                    data: data,
                }
            }
        }

        impl<'a> #getter_trait_name for #ref_struct_name<'a> {
            #(#getters)*
        }

        struct #mut_ref_struct_name<'a> {
            data: &'a mut [u8; 0 #(+ #sizes)*],
        }

        impl<'a> #mut_ref_struct_name<'a> {
            pub fn new(data: &'a mut [u8; 0 #(+ #sizes)*]) -> #mut_ref_struct_name {
                #mut_ref_struct_name {
                    data: data,
                }
            }
        }

        impl<'a> #getter_trait_name for #mut_ref_struct_name<'a> {
            #(#getters)*
        }

        impl<'a> #setter_trait_name for #mut_ref_struct_name<'a> {
            #(#setters)*
        }

        // TODO: Implement a more logical Debug trait
        // TODO: Make the pub-ness optional
        #[derive(Debug, PartialEq)]
        pub struct #struct_name {
            data: [u8; 0 #(+ #sizes)*],
        }

        impl #struct_name {
            pub fn new(data: [u8; 0 #(+ #sizes)*]) -> #struct_name {
                #struct_name {
                    data: data,
                }
            }
        }

        impl #getter_trait_name for #struct_name {
            #(#getters)*
        }

        impl #setter_trait_name for #struct_name {
            #(#setters)*
        }
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn ubx_packet(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    proc_macro::TokenStream::from(process_struct(&input.ident, fields))
                }
                Fields::Unnamed(ref fields) => {
                    unimplemented!();
                }
                Fields::Unit => {
                    unimplemented!();
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    }
}
