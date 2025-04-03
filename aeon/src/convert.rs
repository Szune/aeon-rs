use crate::document::{AeonDocument, AeonMacro};
use crate::value::AeonValue;
use crate::{
    AeonDeserializeError, AeonDeserializeProperty, AeonSerializeProperty, DeserializeResult,
    SerializeResult,
};
use std::collections::HashMap;

pub trait AeonConvert {
    fn nil(self) -> bool;
    fn bool(self) -> Option<bool>;
    fn str(self) -> Option<String>;
    fn int(self) -> Option<i64>;
    fn double(self) -> Option<f64>;
    fn object(self) -> Option<HashMap<String, AeonValue>>;
    fn list(self) -> Option<Vec<AeonValue>>;
    fn list_of<T: AeonDeserializeProperty>(self) -> Option<Vec<T>>;
    fn of<T: AeonDeserializeProperty>(self) -> Option<T>;
    fn get(&self, prop: &str) -> Option<AeonValue>;
    fn remove(&mut self, prop: &str) -> Option<AeonValue>;
}

macro_rules! try_convert(
    ($self:ident, $to:path) => {
        match $self {
            $to(v) => Some(v),
            _ => None,
        }
    };
);

impl AeonConvert for AeonValue {
    fn nil(self) -> bool {
        matches!(self, AeonValue::Nil)
    }

    fn bool(self) -> Option<bool> {
        try_convert!(self, AeonValue::Bool)
    }

    fn str(self) -> Option<String> {
        try_convert!(self, AeonValue::String)
    }

    fn int(self) -> Option<i64> {
        try_convert!(self, AeonValue::Integer)
    }

    fn double(self) -> Option<f64> {
        try_convert!(self, AeonValue::Double)
    }

    fn object(self) -> Option<HashMap<String, AeonValue>> {
        try_convert!(self, AeonValue::Object)
    }

    fn list(self) -> Option<Vec<AeonValue>> {
        try_convert!(self, AeonValue::List)
    }

    fn list_of<T: AeonDeserializeProperty>(self) -> Option<Vec<T>> {
        self.list()
            .and_then(|mut a| a.drain(..).map(T::from_property).map(Result::ok).collect())
    }

    fn of<T: AeonDeserializeProperty>(self) -> Option<T> {
        T::from_property(self).ok()
    }

    fn get(&self, prop: &str) -> Option<AeonValue> {
        match self {
            AeonValue::Object(v) => v.get(prop).cloned(),
            _ => None,
        }
    }

    fn remove(&mut self, prop: &str) -> Option<AeonValue> {
        match self {
            AeonValue::Object(v) => v.remove(prop),
            _ => None,
        }
    }
}

macro_rules! opt_convert(
    ($self:ident, $conversion:ident) => {
        $self.and_then(|a|a.$conversion())
    };

    ($self:ident, $conversion:ident, $arg:ident) => {
        if let Some(a) = $self {
            a.$conversion($arg)
        } else {
            None
        }
    };
);

impl AeonConvert for Option<AeonValue> {
    fn nil(self) -> bool {
        if let Some(a) = self {
            a.nil()
        } else {
            false
        }
    }

    fn bool(self) -> Option<bool> {
        opt_convert!(self, bool)
    }

    fn str(self) -> Option<String> {
        opt_convert!(self, str)
    }

    fn int(self) -> Option<i64> {
        opt_convert!(self, int)
    }

    fn double(self) -> Option<f64> {
        opt_convert!(self, double)
    }

    fn object(self) -> Option<HashMap<String, AeonValue>> {
        opt_convert!(self, object)
    }

    fn list(self) -> Option<Vec<AeonValue>> {
        opt_convert!(self, list)
    }

    fn list_of<T: AeonDeserializeProperty>(self) -> Option<Vec<T>> {
        self.and_then(|a| a.list_of())
    }

    fn of<T: AeonDeserializeProperty>(self) -> Option<T> {
        self.and_then(|a| a.of())
    }

    fn get(&self, prop: &str) -> Option<AeonValue> {
        opt_convert!(self, get, prop)
    }

    fn remove(&mut self, prop: &str) -> Option<AeonValue> {
        opt_convert!(self, remove, prop)
    }
}

pub trait AeonObjectConvert {
    fn get(&self, prop: &str) -> Option<AeonValue>;
    fn get_path(&self, path: &str) -> Option<AeonValue>;
    fn remove(&mut self, prop: &str) -> Option<AeonValue>;
    fn remove_path(&mut self, path: &str) -> Option<AeonValue>;
}

impl AeonObjectConvert for AeonDocument {
    fn get(&self, prop: &str) -> Option<AeonValue> {
        self.properties.get(prop).map(|p| p.value.clone())
    }

    fn get_path(&self, path: &str) -> Option<AeonValue> {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| !f.is_empty());
        let mut current: Option<AeonValue>;
        if let Some(frag) = iter.next() {
            current = self.get(frag);
        } else {
            return None;
        }

        for frag in iter {
            current = current.get(frag);
        }
        current
    }

    fn remove(&mut self, prop: &str) -> Option<AeonValue> {
        if let Some(p) = self.properties.remove(prop) {
            Some(p.value)
        } else {
            None
        }
    }

    fn remove_path(&mut self, path: &str) -> Option<AeonValue> {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| !f.is_empty());
        let mut current: Option<AeonValue>;
        if let Some(frag) = iter.next() {
            current = self.remove(frag);
        } else {
            return None;
        }

        for frag in iter {
            current = current.remove(frag);
        }
        current
    }
}

macro_rules! gen_deserialize {
    ($ty:path, $conv:ident) => {
        impl AeonDeserializeProperty for $ty {
            fn from_property(field: AeonValue) -> DeserializeResult<Self> {
                field.$conv().map_or_else(
                    || {
                        Err(AeonDeserializeError::conversion(format!(
                            "Failed to convert '{}' to '{}'",
                            stringify!($ty),
                            stringify!($conv)
                        )))
                    },
                    |a| Ok(a as $ty),
                )
            }
        }
    };
}

gen_deserialize!(bool, bool);
gen_deserialize!(String, str);
gen_deserialize!(i64, int);
gen_deserialize!(i32, int);
gen_deserialize!(i16, int);
gen_deserialize!(i8, int);
gen_deserialize!(u64, int);
gen_deserialize!(u32, int);
gen_deserialize!(u16, int);
gen_deserialize!(u8, int);
gen_deserialize!(f64, double);
gen_deserialize!(f32, double);

pub fn maybe<T: AeonDeserializeProperty>(thing: Option<AeonValue>) -> DeserializeResult<Option<T>> {
    thing
        .map(Option::<T>::from_property)
        .unwrap_or_else(|| Ok(None))
}

pub fn expected<T: AeonDeserializeProperty>(thing: Option<AeonValue>) -> DeserializeResult<T> {
    thing.map(AeonDeserializeProperty::from_property).unwrap()
}

impl<T: AeonDeserializeProperty> AeonDeserializeProperty for Option<T> {
    fn from_property(field: AeonValue) -> DeserializeResult<Self> {
        match field {
            AeonValue::Nil => Ok(None),
            _ => T::from_property(field).map(|a| Some(a)),
        }
    }
}

impl<T: AeonDeserializeProperty> AeonDeserializeProperty for Vec<T> {
    fn from_property(field: AeonValue) -> DeserializeResult<Self> {
        let field_type = field.tag();
        field
            .list()
            .map(|v| v.into_iter().map(T::from_property).collect())
            .unwrap_or_else(|| {
                Err(AeonDeserializeError::deserialization(format!(
                    "Failed to convert {:?} to {:?}",
                    AeonValue::tag_to_str(field_type),
                    std::any::type_name::<T>()
                )))
            })
    }
}

impl<T: AeonDeserializeProperty> AeonDeserializeProperty for HashMap<String, T> {
    fn from_property(field: AeonValue) -> DeserializeResult<Self> {
        let field_type = field.tag();
        field
            .object()
            .map(|m| {
                m.into_iter()
                    .map(|(k, v)| Ok((k, T::from_property(v)?)))
                    .collect()
            })
            .unwrap_or_else(|| {
                Err(AeonDeserializeError::deserialization(format!(
                    "Failed to convert {:?} to HashMap<String, {:?}>",
                    AeonValue::tag_to_str(field_type),
                    std::any::type_name::<T>()
                )))
            })
    }
}

impl AeonDeserializeProperty for HashMap<String, AeonValue> {
    fn from_property(field: AeonValue) -> DeserializeResult<Self> {
        let field_type = field.tag();
        field.object().map(Ok).unwrap_or_else(|| {
            Err(AeonDeserializeError::deserialization(format!(
                "Failed to convert {:?} to HashMap<String, AeonValue>",
                AeonValue::tag_to_str(field_type)
            )))
        })
    }
}

macro_rules! gen_serialize {
    ($ty:path, $val:ident, $conv:path) => {
        impl AeonSerializeProperty for $ty {
            fn serialize_property(&self) -> SerializeResult<AeonValue> {
                Ok(AeonValue::$val(self.clone() as $conv))
            }

            fn create_property_macros(
                _insert_self: bool,
            ) -> HashMap<String, crate::document::AeonMacro> {
                std::collections::HashMap::new()
            }
        }
    };
}

gen_serialize!(bool, Bool, bool);
gen_serialize!(String, String, String);
gen_serialize!(i64, Integer, i64);
gen_serialize!(i32, Integer, i64);
gen_serialize!(i16, Integer, i64);
gen_serialize!(i8, Integer, i64);
gen_serialize!(u64, Integer, i64);
gen_serialize!(u32, Integer, i64);
gen_serialize!(u16, Integer, i64);
gen_serialize!(u8, Integer, i64);
gen_serialize!(f64, Double, f64);
gen_serialize!(f32, Double, f64);

// blanket impl for Option<T>
impl<T: AeonSerializeProperty> AeonSerializeProperty for Option<T> {
    fn serialize_property(&self) -> SerializeResult<AeonValue> {
        self.as_ref()
            .map(|a| a.serialize_property())
            .unwrap_or(Ok(AeonValue::Nil))
    }

    fn create_property_macros(insert_self: bool) -> HashMap<String, AeonMacro> {
        T::create_property_macros(insert_self)
    }
}

// blanket impl for Vec<T>
impl<T: AeonSerializeProperty> AeonSerializeProperty for Vec<T> {
    fn serialize_property(&self) -> SerializeResult<AeonValue> {
        let converted: SerializeResult<Vec<AeonValue>> =
            self.iter().map(T::serialize_property).collect();
        Ok(AeonValue::List(converted?))
    }

    fn create_property_macros(insert_self: bool) -> HashMap<String, AeonMacro> {
        T::create_property_macros(insert_self)
    }
}

// blanket impl for HashMap<String, T>
impl<T: AeonSerializeProperty> AeonSerializeProperty for HashMap<String, T> {
    fn serialize_property(&self) -> SerializeResult<AeonValue> {
        Ok(AeonValue::Object(
            self.iter()
                .map(|(k, v)| (k.clone(), T::serialize_property(v).unwrap()))
                .collect(),
        ))
    }

    fn create_property_macros(insert_self: bool) -> HashMap<String, AeonMacro> {
        T::create_property_macros(insert_self)
    }
}

impl AeonSerializeProperty for HashMap<String, AeonValue> {
    fn serialize_property(&self) -> SerializeResult<AeonValue> {
        Ok(AeonValue::Object(self.clone()))
    }

    fn create_property_macros(_insert_self: bool) -> HashMap<String, AeonMacro> {
        HashMap::new()
    }
}
