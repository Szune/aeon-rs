use crate::value::{AeonValue};
use crate::object::{AeonObject};
use std::collections::HashMap;

pub trait AeonConvert {
    fn nil(self) -> bool;
    fn bool(self) -> Option<bool>;
    fn str(self) -> Option<String>;
    fn int(self) -> Option<i64>;
    fn double(self) -> Option<f64>;
    fn map(self) -> Option<HashMap<String,AeonValue>>;
    fn list(self) -> Option<Vec<AeonValue>>;
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
        match self {
            AeonValue::Nil => true,
            _ => false,
        }
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

    fn map(self) -> Option<HashMap<String,AeonValue>> {
        try_convert!(self, AeonValue::Map)
    }

    fn list(self) -> Option<Vec<AeonValue>> {
        try_convert!(self, AeonValue::List)
    }

    fn get(&self, prop: &str) -> Option<AeonValue> {
        match self {
            AeonValue::Map(v) => {
                if let Some(p) = v.get(prop) {
                    Some(p.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn remove(&mut self, prop: &str) -> Option<AeonValue> {
        match self {
            AeonValue::Map(v) => {
                if let Some(p) = v.remove(prop) {
                    Some(p)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

macro_rules! opt_convert(
    ($self:ident, $conversion:ident) => {
        if let Some(a) = $self {
            a.$conversion()
        } else {
            None
        }
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
        if let Some(a) = self { a.nil() } 
        else { false }
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

    fn map(self) -> Option<HashMap<String,AeonValue>> {
        opt_convert!(self, map)
    }

    fn list(self) -> Option<Vec<AeonValue>> {
        opt_convert!(self, list)
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

impl AeonObjectConvert for AeonObject {
    fn get(&self, prop: &str) -> Option<AeonValue> {
        return if let Some(p) = self.properties.get(prop) {
            Some(p.value.clone())
        } else {
            None
        }
    }

    fn get_path(&self, path: &str) -> Option<AeonValue> {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| f != "");
        let mut current : Option<AeonValue>;
        if let Some(frag) = iter.next() {
            current = self.get(frag);
        } else {
            return None;
        }

        while let Some(frag) = iter.next() {
            current = current.get(frag);
        }
        current
    }

    fn remove(&mut self, prop: &str) -> Option<AeonValue> {
        return if let Some(p) = self.properties.remove(prop) {
            Some(p.value)
        } else {
            None
        }
    }

    fn remove_path(&mut self, path: &str) -> Option<AeonValue> {
        let fragments = path.split('/');
        let mut iter = fragments.filter(|&f| f != "");
        let mut current : Option<AeonValue>;
        if let Some(frag) = iter.next() {
            current = self.remove(frag);
        } else {
            return None;
        }

        while let Some(frag) = iter.next() {
            current = current.remove(frag);
        }
        current
    }
}
