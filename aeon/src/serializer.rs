use crate::document::{AeonDocument, AeonMacro, AeonProperty};
use crate::value::AeonValue;

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
    fn serialize_aeon(obj: &AeonDocument) -> String;
    fn serialize_macro(&mut self, mac: &AeonMacro, s: &mut String);
    fn serialize_property(&mut self, obj: &AeonDocument, property: &AeonProperty, s: &mut String);
    fn serialize_value(&mut self, obj: &AeonDocument, value: &AeonValue, s: &mut String);
}

pub struct PrettySerializer {
    indent: i8,
    indent_skip: bool,
}

impl PrettySerializer {
    pub fn new() -> Self {
        Self {
            indent: 0,
            indent_skip: false,
        }
    }
}

impl AeonFormatter for PrettySerializer {
    fn serialize_aeon(obj: &AeonDocument) -> String {
        let mut ser = PrettySerializer::new();
        let mut s = String::with_capacity(50);
        for mac in obj.macros.values() {
            ser.serialize_macro(mac, &mut s);
        }
        if !obj.macros.is_empty() {
            s.push('\n');
        }
        for prop in obj.properties.values() {
            ser.serialize_property(obj, prop, &mut s);
            s.push('\n');
            s.push('\n');
        }
        s
    }

    fn serialize_macro(&mut self, mac: &AeonMacro, s: &mut String) {
        s.push('@');
        s.push_str(mac.name.as_str());
        s.push('(');
        for arg in 0..mac.args.len() {
            serialize_arg!(s, arg, &mac.args[arg]);
        }
        s.push(')');
        s.push('\n');
    }

    fn serialize_property(&mut self, obj: &AeonDocument, property: &AeonProperty, s: &mut String) {
        s.push_str(property.name.as_str());
        s.push(':');
        s.push(' ');
        self.serialize_value(obj, &property.value, s);
    }

    fn serialize_value(&mut self, obj: &AeonDocument, value: &AeonValue, s: &mut String) {
        macro_rules! indent_me {
            ($self:expr, $s:expr) => {
                if !$self.indent_skip {
                    for _ in 0..$self.indent {
                        $s.push(' ');
                    }
                } else {
                    $self.indent_skip = false;
                }
            };
        }
        match value {
            AeonValue::Nil => {
                indent_me!(self, s);
                s.push_str("nil");
            }
            AeonValue::Bool(v) => {
                indent_me!(self, s);
                s.push_str(if *v { "true" } else { "false" }); // could probably just use v.to_string() here
            }
            AeonValue::String(v) => {
                indent_me!(self, s);
                s.push('"');
                for x in v.chars() {
                    match x {
                        '\\' => {
                            s.push('\\');
                            s.push('\\');
                        }
                        '\r' => {
                            s.push('\\');
                            s.push('r');
                        }
                        '\n' => {
                            s.push('\\');
                            s.push('n');
                        }
                        '\t' => {
                            s.push('\\');
                            s.push('t');
                        }
                        '"' => {
                            s.push('\\');
                            s.push('"');
                        }
                        _ => {
                            s.push(x);
                        }
                    }
                }
                s.push('"');
            }
            AeonValue::Integer(v) => {
                indent_me!(self, s);
                s.push_str(&v.to_string());
            }
            AeonValue::Double(v) => {
                indent_me!(self, s);
                s.push_str(&format!("{:?}", v));
            }
            AeonValue::List(v) => {
                indent_me!(self, s);
                s.push('[');

                self.indent += 4;
                for (i, item) in v.iter().enumerate() {
                    if i != 0 {
                        s.push(',');
                    }
                    s.push('\n');
                    self.serialize_value(obj, item, s);
                }
                self.indent -= 4;
                if !v.is_empty() {
                    s.push('\n');
                    indent_me!(self, s);
                }
                s.push(']');
            }
            AeonValue::Object(v) => {
                indent_me!(self, s);
                if let Some(m) = obj.try_get_macro(v) {
                    // first check if a macro exists for this map
                    s.push_str(&m.name);
                    s.push('(');
                    self.indent += 4;
                    if v.iter().any(|(_, v)| matches!(v, AeonValue::List(_))) {
                        // map contains a list
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
                        // map contains no list
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
                        if f {
                            f = false;
                        } else {
                            s.push(',');
                            s.push(' ');
                        }
                        s.push('\n');
                        if !is_valid_identifier(k.as_str()) {
                            s.push('"');
                            s.push_str(k.as_str());
                            s.push('"');
                        } else {
                            s.push_str(k.as_str());
                        }
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

fn is_valid_identifier(s: &str) -> bool {
    let start_valid = match s.get(0..=0) {
        None => false,
        Some(first) => first
            .chars()
            .next()
            .is_some_and(|it| it.is_ascii_alphabetic()),
    };
    if !start_valid {
        return false;
    }
    // keywords are not identifiers
    if s == "nil" || s == "true" || s == "false" {
        return false;
    }
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => (),
            _ => return false,
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use crate::document::{AeonDocument, AeonMacro};
    use crate::map;
    use crate::serializer::{AeonFormatter, PrettySerializer};
    use crate::value::AeonValue;

    #[test]
    pub fn serialize_using_macros() {
        let mut aeon = AeonDocument::new();
        aeon.add_macro(AeonMacro::new(
            "character".into(),
            vec![
                "name".into(),
                "world".into(),
                "double".into(),
                "or_nothing".into(),
            ],
        ));
        aeon.add_property(
            "char",
            AeonValue::Object(map![
               "name".into() => AeonValue::String("erki".into()),
               "world".into() => AeonValue::Integer(1),
               "double".into() => AeonValue::Double(139.3567),
               "or_nothing".into() => AeonValue::Nil,
            ]),
        );
        let serialized = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!("@character(name, world, double, or_nothing)\n\nchar: character(\"erki\", 1, 139.3567, nil)\n\n", serialized);
    }

    #[test]
    pub fn serialize_using_nested_macros() {
        let mut aeon = AeonDocument::new();
        aeon.add_macro(AeonMacro::new(
            "character".into(),
            vec![
                "name".into(),
                "world".into(),
                "double".into(),
                "or_nothing".into(),
            ],
        ));
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
        let serialized = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!("@character(name, world, double, or_nothing)\n\nchar: character(\"erki\", 1, 139.3567, character(\"unused\", -53, -11.38, nil))\n\n", serialized);
    }

    #[test]
    pub fn serialize_map_property() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "character",
            AeonValue::Object(map![
               "name".into() => AeonValue::String("erki".into()),
               "world".into() => AeonValue::Integer(1),
               "double".into() => AeonValue::Double(139.3567),
               "or_nothing".into() => AeonValue::Nil,
            ]),
        );
        let serialized = PrettySerializer::serialize_aeon(&aeon);
        // TODO: regex or rewrite serialize implementation to be more testable
        // or just don't test the entire serialization and instead its parts
        assert!(serialized.starts_with("character: {\n"));
        assert!(serialized.ends_with("}\n\n"));
        assert!(serialized.contains(r#"name: "erki""#));
        assert!(serialized.contains(r#"world: 1"#));
        assert!(serialized.contains(r#"double: 139.3567"#));
        assert!(serialized.contains(r#"or_nothing: nil"#));
        assert!(serialized.contains(','));
    }

    #[test]
    pub fn serialize_list_of_strings_property() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "characters",
            AeonValue::List(vec![
                AeonValue::String("erki".into()),
                AeonValue::String("persiko".into()),
                AeonValue::String("frukt".into()),
                AeonValue::String("152436.13999".into()),
            ]),
        );
        let serialized = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!("characters: [\n    \"erki\",\n    \"persiko\",\n    \"frukt\",\n    \"152436.13999\"\n]\n\n", serialized);
    }

    #[test]
    pub fn serialize_string_property() {
        let mut aeon = AeonDocument::new();
        aeon.add_property("character", AeonValue::String("erki".into()));
        let ser = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!("character: \"erki\"\n\n", ser);
    }

    #[test]
    pub fn serialize_string_property_with_escape_char() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "testing",
            AeonValue::String("C:\\Path\\Is\\Escaped\"oh quote\nline\ttab".into()),
        );
        let ser = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!(
            "testing: \"C:\\\\Path\\\\Is\\\\Escaped\\\"oh quote\\nline\\ttab\"\n\n",
            ser
        );
    }

    #[test]
    pub fn serialize_double_with_no_decimals() {
        let mut aeon = AeonDocument::new();
        aeon.add_property(
            "nodecimals",
            AeonValue::Double(85.0),
        );
        let ser = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!(
            "nodecimals: 85.0\n\n",
            ser
        );
    }

    #[test]
    pub fn serialize_macros() {
        let mut aeon = AeonDocument::new();
        aeon.add_macro(AeonMacro::new(
            "character".into(),
            vec!["name".into(), "world".into()],
        ));
        let ser = PrettySerializer::serialize_aeon(&aeon);
        assert_eq!("@character(name, world)\n\n", ser);
    }
}
