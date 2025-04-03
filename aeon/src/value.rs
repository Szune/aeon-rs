use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum AeonValue {
    Nil,
    Bool(bool),
    String(String),
    Integer(i64),
    Double(f64),
    Object(HashMap<String, AeonValue>),
    List(Vec<AeonValue>),
}

impl From<crate::document::AeonDocument> for AeonValue {
    fn from(doc: crate::document::AeonDocument) -> Self {
        let mut obj = HashMap::<String, AeonValue>::new();
        doc.properties
            .into_values()
            .for_each(|a| obj.insert(a.name, a.value).map_or((), |_| ()));
        AeonValue::Object(obj)
    }
}

impl AeonValue {
    pub const fn tag(&self) -> u8 {
        match self {
            AeonValue::Nil => 0,
            AeonValue::Bool(_) => 1,
            AeonValue::String(_) => 2,
            AeonValue::Integer(_) => 3,
            AeonValue::Double(_) => 4,
            AeonValue::Object(_) => 5,
            AeonValue::List(_) => 6,
        }
    }

    pub const fn tag_to_str(tag: u8) -> &'static str {
        match tag {
            0 => "nil",
            1 => "bool",
            2 => "string",
            3 => "int",
            4 => "double",
            5 => "object",
            6 => "list",
            _ => "Missing implementation for tag, bug in AeonValue::tag_to_str",
        }
    }
}
