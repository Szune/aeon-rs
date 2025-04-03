use crate::document::AeonDocument;
use crate::error::{AeonDeserializeError, AeonSerializeError};
use crate::serializer::AeonFormatter;
pub mod convert;
mod deserializer;
pub mod document;
pub mod error;
mod flags;
mod lexer;
mod macros;
mod serializer;
mod token;
pub mod value;

pub type DeserializeResult<T> = Result<T, AeonDeserializeError>;
pub type SerializeResult<T> = Result<T, AeonSerializeError>;

pub fn serialize(aeon: &AeonDocument) -> SerializeResult<String> {
    if aeon.is_empty {
        return Ok(String::new());
    }
    // TODO: write errors
    Ok(serializer::PrettySerializer::serialize_aeon(aeon))
}

pub fn deserialize(s: String) -> DeserializeResult<AeonDocument> {
    let mut deserializer = deserializer::Deserializer::new(&s);
    deserializer.deserialize()
}

pub trait AeonDeserialize
where
    Self: Sized,
{
    fn from_aeon(s: String) -> DeserializeResult<Self>;
}

pub trait AeonSerialize {
    fn to_aeon(&self) -> SerializeResult<String>;
    // TODO: rebuild this to not perform a bunch of unnecessary steps
    fn to_aeon_value(&self) -> SerializeResult<value::AeonValue>;
    fn create_macros(insert_self: bool) -> std::collections::HashMap<String, document::AeonMacro>;
}

pub trait AeonDeserializeProperty
where
    Self: Sized,
{
    fn from_property(field: value::AeonValue) -> DeserializeResult<Self>;
}

pub trait AeonSerializeProperty {
    // TODO: rebuild this to not perform a bunch of unnecessary steps
    fn serialize_property(&self) -> SerializeResult<value::AeonValue>;
    //fn serialize_property_or_nil(&self) -> value::AeonValue;
    fn create_property_macros(
        insert_self: bool,
    ) -> std::collections::HashMap<String, document::AeonMacro>;
}
