use crate::object::{AeonObject, Macro, AeonProperty};
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

pub trait AeonFormatter {
    fn serialize_aeon(obj: AeonObject) -> String;
    fn serialize_macro(&mut self, mac: &Macro, s: &mut String);
    fn serialize_property(&mut self, obj: &AeonObject, property: &AeonProperty, s: &mut String);
    fn serialize_value(&mut self, obj: &AeonObject, value: &AeonValue, s: &mut String);
}

pub struct PrettySerializer {
    indent: i8,
    indent_skip: bool,
}

impl AeonFormatter for PrettySerializer {
    fn serialize_aeon(obj: AeonObject) -> String {
        let mut ser = PrettySerializer {
            indent: 0,
            indent_skip: false,
        };
        let mut s = String::with_capacity(50);
        for (_,v) in &obj.macros {
            ser.serialize_macro(v, &mut s);
        }
        s.push('\n');
        for (_,v) in &obj.properties {
            ser.serialize_property(&obj, v, &mut s);
            s.push('\n');
            s.push('\n');
        }
        s
    }

    fn serialize_macro(&mut self, mac: &Macro, s: &mut String) {
        s.push('@');
        s.push_str(mac.name.as_str());
        s.push('(');
        for arg in 0..mac.args.len() {
            serialize_arg!(s, arg, &mac.args[arg]);
        }
        s.push(')');
        s.push('\n');
    }

    fn serialize_property(&mut self, obj: &AeonObject, property: &AeonProperty, s: &mut String) {
        s.push_str(property.name.as_str());
        s.push(':');
        s.push(' ');
        self.serialize_value(obj, &property.value, s);

    }

    fn serialize_value(&mut self, obj: &AeonObject, value: &AeonValue, s: &mut String) {
        macro_rules! indent_me {
            ($self:expr, $s:expr) => {
                if !$self.indent_skip {
                    for _ in 0..=$self.indent {
                        $s.push(' ');
                    }
                } else {
                    $self.indent_skip = false;
                }
            }
        }
        match value {
            AeonValue::Nil => {
                indent_me!(self, s);
                s.push_str("nil");
            },
            AeonValue::Bool(v) => {
                indent_me!(self, s);
                s.push_str(if *v { "true" } else { "false" }); // could probably just use v.to_string() here
            },
            AeonValue::String(v) => {
                indent_me!(self, s);
                s.push('"');
                s.push_str(v.as_str());
                s.push('"');
            },
            AeonValue::Integer(v) => {
                indent_me!(self, s);
                s.push_str(&v.to_string());
            },
            AeonValue::Double(v) => { // TODO: this is probably a bad idea, fix
                indent_me!(self, s);
                s.push_str(&v.to_string());
            },
            AeonValue::List(v) => {
                indent_me!(self, s);
                s.push('[');

                self.indent += 4;
                for i in 0..v.len() {
                    if i != 0 {
                        s.push(',');
                        s.push(' ');
                    }
                    s.push('\n');
                    self.serialize_value(obj, &v[i], s);
                }
                self.indent -= 4;
                if !v.is_empty() {
                    s.push('\n');
                    indent_me!(self, s);
                }
                s.push(']');
            },
            AeonValue::Map(v) => {
                indent_me!(self, s);
                if let Some(m) = obj.try_get_macro(v) {
                    // first check if a macro exists for this map
                    s.push_str(&m.name);
                    s.push('(');
                    self.indent += 4;
                    if v.iter().any(|(_,v)| matches!(v, AeonValue::List(_))) {
                        self.indent_skip = true;
                        for i in 0..m.args.len() {
                            if i != 0 {
                                s.push(',');
                                s.push('\n');
                            }
                            self.serialize_value(obj, &v[&m.args[i]], s);
                        }
                        self.indent -= 4;
                        if !v.is_empty() {
                            s.push('\n');
                            indent_me!(self, s);
                        }
                    } else {
                        for i in 0..m.args.len() {
                            self.indent_skip = true;
                            if i != 0 {
                                s.push(',');
                                s.push(' ');
                            }
                            self.serialize_value(obj, &v[&m.args[i]], s);
                        }
                        self.indent -= 4;
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
                        s.push('\n');
                        s.push('"');
                        s.push_str(k.as_str());
                        s.push('"');
                        s.push(':');
                        s.push(' ');
                        self.indent_skip = true;
                        self.serialize_value(obj, v, s);
                    }
                    if !v.is_empty() {
                        s.push('\n');
                        indent_me!(self, s);
                    }
                    s.push('}');
                }
            }
        }
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
