use crate::items::*;
use proc_macro::token_stream::IntoIter;
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use std::iter::Peekable;
use std::str::FromStr;

pub type ParseResult<T> = Result<T, TokenStream>;

/// Usage:
///
/// derive_error!("Expected <something>", v.span());
///
/// Can only be used in functions with the signature SomeInput -> Result<SomeOutput, TokenStream>
macro_rules! derive_error {
    ($msg:literal, $span:expr) => {
        return Err(
            TokenStream::from_str(format!("compile_error!(\"{}\");", $msg).as_str())
                .expect("Bug in proc_macro code")
                .into_iter()
                .map(|mut tt| {
                    tt.set_span($span);
                    tt
                })
                .collect(),
        );
    };

    ($msg:expr, $span:expr) => {
        return Err(TokenStream::from_str(
            format!("compile_error!(\"{}\");", $msg.replace("\"", "\\\"")).as_str(),
        )
        .expect("Bug in proc_macro code")
        .into_iter()
        .map(|mut tt| {
            tt.set_span($span);
            tt
        })
        .collect());
    };
}

macro_rules! require_token {
    (take $it:expr, $expect:path[$bind:ident] $(if $guard:expr)?, $span:expr, $errmsg:literal) => {
        match $it {
            $expect($bind) $(if $guard)? => $bind,
            _ => {
                let mut s = String::from($errmsg);
                s.push_str(" was: ");
                s.push_str(format!("{:?}", &$it).as_str());
                derive_error!(s, $span);
            }
        }
    };
    ($it:expr, $expect:pat $(if $guard:expr)?, $span:expr, $errmsg:literal) => {
        match $it {
            $expect $(if $guard)? => (),
            _ => {
                let mut s = String::from($errmsg);
                s.push_str(" was: ");
                s.push_str(format!("{:?}", &$it).as_str());
                derive_error!(s, $span);
            }
        }
    };
}

fn or_unexpected_end(
    it: Option<TokenTree>,
    span: Option<proc_macro::Span>,
) -> ParseResult<TokenTree> {
    match it {
        None => {
            derive_error!(
                "Unexpected end: Expected a full struct with braces as delimiters",
                span.unwrap_or_else(proc_macro::Span::call_site)
            );
        }
        Some(tt) => Ok(tt),
    }
}

pub fn parse_struct(ts: TokenStream) -> ParseResult<Struct> {
    let mut iter = ts.into_iter().peekable();

    let attrs = get_attrs(&mut iter)?;
    if attrs.is_none() {
        derive_error!(
            "Expected a struct or attribute(s) on struct",
            proc_macro::Span::call_site()
        );
    }
    let attrs = attrs.unwrap();

    let tt = or_unexpected_end(iter.next(), None)?;
    let maybe_pub = matches!(&tt, TokenTree::Ident(s) if s.to_string() == "pub");

    // if tt was consumed (matched "pub"), get next, otherwise use tt
    let tt = if maybe_pub {
        or_unexpected_end(iter.next(), Some(tt.span()))?
    } else {
        tt
    };
    require_token!(tt, TokenTree::Ident(ref v) if v.to_string() == "struct", tt.span(), "Expected 'struct'");

    let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;
    let span = tt.span();
    let struct_ident = require_token!(
        take tt,
        TokenTree::Ident[v],
        tt.span(),
        "Expected struct identifier (name)"
    );

    let tt = or_unexpected_end(iter.next(), Some(span))?;
    let struct_group = require_token!(
        take tt,
        TokenTree::Group[v] if matches!(v.delimiter(), Delimiter::Brace),
        tt.span(),
        "Expected braces surrounding struct fields (cannot be used on generic structs)");

    let fields = get_fields(struct_group)?;

    Ok(Struct {
        ident: struct_ident,
        modifier: if maybe_pub {
            Modifier::Pub
        } else {
            Modifier::None
        },
        fields,
        attrs,
    })
}

fn get_attrs(iter: &mut Peekable<IntoIter>) -> ParseResult<Option<Vec<Attribute>>> {
    // overarching parser of attributes
    // handles '#' and then delegates to other functions
    // then looks for another '#' to see if there are more attributes to parse
    let mut attrs = Vec::new();

    loop {
        let tt = iter.peek().cloned();
        if tt.is_none() {
            return if attrs.is_empty() {
                // this is fine, it means the last field has been parsed
                Ok(None)
            } else {
                // this is not fine, it means there is a trailing attribute in the struct
                // could be wrong, but that should be an error
                derive_error!("Trailing attribute", proc_macro::Span::call_site());
            };
        }

        let tt = tt.unwrap();
        let span = tt.span();

        if !matches!(tt, TokenTree::Punct(p) if p.as_char() == '#') {
            return Ok(Some(attrs));
        }

        // skip '#'
        let _ = iter.next().unwrap();

        let tt = or_unexpected_end(iter.next(), Some(span))?;
        let attr_group = require_token!(
        take tt,
        TokenTree::Group[v] if matches!(v.delimiter(), Delimiter::Bracket),
        tt.span(),
        "Expected brackets surrounding attribute");

        let attrs_from_group = get_attr(attr_group)?;
        attrs.extend(attrs_from_group);
    }
}

/// N.B. This should be called with a single attribute group
///
/// Handles "ident", "ident(...)" and "ident(...), ident(...)"
///
/// e.g. the parts surrounded by \[\] in "#\[attribute\]", "#\[attribute(...)\]" or "#\[attribute(...), other_attribute(...)\]"
fn get_attr(group: Group) -> ParseResult<Vec<Attribute>> {
    let mut attrs = Vec::new();
    let ts = group.stream();
    let mut iter = ts.into_iter().peekable();

    loop {
        // TODO: better span + error message
        let tt = or_unexpected_end(iter.next(), None)?;
        let attr_ident = require_token!(
            take tt,
            TokenTree::Ident[v],
            tt.span(),
            "Expected attribute identifier (name)"
        );
        // attr

        let tt = iter.next();
        if tt.is_none() {
            attrs.push(Attribute {
                ident: attr_ident,
                opts: Vec::new(),
            });
            break;
        }
        let tt = tt.unwrap();

        // comma found before group
        // attr,
        if matches!(tt, TokenTree::Punct(ref p) if p.as_char() == ',') {
            attrs.push(Attribute {
                ident: attr_ident,
                opts: Vec::new(),
            });

            if iter.peek().is_none() {
                break; // optional trailing comma
            } else {
                continue; // next attribute
            }
        }

        // TODO: handle attributes with macro calls inside: #[thing = include_str!("hello.md")]
        // attr =
        if matches!(tt, TokenTree::Punct(ref p) if p.as_char() == '=') {
            let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;

            let attr_option = require_token!(
                take tt,
                TokenTree::Literal[v],
                tt.span(),
                "Expected literal after equals in #[x = \"y\"] attribute format"
            );
            // attr = "test"

            attrs.push(Attribute {
                ident: attr_ident,
                opts: vec![AttributeOption::Value(AttributeValue::Literal(attr_option))],
            });

            // expect comma after group
            let tt = iter.next();
            if tt.is_none() {
                break; // no more attributes in attribute group
            }
            let tt = tt.unwrap();
            require_token!(tt, TokenTree::Punct(p) if p.as_char() == ',', tt.span(), "Expected comma");
            // attr = "test",
            continue; // next attribute
        }

        // attr(
        let attr_options_group = require_token!(
            take tt,
            TokenTree::Group[v] if matches!(v.delimiter(), Delimiter::Parenthesis),
            tt.span(),
            "Expected attribute options (macro calls in attributes have not been implemented yet)"
        );

        let attr_options = get_attr_options(attr_options_group)?;
        attrs.push(Attribute {
            ident: attr_ident,
            opts: attr_options,
        });
        // attr(...)

        // expect comma after group
        let tt = iter.next();
        if tt.is_none() {
            break; // no more attributes in attribute group
        }
        let tt = tt.unwrap();
        require_token!(tt, TokenTree::Punct(p) if p.as_char() == ',', tt.span(), "Expected comma");
        // attr(...),
    }

    Ok(attrs)
}

fn get_attr_options(group: Group) -> ParseResult<Vec<AttributeOption>> {
    let mut attr_options = Vec::new();
    let ts = group.stream();
    let mut iter = ts.into_iter().peekable();

    if iter.peek().is_none() {
        // attr()
        return Ok(attr_options);
    }

    loop {
        // TODO: better span + error message

        let tt = or_unexpected_end(iter.next(), None)?;
        // get value_or_key and save it
        // if there is an equal sign following the value_or_key, it is a key
        // if there is a comma or nothing following the value_or_key, it is a value
        let value_or_key = tt;
        // option

        let tt = iter.next();
        if tt.is_none() {
            // no more attribute options, value_or_key is a value
            let attr_option_value = tt_to_attr_value(value_or_key)?;
            attr_options.push(AttributeOption::Value(attr_option_value));
            break;
        }

        let tt = tt.unwrap();
        // comma found before equals, value_or_key is a value
        // option,
        if matches!(tt, TokenTree::Punct(ref p) if p.as_char() == ',') {
            let attr_option_value = tt_to_attr_value(value_or_key)?;
            attr_options.push(AttributeOption::Value(attr_option_value));

            if iter.peek().is_none() {
                break; // optional trailing comma
            } else {
                continue; // next attribute option
            }
        }

        // value_or_key is now known to be a key and an equal sign is expected
        let attr_option_ident = require_token!(
            take value_or_key,
            TokenTree::Ident[v],
            value_or_key.span(),
            "Expected attribute option key"
        );

        // require equal sign
        //let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;
        require_token!(tt, TokenTree::Punct(ref p) if p.as_char() == '=', tt.span(), "Expected equals and attribute option value");
        // option =

        let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;
        let attr_option_value = tt_to_attr_value(tt)?;
        // option = value
        attr_options.push(AttributeOption::KeyValue(
            attr_option_ident,
            attr_option_value,
        ));

        let tt = iter.next();
        if tt.is_none() {
            // no more attribute options
            break;
        }

        let tt = tt.unwrap();
        // require comma if there are more tokens after attribute option value
        // option = value,
        require_token!(tt, TokenTree::Punct(ref p) if p.as_char() == ',', tt.span(), "Expected comma before next attribute option");
        if iter.peek().is_none() {
            break; // optional trailing comma
        }

        // next option
    }

    Ok(attr_options)
}

fn tt_to_attr_value(it: TokenTree) -> ParseResult<AttributeValue> {
    Ok(match it {
        TokenTree::Ident(id) => {
            let id_str = id.to_string();
            match id_str.as_str() {
                "true" => AttributeValue::Bool(true),
                "false" => AttributeValue::Bool(false),
                _ => AttributeValue::Ident(id),
            }
        }
        TokenTree::Literal(lit) => AttributeValue::Literal(lit),
        _ => {
            derive_error!("Expected attribute value", it.span());
        }
    })
}

fn get_fields(group: Group) -> ParseResult<Vec<Field>> {
    let mut fields = Vec::new();
    let ts = group.stream();
    let mut iter = ts.into_iter().peekable();

    loop {
        let attrs = get_attrs(&mut iter)?;
        if attrs.is_none() {
            break;
        }
        let attrs = attrs.unwrap();

        let tt = or_unexpected_end(iter.next(), None)?;
        let maybe_pub = matches!(&tt, TokenTree::Ident(s) if s.to_string() == "pub");

        // if tt was consumed (matched "pub"), get next, otherwise use tt
        let tt = if maybe_pub {
            or_unexpected_end(iter.next(), Some(tt.span()))?
        } else {
            tt
        };
        let span = tt.span();
        let struct_ident = require_token!(
            take tt,
            TokenTree::Ident[v],
            tt.span(),
            "Expected field identifier (name)"
        );

        let tt = or_unexpected_end(iter.next(), Some(span))?;
        require_token!(tt, TokenTree::Punct(p) if p.as_char() == ':', tt.span(), "Expected ':' followed by field type");

        let field_type = recursive_get_field_type(&mut iter)?;
        fields.push(Field {
            ident: struct_ident,
            modifier: if maybe_pub {
                Modifier::Pub
            } else {
                Modifier::None
            },
            typ: field_type,
            attrs,
        });

        // comma or end
        let tt = iter.next();
        if tt.is_none() {
            break;
        }
        let tt = tt.unwrap();
        require_token!(tt, TokenTree::Punct(p) if p.as_char() == ',', tt.span(), "Expected ',' or end of struct");
    }

    Ok(fields)
}

fn recursive_get_field_type(iter: &mut Peekable<IntoIter>) -> ParseResult<Type> {
    let mut full_path = Vec::new();
    let mut generics = Vec::new();

    let tt = or_unexpected_end(iter.peek().cloned(), None)?;
    // check if there is a leading "::" (global path)
    let mut span = tt.span();
    if matches!(tt, TokenTree::Punct(p) if p.as_char() == ':') {
        let _ = iter.next().unwrap(); // skip peeked
        let tt = or_unexpected_end(iter.next(), Some(span))?;
        require_token!(tt, TokenTree::Punct(p) if p.as_char() == ':', tt.span(), "Expected '::'");
        full_path.push(PathPart::DoubleColon);
    }

    loop {
        let tt = or_unexpected_end(iter.next(), Some(span))?;
        span = tt.span();
        let current_ident = require_token!(
            take tt,
            TokenTree::Ident[v],
            tt.span(),
            "Expected path (identifier)"
        );
        full_path.push(PathPart::Ident(current_ident.clone()));

        match iter.peek() {
            Some(s) if matches!(s, TokenTree::Punct(p) if p.as_char() == ',' || p.as_char() == '>') =>
            {
                return Ok(Type {
                    ident: current_ident,
                    generics,
                    full_path,
                });
            }
            None => {
                return Ok(Type {
                    ident: current_ident,
                    generics,
                    full_path,
                });
            }
            _ => {}
        }

        // handle ident::ident, ident::<generic_arg> and ident<generic_arg>
        let mut delimiter_found = false;
        let tt = or_unexpected_end(iter.peek().cloned(), Some(span))?;
        if matches!(tt, TokenTree::Punct(p) if p.as_char() == ':') {
            let tt = iter.next().unwrap(); // skip peeked
            let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;
            span = tt.span();
            require_token!(tt, TokenTree::Punct(p) if p.as_char() == ':', tt.span(), "Expected '::'");
            full_path.push(PathPart::DoubleColon);
            delimiter_found = true;
        }

        let tt = or_unexpected_end(iter.peek().cloned(), Some(span))?;
        match tt {
            TokenTree::Punct(ref p) if p.as_char() == ':' => {
                if delimiter_found {
                    derive_error!("Unexpected double colon after double colon", tt.span());
                }
                // skip peeked
                let tt = iter.next().unwrap();
                let tt = or_unexpected_end(iter.next(), Some(tt.span()))?;
                require_token!(tt, TokenTree::Punct(p) if p.as_char() == ':', tt.span(), "Expected '::'");
                full_path.push(PathPart::DoubleColon);
            }
            TokenTree::Punct(p) if p.as_char() == '<' => {
                // skip peeked
                let tt = iter.next().unwrap();
                span = tt.span();

                loop {
                    let generic_type = recursive_get_field_type(iter)?;
                    generics.push(generic_type);

                    let tt = or_unexpected_end(iter.next(), Some(span))?;
                    span = tt.span();
                    match tt {
                        TokenTree::Punct(p) if p.as_char() == '>' => {
                            // generic finished
                            return Ok(Type {
                                ident: current_ident,
                                generics,
                                full_path,
                            });
                        }
                        TokenTree::Punct(p) if p.as_char() == ',' => {
                            // next generic arg
                        }
                        _ => {
                            derive_error!(
                                "Expected '>' or ',' after generic type argument",
                                tt.span()
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
