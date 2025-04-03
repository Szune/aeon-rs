use proc_macro::Ident;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Modifier {
    Pub,
    None,
}

#[derive(Debug)]
pub struct Struct {
    pub ident: Ident,
    pub modifier: Modifier,
    pub fields: Vec<Field>,
    pub attrs: Vec<Attribute>,
}

impl Struct {
    pub fn attr(&self, ident: &str) -> Option<&Attribute> {
        self.attrs
            .iter()
            .find(|attr| attr.ident.to_string() == ident)
    }

    pub fn all_attrs(&self, ident: &str) -> Option<Vec<&Attribute>> {
        let attrs: Vec<&Attribute> = self
            .attrs
            .iter()
            .filter(|attr| attr.ident.to_string() == ident)
            .collect();
        if attrs.is_empty() {
            None
        } else {
            Some(attrs)
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub modifier: Modifier,
    pub typ: Type,
    pub attrs: Vec<Attribute>,
}

impl Field {
    pub fn attr(&self, ident: &str) -> Option<&Attribute> {
        self.attrs
            .iter()
            .find(|attr| attr.ident.to_string() == ident)
    }

    pub fn all_attrs(&self, ident: &str) -> Option<Vec<&Attribute>> {
        let attrs: Vec<&Attribute> = self
            .attrs
            .iter()
            .filter(|attr| attr.ident.to_string() == ident)
            .collect();
        if attrs.is_empty() {
            None
        } else {
            Some(attrs)
        }
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub ident: Ident,
    pub opts: Vec<AttributeOption>,
}

impl Attribute {
    pub fn first_value(&self) -> Option<&AttributeValue> {
        self.opts.first().map(|opt| match opt {
            AttributeOption::KeyValue(_, v) | AttributeOption::Value(v) => v,
        })
    }

    pub fn opt(&self, ident: &str) -> Option<&AttributeOption> {
        self.opts.iter().find(|opt| {
            matches!(opt, AttributeOption::Value(AttributeValue::Ident(id))
            | AttributeOption::KeyValue(id, _) if id.to_string() == ident)
        })
    }

    pub fn opt_value(&self, ident: &str) -> Option<&AttributeValue> {
        self.opt(ident).map(|opt| match opt {
            AttributeOption::KeyValue(_, v) | AttributeOption::Value(v) => v,
        })
    }
}

#[derive(Debug)]
pub enum AttributeOption {
    KeyValue(Ident, AttributeValue),
    Value(AttributeValue),
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// Any literal value that is not an ident
    ///
    /// N.B. calling `to_string()` on [`proc_macro::Literal`] returns the literal
    /// as it is written in the source, e.g. `"test"` returns `"\"test\""`, `5` returns `"5"`
    Literal(proc_macro::Literal),
    /// Any ident that is true/false
    Bool(bool),
    /// Any ident that is not true/false
    Ident(Ident),
}

impl AttributeValue {
    pub fn unwrap_literal(self) -> proc_macro::Literal {
        match self {
            AttributeValue::Literal(lit) => lit,
            AttributeValue::Bool(_) => {
                panic!("called `AttributeValue::unwrap_literal()` on a `Bool` value")
            }
            AttributeValue::Ident(_) => {
                panic!("called `AttributeValue::unwrap_literal()` on an `Ident` value")
            }
        }
    }

    pub fn unwrap_bool(self) -> bool {
        match self {
            AttributeValue::Bool(b) => b,
            AttributeValue::Literal(_) => {
                panic!("called `AttributeValue::unwrap_bool()` on a `Literal` value")
            }
            AttributeValue::Ident(_) => {
                panic!("called `AttributeValue::unwrap_bool()` on an `Ident` value")
            }
        }
    }

    pub fn unwrap_ident(self) -> Ident {
        match self {
            AttributeValue::Ident(id) => id,
            AttributeValue::Literal(_) => {
                panic!("called `AttributeValue::unwrap_ident()` on a `Literal` value")
            }
            AttributeValue::Bool(_) => {
                panic!("called `AttributeValue::unwrap_ident()` on a `Bool` value")
            }
        }
    }
}

#[derive(Debug)]
pub enum PathPart {
    Ident(Ident),
    DoubleColon,
}

#[derive(Debug)]
pub struct Type {
    pub ident: Ident,
    /// Full _given_ path, not necessarily fully qualified path
    pub full_path: Vec<PathPart>,
    pub generics: Vec<Type>,
}

impl Type {
    /// Returns everything but generics
    pub fn to_full_path(&self) -> String {
        let mut s = String::new();
        for p in &self.full_path {
            match p {
                PathPart::Ident(i) => {
                    s.push_str(i.to_string().as_str());
                }
                PathPart::DoubleColon => {
                    s.push_str("::");
                }
            }
        }
        s
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut last_was_double_colon = false;
        for p in &self.full_path {
            match p {
                PathPart::Ident(i) => {
                    last_was_double_colon = false;
                    write!(f, "{}", i)?;
                }
                PathPart::DoubleColon => {
                    last_was_double_colon = true;
                    write!(f, "::")?;
                }
            }
        }

        if !self.generics.is_empty() {
            if !last_was_double_colon {
                write!(f, "::")?;
            }
            write!(f, "<")?;
            for (idx, g) in self.generics.iter().enumerate() {
                if idx > 0 {
                    write!(f, ",")?;
                }
                g.fmt(f)?;
            }
            write!(f, ">")?;
        }

        Ok(())
    }
}
