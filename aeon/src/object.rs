use crate::value::{AeonValue};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Macro {
    pub name: String,
    pub args: Vec<String>,
}

impl Macro {
    pub fn new(name: String, args: Vec<String>) -> Macro {
        Macro {
            name,
            args,
        }
    }

    pub fn len(&self) -> usize { self.args.len() }

    pub fn can_apply(&self, map: &HashMap<String, AeonValue>) -> bool {
        for arg in &self.args {
            if !map.contains_key(arg) { return false; }
        }
        true
    }

    pub fn apply(&self, index: usize, value: AeonValue, map: &mut HashMap<String, AeonValue>) {
        map.insert(self.args[index].clone(), value);
    }
}


#[derive(Debug)]
pub struct AeonProperty {
    pub name: String,
    pub value: AeonValue,
}

impl AeonProperty {
    pub fn new(name: String, value: AeonValue) -> AeonProperty {
        AeonProperty {
            name,
            value,
        }
    }
}

#[derive(Debug)]
pub struct AeonObject {
    pub macros: HashMap<String, Macro>,
    pub properties: HashMap<String, AeonProperty>,
    pub is_empty: bool,
}

impl AeonObject {
    pub fn new() -> AeonObject {
        AeonObject {
            macros: HashMap::new(),
            properties: HashMap::new(),
            is_empty: true,
        }
    }

    pub fn add_property(&mut self,  value: AeonProperty) {
        if self.properties.contains_key(&value.name) {
            return;
        }
        self.properties.insert(value.name.clone(), value);
        self.is_empty = false;
    }

    pub fn add_macro(&mut self, mac: Macro) {
        if self.macros.contains_key(&mac.name) {
            return;
        }
        self.macros.insert(mac.name.clone(), mac);
        self.is_empty = false;
    }

    pub fn apply_macro(&mut self, name: String, mut params: Vec<AeonValue>) -> AeonValue {
        if let Some(mac) = self.macros.get(name.as_str()) {
            let len = params.len();
            if mac.len() != len {
                panic!("Wrong number of args to macro {}: was {}, expected {}", name, len, mac.len());
            }

            let mut map = HashMap::<String,AeonValue>::new();

            for (idx,parameter) in params.drain(..).enumerate() {
                mac.apply(idx, parameter, &mut map);
            }
            AeonValue::Map(map)
        } else {
            panic!("Macro does not exist: {}", name);
        }
    }

    pub fn try_get_macro(&self, map: &HashMap<String, AeonValue>) -> Option<&Macro> {
        if let Some((_, m)) = self.macros
            .iter()
            .filter(|(_,v)| v.len() == map.len())
            .find(|(_,v)| v.can_apply(map)) {
                return Some(m);
        }
        None
    }

    pub fn copy_macros_to(&self, other: &mut AeonObject) {
        other.macros.extend(self.macros.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::object::{AeonProperty, AeonValue, AeonObject, Macro};
    use crate::convert::*;
    use crate::map;

    #[test]
    pub fn get_property_integer() {
        let mut aeon = AeonObject::new();
        let prop = AeonProperty::new("num".into(), AeonValue::Integer(79));
        aeon.add_property(prop);
        assert_eq!(79, aeon.remove("num").int().unwrap());
    }

    #[test]
    pub fn get_property_string() {
        let mut aeon = AeonObject::new();
        let prop = AeonProperty::new("char".into(), AeonValue::String("åöäºÃœ".into()));
        aeon.add_property(prop);
        assert_eq!("åöäºÃœ".to_string(), aeon.remove("char").str().unwrap());
    }

    #[test]
    pub fn copy_macros() {
        let mut aeon = AeonObject::new();
        aeon.add_macro(Macro::new("bread".into(), vec!["'s good".into()]));
        let mut second_aeon = AeonObject::new();
        second_aeon.add_macro(Macro::new("tubes".into(), vec!["of copper".into()]));
        aeon.copy_macros_to(&mut second_aeon);
        assert_eq!(1, aeon.macros.len());
        assert_eq!(2, second_aeon.macros.len());
    }

    #[test]
    pub fn get_nested_property() {
        let mut aeon = AeonObject::new();
        let prop = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Map(map![
                 "name".into() => AeonValue::String("unused".into()),
                 "world".into() => AeonValue::Integer(-53),
                 "double".into() => AeonValue::Double(-11.38),
                 "or_nothing".into() => AeonValue::Nil,
           ]),
        ]));
        aeon.add_property(prop);
        assert_eq!(-53, aeon.remove("char").remove("or_nothing").get("world").int().unwrap());
    }

    #[test]
    pub fn get_nested_property_using_path_syntax() {
        let mut aeon = AeonObject::new();
        let prop = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Map(map![
                 "name".into() => AeonValue::String("unused".into()),
                 "world".into() => AeonValue::Integer(-53),
                 "double".into() => AeonValue::Double(-11.38),
                 "or_nothing".into() => AeonValue::Nil,
           ]),
        ]));
        aeon.add_property(prop);
        assert_eq!(-53, aeon.get_path("char/or_nothing/world").int().unwrap());
    }

    #[test]
    pub fn remove_nested_property_using_path_syntax() {
        let mut aeon = AeonObject::new();
        let prop = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Map(map![
                 "name".into() => AeonValue::String("unused".into()),
                 "world".into() => AeonValue::Integer(-53),
                 "double".into() => AeonValue::Double(-11.38),
                 "or_nothing".into() => AeonValue::Nil,
           ]),
        ]));
        aeon.add_property(prop);
        assert_eq!(-53, aeon.remove_path("char/or_nothing/world").int().unwrap());
    }
}
