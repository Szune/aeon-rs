extern crate proc_macro;
extern crate syn;
//#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, Lit, Meta, MetaList, MetaNameValue,
    NestedMeta, Type,
};

#[proc_macro_derive(Deserialize, attributes(ignore))]
pub fn aeon_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    assert!(
        input.generics.type_params().next().is_none(),
        "Deserialize is not implemented for generic types"
    );

    let readfunc = read_fields(&input.data);
    let readmacro = read_fields_macro(&input.data);
    let macro_ident: Ident = Ident::new(
        &quote!(read_#name)
            .to_string()
            .as_str()
            .replace("\"", "")
            .replace(" ", ""),
        name.span(),
    );

    let expanded = quote! {
        macro_rules! #macro_ident {
            ($buf: expr) => (
                #name {
                    #readmacro
                }
            );
        }
        impl Deserialize for #name {
            fn from_packet(packet: Vec<u8>) -> Self {
                let mut buf_iter = &mut packet.into_iter();
                skip_header!(&mut buf_iter);
                Self {
                    #readfunc
                }
            }
        }

    };
    //println!("{}",expanded);

    TokenStream::from(expanded)
}

fn read_fields(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields
                    .named
                    .iter()
                    .filter(|f| !get_name_attribute("ignore".into(), &f.attrs))
                    .map(|f| {
                        let name = &f.ident;
                        let macro_call = type_name_to_ident("read_".into(), &f.ty, f.span());
                        quote_spanned! {f.span() =>
                            #name : #macro_call!(&mut buf_iter),
                        }
                    });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn read_fields_macro(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields
                    .named
                    .iter()
                    .filter(|f| !get_name_attribute("ignore".into(), &f.attrs))
                    .map(|f| {
                        let name = &f.ident;
                        let macro_call = type_name_to_ident("read_".into(), &f.ty, f.span());
                        quote_spanned! {f.span() =>
                            #name : #macro_call!($buf),
                        }
                    });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

#[proc_macro_derive(Serialize, attributes(ignore))]
pub fn aeon_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    assert!(
        input.generics.type_params().next().is_none(),
        "Serialize is not implemented for generic types"
    );
    let packet_id = get_attribute::<u8>("packet_id".into(), input.attrs);
    let name = input.ident;

    let writefunc = write_fields(&input.data);
    let writemacro = write_fields_macro(&input.data);

    let macro_ident: Ident = Ident::new(
        &quote!(write_#name)
            .to_string()
            .as_str()
            .replace("\"", "")
            .replace(" ", ""),
        name.span(),
    );
    let arr_macro_ident: Ident = Ident::new(
        &quote!(write_arr_#name)
            .to_string()
            .as_str()
            .replace("\"", "")
            .replace(" ", ""),
        name.span(),
    );
    let expanded = quote! {
        macro_rules! #macro_ident {
            ($buf: expr, $val: expr) => (
                #writemacro
            );
        }
        macro_rules! #arr_macro_ident {
            ($buf: expr, $val: expr) => (
                for i in &$val {
                    #macro_ident!($buf, i);
                }
            );
        }
        impl Serialize for #name {
            fn to_packet(&self) -> Vec<u8> {
                let mut packet = Vec::<u8>::new();
                write_u16!(packet, 0);
                write_u8!(packet, #packet_id);
                #writefunc
                write_len!(packet);
                packet
            }
        }

    };
    //println!("{}",expanded);

    TokenStream::from(expanded)
}

fn write_fields(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields
                    .named
                    .iter()
                    .filter(|f| !get_name_attribute("ignore".into(), &f.attrs))
                    .map(|f| {
                        let name = &f.ident;
                        let macro_call = type_name_to_ident("write_".into(), &f.ty, f.span());
                        let count = get_type_attribute("count".into(), &f.attrs);
                        let macro_count = if count.is_some() {
                            let ident = type_ident_to_macro_ident(
                                "write_".into(),
                                count.clone().unwrap(),
                                f.span(),
                            );
                            quote! { #ident!(packet, self.#name.len() as #count); }
                        } else {
                            quote! {}
                        };
                        quote_spanned! {f.span() =>
                            #macro_count
                            #macro_call!(packet, self.#name);
                        }
                    });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn write_fields_macro(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields
                    .named
                    .iter()
                    .filter(|f| !get_name_attribute("ignore".into(), &f.attrs))
                    .map(|f| {
                        let name = &f.ident;
                        let macro_call = type_name_to_ident("write_".into(), &f.ty, f.span());
                        quote_spanned! {f.span() =>
                            #macro_call!($buf, $val.#name);
                        }
                    });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn get_attribute<T>(name: String, input: Vec<syn::Attribute>) -> Option<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    for att in input.into_iter() {
        let attr = att.parse_meta().unwrap();
        match attr {
            Meta::NameValue(MetaNameValue {
                ref path, ref lit, ..
            }) if path.get_ident().unwrap().to_string() == name => {
                if let Lit::Int(lit) = lit {
                    return Some(lit.base10_parse::<T>().unwrap());
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}

fn get_type_attribute(name: String, input: &Vec<syn::Attribute>) -> Option<syn::Ident> {
    for att in input.into_iter() {
        let attr = att.parse_meta().unwrap();
        match attr {
            Meta::List(MetaList {
                ref path, nested, ..
            }) if path.get_ident().unwrap().to_string() == name => match nested.first() {
                Some(NestedMeta::Meta(meta)) => {
                    return Some(meta.path().get_ident().unwrap().clone());
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    None
}

fn get_name_attribute(name: String, input: &Vec<syn::Attribute>) -> bool {
    for att in input.into_iter() {
        let attr = att.parse_meta().unwrap();
        match attr {
            Meta::Path(p) if p.get_ident().unwrap().to_string() == name => return true,
            _ => return false,
        }
    }
    false
}

fn type_name_to_ident(prefix: String, ty: &Type, s: proc_macro2::Span) -> Ident {
    let mut name = format!(
        "{}{}",
        prefix,
        quote!(#ty)
            .to_string()
            .as_str()
            .replace("\"", "")
            .replace(" ", "")
    );
    name = if name.contains("Vec<") {
        name.replace("Vec<", "arr_").replace(">", "")
    } else {
        name
    };
    let ident = Ident::new(&name, s);
    ident
}

fn type_ident_to_macro_ident(prefix: String, ty: syn::Ident, s: proc_macro2::Span) -> Ident {
    let name = format!(
        "{}{}",
        prefix,
        quote!(#ty)
            .to_string()
            .as_str()
            .replace("\"", "")
            .replace(" ", "")
    );
    let ident = Ident::new(&name, s);
    ident
}
