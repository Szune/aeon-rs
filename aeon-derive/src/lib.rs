mod utils;

extern crate proc_macro;

use aeon_derive_utils::{Field, Struct, Type};
use proc_macro::{Ident, TokenStream};
use std::collections::HashSet;
use std::str::FromStr;

#[proc_macro_derive(Deserialize, attributes(aeon))]
pub fn aeon_deserialize(input: TokenStream) -> TokenStream {
    let parsed = match aeon_derive_utils::parse_token_stream(input) {
        Err(err) => return err,
        Ok(ok) => ok,
    };

    let name = parsed.ident.clone();

    let property_assignments = generate_property_assignments(&parsed);

    let expanded = format!(
        r#"
impl aeon::AeonDeserialize for {} {{
    fn from_aeon(s: String) -> aeon::DeserializeResult<Self> {{
        use aeon::AeonDeserializeProperty;
        aeon::deserialize(s).and_then(|doc| Self::from_property(doc.into()))
    }}
}}
impl aeon::AeonDeserializeProperty for {} {{
    fn from_property(field: aeon::value::AeonValue) -> aeon::DeserializeResult<Self> {{
        use aeon::convert::AeonConvert;
        Ok(Self {{
            {}
        }})
    }}
}}
"#,
        name, name, property_assignments
    );

    TokenStream::from_str(expanded.as_str())
        .expect("Internal proc_macro error in Deserialize of aeon-derive")
}

/* printing out all attributes in generate_property_assignments:
   if let Some(attr_values) = f.all_attrs("doc") {
       //.map(|d|
       //d.iter().flat_map(|a|&a.opts).collect::<Vec<&parts::AttributeOption>>()) {
       let attr_values = f
           .attrs
           .iter()
           .flat_map(|a| &a.opts)
           .collect::<Vec<&parts::AttributeOption>>();
       format!(
           "{}: {{ /// {}\r\n struct Testoof{{}}  panic!()}}, ",
           name,
           attr_values
               .into_iter()
               .map(|x| format!("{:?}", x))
               .collect::<Vec<String>>()
               .join(" |||||| ")
       )
   } else {
       format!("{}: {},", name, prop_assignment)
   }
*/

fn generate_property_assignments(data: &Struct) -> String {
    let recurse: Vec<String> = data
        .fields
        .iter()
        .map(|f: &Field| {
            let name = f.ident.to_string();
            let prop_assignment = generate_property_assignment_for_type(&f.ident, &f.typ);
            format!("{}: {},", name, prop_assignment)
        })
        .collect();
    recurse.join("\r\n")
}

fn generate_property_assignment_for_type(name: &Ident, typ: &Type) -> String {
    let prop = format!("{:#}", name);

    match typ.ident.to_string().as_str() {
        "Option" => {
            format!(
                r#"
field
    .get("{}")
    .map(aeon::AeonDeserializeProperty::from_property)
    .transpose()?
"#,
                prop
            )
        }
        _ => {
            format!(
                r#"
field
    .get("{}")
    .map(aeon::AeonDeserializeProperty::from_property)
    .transpose()?
    .expect("Failed to deserialize property `{}`")
"#,
                prop, prop
            )
        }
    }
}

#[proc_macro_derive(Serialize, attributes(aeon))]
pub fn aeon_serialize(input: TokenStream) -> TokenStream {
    let parsed = match aeon_derive_utils::parse_token_stream(input) {
        Err(err) => return err,
        Ok(ok) => ok,
    };

    let name = parsed.ident.clone();

    let property_hashmap_insertions =
        generate_property_hashmap_insertions_for_serialization(&parsed);
    let insert_self_macro = generate_insert_self_macro(&name, &parsed);
    let create_macros_calls = generate_create_macros_calls(&parsed);

    let expanded = format!(
r#"        impl aeon::AeonSerialize for {} {{
            fn to_aeon(&self) -> aeon::SerializeResult<String> {{
                use aeon::document::AeonDocument;
                let mut doc = AeonDocument::try_from_object(self.to_aeon_value()?).unwrap();
                doc.set_macros(Self::create_macros(false));
                aeon::serialize(&doc)
            }}

            fn to_aeon_value(&self) -> aeon::SerializeResult<aeon::value::AeonValue> {{
                use aeon::AeonSerializeProperty;
                self.serialize_property()
            }}

            fn create_macros(insert_self: bool) -> std::collections::HashMap<String, aeon::document::AeonMacro> {{
                use aeon::AeonSerializeProperty;
                Self::create_property_macros(insert_self)
            }}
        }}
  "#, name) +
        format!(
r#"       impl aeon::AeonSerializeProperty for {} {{
            fn serialize_property(&self) -> aeon::SerializeResult<aeon::value::AeonValue> {{
                use aeon::value::AeonValue;
                let mut obj = std::collections::HashMap::<String, AeonValue>::new();
                {}
                Ok(AeonValue::Object(obj))
            }}

            fn create_property_macros(insert_self: bool) -> std::collections::HashMap<String, aeon::document::AeonMacro> {{
                use aeon::document::AeonMacro;
                let mut macros = std::collections::HashMap::<String, AeonMacro>::new();
                if insert_self {{
                    {}
                }}
                {}
                macros
            }}
        }}
"#, name, property_hashmap_insertions, insert_self_macro, create_macros_calls).as_str();

    TokenStream::from_str(expanded.as_str())
        .expect("Internal proc_macro error in Serialize of aeon-derive")
}

fn generate_property_hashmap_insertions_for_serialization(data: &Struct) -> String {
    let recurse: Vec<String> = data
        .fields
        .iter()
        .map(|f: &Field| {
            let name = f.ident.to_string();
            let prop = format!("\"{}\"", name);
            format!(
                r#"
obj.insert(
    {}.into(),
    self.{}.serialize_property()?,
);
"#,
                prop, name
            )
        })
        .collect();

    recurse.join("\r\n")
}

fn generate_insert_self_macro(name: &Ident, data: &Struct) -> String {
    let recurse: Vec<String> = data
        .fields
        .iter()
        .map(|f: &Field| {
            let name = f.ident.to_string();
            let prop = format!("\"{}\"", name);
            format!("{},", prop)
        })
        .collect();

    let names = recurse.join("\r\n");
    let name_string = format!("\"{}\"", name);
    format!(
        r#"
macros.insert(
    {}.into(),
    AeonMacro::new_cloned(
        {},
        vec![
            {}
        ]
    )
);
"#,
        name_string, name_string, names
    )
}

fn generate_create_macros_calls(data: &Struct) -> String {
    let recurse: Vec<String> = data
        .fields
        .iter()
        .flat_map(|f: &Field| utils::get_macro_types_from_type(&f.typ))
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|t| {
            format!(
                r#"
macros.extend({}::create_property_macros(true));
"#,
                t
            )
        })
        .collect();

    recurse.join("\r\n")
}
