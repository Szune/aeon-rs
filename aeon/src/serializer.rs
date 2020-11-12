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

impl PrettySerializer {
	pub fn new() -> Self {
		Self {
            indent: 0,
            indent_skip: false,
        }
	}
}

impl AeonFormatter for PrettySerializer {
    fn serialize_aeon(obj: AeonObject) -> String {
        let mut ser = PrettySerializer::new();
        let mut s = String::with_capacity(50);
        for (_,v) in &obj.macros {
            ser.serialize_macro(v, &mut s);
        }
		if !obj.macros.is_empty() {
				s.push('\n');
		}
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
                    for _ in 0..$self.indent {
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
                        if f { f = false; }
                        else {
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
        Some(first) => matches!(first.chars().next().unwrap(), 'a' ..= 'z' | 'A' ..= 'Z'),
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
            'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => (),
            _ => return false,
        }
    }
    true
}