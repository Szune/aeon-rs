use crate::value::{AeonValue};
use crate::object::{AeonObject};
use std::collections::HashMap;
use crate::{AeonDeserialize,AeonSerialize};

pub trait AeonConvert {
    fn nil(self) -> bool;
    fn bool(self) -> bool;
    fn str(self) -> String;
    fn int(self) -> i64;
    fn double(self) -> f64;
    fn ip(self) -> std::net::IpAddr;
    fn ip_str(self) -> String;
    fn map(self) -> HashMap<String,AeonValue>;
    fn list(self) -> Vec<AeonValue>;
    fn get(&self, prop: &str) -> AeonValue;
    fn remove(&mut self, prop: &str) -> AeonValue;
}

macro_rules! panic_convert(
    ($self:ident, $to:path) => {
        match $self {
            $to(v) => v,
            _ => panic!("Invalid value conversion for {:?}", $self),
        }
    };
);

impl AeonConvert for AeonValue {
    fn nil(self) -> bool {
        match self {
            AeonValue::Nil => true,
            _ => false,
        }
    }

    fn bool(self) -> bool {
        panic_convert!(self, AeonValue::Bool)
    }

    fn str(self) -> String {
        panic_convert!(self, AeonValue::String)
    }

    fn int(self) -> i64 {
        panic_convert!(self, AeonValue::Integer)
    }

    fn double(self) -> f64 {
        panic_convert!(self, AeonValue::Double)
    }

    fn ip(self) -> std::net::IpAddr {
        panic_convert!(self, AeonValue::Ip)
    }

    fn ip_str(self) -> String {
        match self {
            AeonValue::Ip(ip) => ip.to_string(),
            _ => panic!("Invalid value conversion for {:?}", self),
        }
    }


    fn map(self) -> HashMap<String,AeonValue> {
        panic_convert!(self, AeonValue::Map)
    }

    fn list(self) -> Vec<AeonValue> {
        panic_convert!(self, AeonValue::List)
    }

    fn get(&self, prop: &str) -> AeonValue {
        match self {
            AeonValue::Map(v) => {
                if let Some(p) = v.get(prop) {
                    p.clone()
                } else {
                    panic!("Failed to get property {:?}", prop);
                }
            }
            _ => panic!("Failed to get property {:?}", prop),
        }
    }

    fn remove(&mut self, prop: &str) -> AeonValue {
        match self {
            AeonValue::Map(v) => {
                if let Some(p) = v.remove(prop) {
                    p
                } else {
                    panic!("Failed to get property {:?}", prop);
                }
            }
            _ => panic!("Failed to get property {:?}", prop),
        }
    }

}

pub trait AeonObjectConvert {
    fn get(&self, prop: &str) -> AeonValue;
    fn get_path(&self, path: &str) -> AeonValue;
    fn remove(&mut self, prop: &str) -> AeonValue;
    fn remove_path(&mut self, path: &str) -> AeonValue;
}

impl AeonObjectConvert for AeonObject {
    fn get(&self, prop: &str) -> AeonValue {
        return if let Some(p) = self.properties.get(prop) {
            p.value.clone()
        } else {
            panic!("Failed to get property '{}'", prop);
        }
    }

    fn get_path(&self, path: &str) -> AeonValue {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| f != "");
        let mut current : AeonValue;
        if let Some(frag) = iter.next() {
            current = self.get(frag);
        } else {
            panic!("Failed to get property '{}'", path);
        }

        while let Some(frag) = iter.next() {
            current = current.get(frag);
        }
        current
    }

    fn remove(&mut self, prop: &str) -> AeonValue {
        return if let Some(p) = self.properties.remove(prop) {
            p.value
        } else {
            panic!("Failed to get property '{}'", prop);
        }
    }

    fn remove_path(&mut self, path: &str) -> AeonValue {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| f != "");
        let mut current : AeonValue;
        if let Some(frag) = iter.next() {
            current = self.remove(frag);
        } else {
            panic!("Failed to get property '{}'", path);
        }

        while let Some(frag) = iter.next() {
            current = current.remove(frag);
        }
        current
    }
}

macro_rules! gen_deserialize {
    ($ty:ident, $conv:ident) => {
        impl AeonDeserialize for $ty {
            fn from_property(field: AeonValue) -> Self {
                field.$conv() as $ty
            }

            fn from_aeon(_s: String) -> Self {
                unimplemented!()
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

macro_rules! gen_serialize {
    ($ty:ident, $val:ident, $conv:ident) => {
        impl AeonSerialize for $ty {
            fn to_aeon(&self) -> String {
                unimplemented!()
            }

            fn serialize_aeon_property(&self) -> crate::value::AeonValue {
                AeonValue::$val(self.clone() as $conv)
            }

            fn create_macros(_insert_self: bool) -> std::collections::HashMap::<String, crate::object::Macro> {
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
