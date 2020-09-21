use crate::object::{AeonObject};
use crate::value::{AeonValue};

macro_rules! serialize_arg(
    ($s:ident, $idx:ident, $val:expr) => {
        if $idx == 0 {
            $s.push_str($val);
        } else {
            $s.push(',');
            $s.push(' ');
            $s.push_str($val);
        }
    }
);

pub struct Serializer {
    pub aeon: AeonObject,
}


impl Serializer {
    pub fn new(aeon: AeonObject) -> Serializer {
        Serializer {
            aeon
        }
    }

    fn serialize_macros(&self, s: &mut String) {
        for m in &self.aeon.macros {
            s.push('@');
            s.push_str(m.1.name.as_str());
            s.push('(');
            for arg in 0..m.1.args.len() {
                serialize_arg!(s, arg, &m.1.args[arg]);
            }
            s.push(')');
            s.push('\n');
        }
    }

    fn serialize_property_value(&self, s: &mut String, val: &AeonValue) {
        match val {
            AeonValue::Nil => {
                s.push_str("nil");
            },
            AeonValue::String(v) => {
                s.push('"');
                s.push_str(v.as_str());
                s.push('"');
            },
            AeonValue::Integer(v) => {
                s.push_str(&v.to_string());
            },
            AeonValue::Double(v) => {
                s.push_str(&v.to_string());
            },
            AeonValue::List(v) => {
                s.push('[');
                for i in 0..v.len() {
                    if i != 0 {
                        s.push(',');
                        s.push(' ');
                    }
                    self.serialize_property_value(s, &v[i]);
                }
                s.push(']');
            }
            AeonValue::Map(v) => {
                if let Some(m) = self.aeon.try_get_macro(v) {
                    // first check if a macro exists for this map
                    s.push_str(&m.name);
                    s.push('(');
                    for i in 0..m.args.len() {
                        if i != 0 {
                            s.push(',');
                            s.push(' ');
                        }
                        self.serialize_property_value(s, &v[&m.args[i]]);
                    }
                    s.push(')');
                } else {
                    // if not, serialize as a regular map
                    s.push('{');
                    let mut f = true;
                    for (k, v) in v.iter() {
                        if f { f = false; }
                        else {
                            s.push(',');
                            s.push(' ');
                        }
                        s.push('"');
                        s.push_str(k.as_str());
                        s.push('"');
                        s.push(':');
                        s.push(' ');
                        self.serialize_property_value(s, v);
                    }
                    s.push('}');
                }
            }
        }
    }

    fn serialize_properties(&self, s: &mut String) {
        for p in &self.aeon.properties {
            s.push_str(p.1.name.as_str());
            s.push(':');
            s.push(' ');
            self.serialize_property_value(s, &p.1.value);

            s.push('\n');
        }
    }


    pub fn serialize(&self) -> String {
        let mut s = String::with_capacity(50);
        self.serialize_macros(&mut s);
        self.serialize_properties(&mut s);
        s
    }

}

#[cfg(test)]
mod tests {
    use crate::object::{AeonObject, AeonProperty, Macro};
    use crate::value::{AeonValue};
    use crate::serializer::Serializer;
    use crate::map;

    #[test]
    pub fn serialize_using_macros() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("char".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Nil,
        ]));
        aeon.add_macro(Macro::new("character".into(), vec![
            "name".into(),
            "world".into(),
            "double".into(),
            "or_nothing".into(),
        ]));
        aeon.add_property(aeon_value);
        let ser = Serializer::new(aeon);
        let serialized = ser.serialize();
        assert_eq!("@character(name, world, double, or_nothing)\nchar: character(\"erki\", 1, 139.3567, nil)\n", serialized);
    }

    #[test]
    pub fn serialize_using_nested_macros() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("char".into(), AeonValue::Map(map![
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
        aeon.add_macro(Macro::new("character".into(), vec![
            "name".into(),
            "world".into(),
            "double".into(),
            "or_nothing".into(),
        ]));
        aeon.add_property(aeon_value);
        let ser = Serializer::new(aeon);
        let serialized = ser.serialize();
        assert_eq!("@character(name, world, double, or_nothing)\nchar: character(\"erki\", 1, 139.3567, character(\"unused\", -53, -11.38, nil))\n", serialized);
    }

    #[test]
    pub fn serialize_map_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("character".into(), AeonValue::Map(map![
           "name".into() => AeonValue::String("erki".into()),
           "world".into() => AeonValue::Integer(1),
           "double".into() => AeonValue::Double(139.3567),
           "or_nothing".into() => AeonValue::Nil,
        ]));
        aeon.add_property(aeon_value);
        let ser = Serializer::new(aeon);
        let serialized = ser.serialize();
        // TODO: regex or rewrite serialize implementation to be more testable
        // or just don't test the entire serialization and instead its parts
        assert!(serialized.starts_with("character: {\""));
        assert!(serialized.ends_with("}\n"));
        assert!(serialized.contains(r#""name": "erki""#));
        assert!(serialized.contains(r#""world": 1"#));
        assert!(serialized.contains(r#""double": 139.3567"#));
        assert!(serialized.contains(r#""or_nothing": nil"#));
        assert!(serialized.contains(","));
    }

    #[test]
    pub fn serialize_list_of_strings_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("characters".into(), AeonValue::List(vec![
           AeonValue::String("erki".into()),
           AeonValue::String("persiko".into()),
           AeonValue::String("frukt".into()),
           AeonValue::String("152436.13999".into()),
        ]));
        aeon.add_property(aeon_value);
        let ser = Serializer::new(aeon);
        assert_eq!("characters: [\"erki\", \"persiko\", \"frukt\", \"152436.13999\"]\n", ser.serialize());
    }

    #[test]
    pub fn serialize_string_property() {
        let mut aeon = AeonObject::new();
        let aeon_value = AeonProperty::new("character".into(), AeonValue::String("erki".into()));
        aeon.add_property(aeon_value);
        let ser = Serializer::new(aeon);
        assert_eq!("character: \"erki\"\n", ser.serialize());
    }


    #[test]
    pub fn serialize_macros() {
        let mut aeon = AeonObject::new();
        aeon.add_macro(Macro::new("character".into(), vec!["name".into(), "world".into()]));
        let ser = Serializer::new(aeon);
        assert_eq!("@character(name, world)\n", ser.serialize());
    }
}
