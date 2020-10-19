use crate::serializer::AeonFormatter;
mod macros;
pub mod object;
pub mod value;
mod lexer;
mod serializer;
mod deserializer;
mod token;
pub mod convert;
pub mod convert_panic;

pub fn serialize(aeon: object::AeonObject) -> String {
    if aeon.is_empty { return String::new() }
    //let ser = serializer::Serializer::new(aeon, true);
    //ser.serialize()
    serializer::PrettySerializer::serialize_aeon(aeon)
}

pub fn deserialize(s: String) -> Result<object::AeonObject, String> {
    let mut deserializer = deserializer::Deserializer::new(&s);
    deserializer.deserialize()
}


pub trait AeonDeserialize {
    fn from_aeon(s: String) -> Self;
    fn from_property(field: value::AeonValue) -> Self;
}

pub trait AeonSerialize {
    fn to_aeon(&self) -> String;
    // TODO: rebuild this to not perform a bunch of unnecessary steps
    fn serialize_aeon_property(&self) -> value::AeonValue;
    fn create_macros(insert_self: bool) -> std::collections::HashMap::<String, object::Macro>;
}

pub struct SerializedProperty {
    pub macros: std::collections::HashMap<String,object::Macro>,
    pub value: value::AeonValue,
}

