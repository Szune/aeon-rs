mod macros;
mod object;
mod value;
mod lexer;
mod serializer;
mod deserializer;
mod token;

pub use value::AeonConvert;

pub fn serialize(aeon: object::AeonObject) -> String {
    if aeon.is_empty { return String::new() }
    let ser = serializer::Serializer::new(aeon);
    ser.serialize()
}

pub fn deserialize(s: String) -> Result<object::AeonObject, String> {
    let mut deserializer = deserializer::Deserializer::new(&s);
    deserializer.deserialize()
}
