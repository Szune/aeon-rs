use crate::value::AeonValue;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Clone, Debug)]
pub struct AeonMacro {
    pub name: String,
    pub args: Vec<String>,
}

impl AeonMacro {
    pub fn new(name: String, args: Vec<String>) -> AeonMacro {
        AeonMacro { name, args }
    }

    pub fn new_cloned(name: &str, args: Vec<&str>) -> AeonMacro {
        AeonMacro {
            name: name.to_string(),
            args: args.into_iter().map(|a| a.to_string()).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn can_apply(&self, map: &HashMap<String, AeonValue>) -> bool {
        for arg in &self.args {
            if !map.contains_key(arg) {
                return false;
            }
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
        AeonProperty { name, value }
    }
}

#[derive(Debug, Default)]
pub struct AeonDocument {
    pub macros: HashMap<String, AeonMacro>,
    pub properties: HashMap<String, AeonProperty>,
    pub is_empty: bool,
}

impl AeonDocument {
    pub fn new() -> AeonDocument {
        AeonDocument {
            macros: HashMap::new(),
            properties: HashMap::new(),
            is_empty: true,
        }
    }

    pub fn try_from_object(object: AeonValue) -> Option<AeonDocument> {
        match object {
            AeonValue::Object(obj) => Some(Self::from_iter(obj)),
            _ => None,
        }
    }

    pub fn add_property(&mut self, name: &str, value: AeonValue) {
        if self.properties.contains_key(name) {
            return;
        }
        self.properties
            .insert(name.to_string(), AeonProperty::new(name.to_string(), value));
        self.is_empty = false;
    }

    pub fn add_or_replace_property(&mut self, name: String, value: AeonValue) {
        self.properties
            .insert(name.clone(), AeonProperty::new(name, value));
        self.is_empty = false;
    }

    pub fn add_macro(&mut self, mac: AeonMacro) {
        if self.macros.contains_key(&mac.name) {
            return;
        }
        self.macros.insert(mac.name.clone(), mac);
        self.is_empty = false;
    }

    pub fn add_or_replace_macro(&mut self, mac: AeonMacro) {
        self.macros.insert(mac.name.clone(), mac);
        self.is_empty = false;
    }

    pub fn set_macros(&mut self, macros: HashMap<String, AeonMacro>) {
        self.macros = macros;
    }

    pub fn apply_macro(&mut self, name: String, mut params: Vec<AeonValue>) -> AeonValue {
        if let Some(mac) = self.macros.get(name.as_str()) {
            let len = params.len();
            if mac.len() != len {
                panic!(
                    "Wrong number of args to macro {}: was {}, expected {}",
                    name,
                    len,
                    mac.len()
                );
            }

            let mut map = HashMap::<String, AeonValue>::new();

            for (idx, parameter) in params.drain(..).enumerate() {
                mac.apply(idx, parameter, &mut map);
            }
            AeonValue::Object(map)
        } else {
            panic!("Macro does not exist: {}", name);
        }
    }

    pub fn try_get_macro(&self, map: &HashMap<String, AeonValue>) -> Option<&AeonMacro> {
        if let Some((_, m)) = self
            .macros
            .iter()
            .filter(|(_, v)| v.len() == map.len())
            .find(|(_, v)| v.can_apply(map))
        {
            return Some(m);
        }
        None
    }

    pub fn copy_macros_to(&self, other: &mut AeonDocument) {
        other.macros.extend(self.macros.clone());
    }
}

impl FromIterator<(String, AeonValue)> for AeonDocument {
    fn from_iter<T: IntoIterator<Item = (String, AeonValue)>>(iter: T) -> Self {
        let mut doc = Self::new();
        iter.into_iter().for_each(|(k, v)| {
            doc.add_or_replace_property(k, v);
        });
        doc
    }
}

#[cfg(test)]
mod tests {
    use crate::convert::*;
    use crate::document::{AeonDocument, AeonMacro, AeonValue};
    use crate::map;

    #[test]
    pub fn get_property_integer() {
        let mut aeon = AeonDocument::new();
        aeon.add_property("num", AeonValue::Integer(79));
        assert_eq!(79, aeon.remove("num").int().unwrap());
    }

    #[test]
    pub fn get_property_string() {
        let mut aeon = AeonDocument::new();
        aeon.add_property("char", AeonValue::String("åöäºÃœ".into()));
        assert_eq!("åöäºÃœ".to_string(), aeon.remove("char").str().unwrap());
    }

    #[test]
    pub fn copy_macros() {
        let mut aeon = AeonDocument::new();
        aeon.add_macro(AeonMacro::new("bread".into(), vec!["'s good".into()]));
        let mut second_aeon = AeonDocument::new();
        second_aeon.add_macro(AeonMacro::new("tubes".into(), vec!["of copper".into()]));
        aeon.copy_macros_to(&mut second_aeon);
        assert_eq!(1, aeon.macros.len());
        assert_eq!(2, second_aeon.macros.len());
    }

    #[test]
    pub fn get_nested_property() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "char",
            AeonValue::Object(map![
               "name".into() => AeonValue::String("erki".into()),
               "world".into() => AeonValue::Integer(1),
               "double".into() => AeonValue::Double(139.3567),
               "or_nothing".into() => AeonValue::Object(map![
                     "name".into() => AeonValue::String("unused".into()),
                     "world".into() => AeonValue::Integer(-53),
                     "double".into() => AeonValue::Double(-11.38),
                     "or_nothing".into() => AeonValue::Nil,
               ]),
            ]),
        );
        assert_eq!(
            -53,
            aeon.remove("char")
                .remove("or_nothing")
                .get("world")
                .int()
                .unwrap()
        );
    }

    #[test]
    pub fn get_nested_property_using_path_syntax() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "char",
            AeonValue::Object(map![
               "name".into() => AeonValue::String("erki".into()),
               "world".into() => AeonValue::Integer(1),
               "double".into() => AeonValue::Double(139.3567),
               "or_nothing".into() => AeonValue::Object(map![
                     "name".into() => AeonValue::String("unused".into()),
                     "world".into() => AeonValue::Integer(-53),
                     "double".into() => AeonValue::Double(-11.38),
                     "or_nothing".into() => AeonValue::Nil,
               ]),
            ]),
        );
        assert_eq!(-53, aeon.get_path("char/or_nothing/world").int().unwrap());
    }

    #[test]
    pub fn remove_nested_property_using_path_syntax() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "char",
            AeonValue::Object(map![
               "name".into() => AeonValue::String("erki".into()),
               "world".into() => AeonValue::Integer(1),
               "double".into() => AeonValue::Double(139.3567),
               "or_nothing".into() => AeonValue::Object(map![
                     "name".into() => AeonValue::String("unused".into()),
                     "world".into() => AeonValue::Integer(-53),
                     "double".into() => AeonValue::Double(-11.38),
                     "or_nothing".into() => AeonValue::Nil,
               ]),
            ]),
        );
        assert_eq!(
            -53,
            aeon.remove_path("char/or_nothing/world").int().unwrap()
        );
    }
}
