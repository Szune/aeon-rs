extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, Meta, Type, Field
};

#[proc_macro_derive(Deserialize, attributes(ignore))]
pub fn aeon_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    assert!(
        input.generics.type_params().next().is_none(),
        "Deserialize is not implemented for generic types"
    );

    let from_aeon_func = generate_from_aeon_func(&input.data);
    let from_property_func = generate_from_property_func(&input.data);

    let expanded = quote! {
        impl AeonDeserialize for #name {
            fn from_property(field: aeon::value::AeonValue) -> Self {
                Self {
                    #from_property_func
                }
            }

            fn from_aeon(s: String) -> Self {
                let aeon = aeon::deserialize(s).unwrap();
                Self {
                    #from_aeon_func
                }
            }
        }

    };

    TokenStream::from(expanded)
}

/// These identifiers do not contain the surrounding Vec or HashMap!
enum SerTy {
    Regular(Ident),
    Vec(Ident),
    Map(Ident),
}

fn generate_from_property_func(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let name = &f.ident;
        let ser_ty = get_ser_ty(&f.ty, f.span());
        let func_call = from_property_func_call_from_ser_ty(name.clone().unwrap(), ser_ty);
        quote_spanned! {f.span() =>
            #name : #func_call,
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn from_property_func_call_from_ser_ty(name: Ident, ser_ty: SerTy) -> proc_macro2::TokenStream {
    let prop = format!("{:#}", name);
    match ser_ty { // ordered by complexity
        SerTy::Map(_) => quote! { field.get(#prop).map() },
        SerTy::Regular(id) => quote! { #id::from_property(field.get(#prop)) },
        SerTy::Vec(id) => quote! { field.get(#prop).list().drain(..).map(|field| #id::from_property(field)).collect() },
    }
}

fn generate_from_aeon_func(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let name = &f.ident;
        let ser_ty = get_ser_ty(&f.ty, f.span());
        let func_call = from_aeon_func_call_from_ser_ty(name.clone().unwrap(), ser_ty);
        quote_spanned! {f.span() =>
            #name : #func_call,
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn from_aeon_func_call_from_ser_ty(name: Ident, ser_ty: SerTy) -> proc_macro2::TokenStream {
    let prop = format!("{:#}", name);
    match ser_ty { // ordered by complexity
        SerTy::Map(_) => quote! { aeon.get(#prop).map() },
        SerTy::Regular(id) => quote! { #id::from_property(aeon.get(#prop)) },
        SerTy::Vec(id) => quote! { aeon.get(#prop).list().drain(..).map(|field| #id::from_property(field)).collect() },
    }
}

/// Get type that is being serialized/deserialized for easier/lazier handling
fn get_ser_ty(ty: &Type, s: proc_macro2::Span) -> SerTy {
    let mut ty_name = quote!(#ty).to_string() // may need .as_str()
        .replace("\"", "")
        .replace(" ", "");

    macro_rules! inner_generic_ty {
        ($ty:ident) => (
            {
                // keep inner generic name
                let begin = ty_name.find('<').unwrap();
                ty_name.replace_range(..=begin, "");
                let end = ty_name.rfind('>').unwrap();
                ty_name.replace_range(end.., "");
                SerTy::$ty(Ident::new(&ty_name, s))
            }
        );
    }

    if ty_name.contains("Vec<") {
        inner_generic_ty!(Vec)
    } else if ty_name.contains("HashMap<") {
        inner_generic_ty!(Map)
    } else {
        SerTy::Regular(Ident::new(&ty_name, s))
    }
}


#[proc_macro_derive(Serialize, attributes(ignore))]
pub fn aeon_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    assert!(
        input.generics.type_params().next().is_none(),
        "Serialize is not implemented for generic types"
    );

    let name = input.ident;
    let name_lower = quote!(#name).to_string().to_lowercase();

    let to_aeon_func = generate_to_aeon_func(&input.data);
    let serialize_aeon_property_func = generate_serialize_aeon_property_func(&input.data);
    let create_macros_func = generate_create_macros_func(&input.data);

    let args_vec = generate_args_vec(&input.data);

    let expanded = quote! {
        impl AeonSerialize for #name {
            fn to_aeon(&self) -> String {
                let mut obj = aeon::object::AeonObject::new();
                #to_aeon_func

                let mut macros = Self::create_macros(false);
                macros.drain().for_each(|(k,v)| obj.add_macro(v));
                aeon::serialize(obj)
            }

            fn serialize_aeon_property(&self) -> aeon::value::AeonValue {
                let mut map = std::collections::HashMap::<String,aeon::value::AeonValue>::new();
                #serialize_aeon_property_func
                aeon::value::AeonValue::Map(map)
            }

            fn create_macros(insert_self: bool) -> std::collections::HashMap::<String, aeon::object::Macro> {
                let mut macros = std::collections::HashMap::<String,aeon::object::Macro>::new();
                if insert_self {
                    macros.insert(#name_lower.into(),
                        aeon::object::Macro::new(#name_lower.into(), #args_vec));
                }
                #create_macros_func
                macros
            }
        }

    };

    TokenStream::from(expanded)
}

fn generate_to_aeon_func(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let name = &f.ident;
        let ser_ty = get_ser_ty(&f.ty, f.span());
        let func_call = to_aeon_func_call_from_ser_ty(name.clone().unwrap(), ser_ty);
        quote_spanned! {f.span() =>
            #func_call
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn to_aeon_func_call_from_ser_ty(name: Ident, ser_ty: SerTy) -> proc_macro2::TokenStream {
    let prop = format!("{:#}", name).to_lowercase();
    match ser_ty { // ordered by complexity
        // TODO: allow serializing HashMap<String,TNotAeonValue> if the values can be converted to
        // AeonValues
        SerTy::Map(_) => quote! {
            obj.add_property(aeon::object::AeonProperty::new(
                    #prop.into(),
                    aeon::value::AeonValue::Map(self.#name.clone()))); 
        },
        SerTy::Regular(id) => quote! { 
            { 
                let ser = #id::serialize_aeon_property(&self.#name);
                obj.add_property(aeon::object::AeonProperty::new(#prop.into(), ser));
            }
        },
        SerTy::Vec(id) => quote! {
            obj.add_property(
                aeon::object::AeonProperty::new(
                    #prop.into(),
                    aeon::value::AeonValue::List(
                        self.#name.iter()
                        .map(|item| {
                            let ser = #id::serialize_aeon_property(item);
                            ser
                        }).collect()))); 
        },
    }
}

fn generate_create_macros_func(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let ser_ty = get_ser_ty(&f.ty, f.span());
        let func_call = create_macros_func_call_from_ser_ty(ser_ty);
        quote_spanned! {f.span() =>
            #func_call
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn create_macros_func_call_from_ser_ty(ser_ty: SerTy) -> proc_macro2::TokenStream {
    match ser_ty { // ordered by complexity
        SerTy::Map(_) => quote! { }, // no point in making macros for arbitrary hashmaps
        SerTy::Regular(id) => quote! {
            macros.extend(#id::create_macros(true));
        },
        SerTy::Vec(id) => quote! { 
            macros.extend(#id::create_macros(true));
        },
    }
}

fn generate_serialize_aeon_property_func(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let name = &f.ident;
        let ser_ty = get_ser_ty(&f.ty, f.span());
        let func_call = serialize_aeon_property_func_call_from_ser_ty(name.clone().unwrap(), ser_ty);
        quote_spanned! {f.span() =>
            #func_call
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn serialize_aeon_property_func_call_from_ser_ty(name: Ident, ser_ty: SerTy) -> proc_macro2::TokenStream {
    let prop = format!("{:#}", name).to_lowercase();
    match ser_ty { // ordered by complexity
        // TODO: allow serializing HashMap<String,TNotAeonValue> if the values can be converted to
        // AeonValues
        SerTy::Map(_m) => quote! {
            map.insert(#prop.into(),
                       aeon::value::AeonValue::Map(self.#name.clone()));
        },
        SerTy::Regular(id) => quote! {
            {
                let ser = #id::serialize_aeon_property(&self.#name);
                map.insert(#prop.into(), ser);
            }
        },
        SerTy::Vec(id) => quote! {
            map.insert(#prop.into(),
                aeon::value::AeonValue::List(
                    self.#name.iter()
                    .map(|item| {
                        let ser = #id::serialize_aeon_property(item);
                        ser
                    }).collect()));
        },
    }
}

fn generate_args_vec(data: &Data) -> proc_macro2::TokenStream {
    let fields = get_struct_fields(data);
    let recurse = fields.iter().map(|f| {
        let name = f.ident.clone().unwrap();
        let name_lower = quote!(#name).to_string().to_lowercase();
        quote_spanned! {f.span() =>
            #name_lower.into(),
        }
    });
    quote! {
        vec![ #(#recurse)* ]
    }
}

fn get_struct_fields(data: &Data) -> Vec<&Field> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                fields
                    .named
                    .iter()
                    .filter(|f| !get_name_attribute("ignore".into(), &f.attrs))
                    .collect()
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
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

