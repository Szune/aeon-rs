use std::collections::HashMap;

pub trait AeonConvert {
    fn nil(self) -> bool;
    fn str(self) -> Option<String>;
    fn int(self) -> Option<i64>;
    fn double(self) -> Option<f64>;
    fn map(self) -> Option<HashMap<String,AeonValue>>;
    fn list(self) -> Option<Vec<AeonValue>>;
    fn get(&self, prop: &str) -> Option<AeonValue>;
    fn remove(&mut self, prop: &str) -> Option<AeonValue>;
}

#[derive(Clone, Debug)]
pub enum AeonValue {
    Nil,
    String(String),
    Integer(i64),
    Double(f64),
    Map(HashMap<String,AeonValue>),
    List(Vec<AeonValue>),
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
