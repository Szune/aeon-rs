use crate::document::{AeonDocument, AeonMacro};
use crate::error::AeonDeserializeError;
use crate::lexer::Lexer;
use crate::token::Token;
use crate::value::AeonValue;
use crate::DeserializeResult;
use std::collections::HashMap;

pub struct Deserializer<'a> {
    lexer: Lexer<'a>,
}

macro_rules! require {
    ($actual:expr, arg $ok:path) => {
        if let Some(it) = $actual? {
            match it {
                $ok(a) => Ok(a),
                e => Err(AeonDeserializeError::deserialization(format!(
                    "Unexpected token: {:?}",
                    e
                ))),
            }
        } else {
            Err(AeonDeserializeError::deserialization(format!(
                "Unexpected token: {:?}",
                $actual
            )))
        }
    };
    ($actual:expr, $ok:path) => {
        if let Some(it) = $actual? {
            match it {
                $ok => Ok(()),
                e => Err(AeonDeserializeError::deserialization(format!(
                    "Unexpected token: {:?}",
                    e
                ))),
            }
        } else {
            Err(AeonDeserializeError::deserialization(format!(
                "Unexpected token: {:?}",
                $actual
            )))
        }
    };
}

impl<'a> Deserializer<'a> {
    pub fn new(code: &'a str) -> Deserializer<'a> {
        Deserializer {
            lexer: Lexer::new(code),
        }
    }

    pub fn deserialize(&mut self) -> DeserializeResult<AeonDocument> {
        let mut aeon = AeonDocument::new();
        'outer: loop {
            // result<option>
            let res = self.lexer.next()?;
            if let Some(it) = res {
                match it {
                    Token::At => {
                        // deserialize macro
                        self.deserialize_macro(&mut aeon)?;
                    }
                    Token::Identifier(ident) => {
                        // deserialize property
                        self.deserialize_property(&mut aeon, ident)?;
                    }
                    _ => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in main scope {:?}",
                            it
                        )))
                    }
                }
            } else {
                break 'outer;
            }
        }

        Ok(aeon)
    }

    fn deserialize_macro(&mut self, aeon: &mut AeonDocument) -> DeserializeResult<()> {
        let ident = require!(self.lexer.next(), arg Token::Identifier)?;
        require!(self.lexer.next(), Token::LeftParenthesis)?;
        let mut args = Vec::<String>::new();
        while let Some(tok) = self.lexer.next()? {
            match tok {
                Token::Identifier(id) => {
                    args.push(id);
                }
                Token::RightParenthesis => break,
                e => {
                    return Err(AeonDeserializeError::deserialization(format!(
                        "Unexpected token in macro definition: {:?}",
                        e
                    )))
                }
            }
            if let Some(comma_or_parens) = self.lexer.next()? {
                match comma_or_parens {
                    Token::Comma => (),
                    Token::RightParenthesis => break,
                    e => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in macro definition: {:?}",
                            e
                        )))
                    }
                }
            } else {
                return Err(AeonDeserializeError::deserialization(format!(
                    "Unterminated macro definition {}",
                    ident
                )));
            }
        }
        aeon.add_macro(AeonMacro::new(ident, args));
        Ok(())
    }

    fn deserialize_property(
        &mut self,
        aeon: &mut AeonDocument,
        prop_name: String,
    ) -> DeserializeResult<()> {
        require!(self.lexer.next(), Token::Colon)?;
        if let Some(tok) = self.lexer.next()? {
            let val = self.deserialize_property_value(&tok, aeon)?;
            aeon.add_property(&prop_name, val);
        } else {
            return Err(AeonDeserializeError::deserialization(format!(
                "Unterminated property value {}",
                prop_name
            )));
        }
        Ok(())
    }

    fn deserialize_property_value(
        &mut self,
        tok: &Token,
        aeon: &mut AeonDocument,
    ) -> DeserializeResult<AeonValue> {
        match tok {
            Token::Identifier(id) => self.deserialize_macro_use(id.clone(), aeon),
            Token::LeftBracket => self.deserialize_list(aeon),
            Token::LeftBrace => self.deserialize_map(aeon),
            maybe => match self.deserialize_constants(maybe) {
                t @ Ok(_) => t,
                Err(e) => Err(AeonDeserializeError::deserialization(format!(
                    "Unexpected token in property value: {:?}",
                    e
                ))),
            },
        }
    }

    fn deserialize_macro_use(
        &mut self,
        name: String,
        aeon: &mut AeonDocument,
    ) -> DeserializeResult<AeonValue> {
        require!(self.lexer.next(), Token::LeftParenthesis)?;
        let mut values = Vec::<AeonValue>::new();
        while let Some(tok) = self.lexer.next()? {
            match tok {
                Token::RightParenthesis => break,
                maybe => match self.deserialize_property_value(&maybe, aeon) {
                    Ok(t) => values.push(t),
                    Err(e) => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in macro call: {:?}",
                            e
                        )))
                    }
                },
            }
            if let Some(comma_or_parens) = self.lexer.next()? {
                match comma_or_parens {
                    Token::Comma => (),
                    Token::RightParenthesis => break,
                    e => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in macro call: {:?}",
                            e
                        )))
                    }
                }
            } else {
                return Err(AeonDeserializeError::deserialization(format!(
                    "Unterminated macro call {}",
                    name
                )));
            }
        }

        Ok(aeon.apply_macro(name, values))
    }

    fn deserialize_constants(&mut self, tok: &Token) -> DeserializeResult<AeonValue> {
        match tok {
            Token::String(s) => Ok(AeonValue::String(s.clone())),
            Token::Integer(i) => Ok(AeonValue::Integer(*i)),
            Token::Double(d) => Ok(AeonValue::Double(*d)),
            Token::True => Ok(AeonValue::Bool(true)),
            Token::False => Ok(AeonValue::Bool(false)),
            Token::Nil => Ok(AeonValue::Nil),
            s => Err(AeonDeserializeError::deserialization(format!(
                "Unexpected token {:?} when constant was expected",
                s
            ))),
        }
    }

    fn deserialize_list(&mut self, aeon: &mut AeonDocument) -> DeserializeResult<AeonValue> {
        let mut values = Vec::<AeonValue>::new();
        while let Some(tok) = self.lexer.next()? {
            match tok {
                Token::RightBracket => break,
                maybe => match self.deserialize_property_value(&maybe, aeon) {
                    Ok(t) => values.push(t),
                    Err(e) => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in list: {:?}",
                            e
                        )))
                    }
                },
            }
            if let Some(comma_or_parens) = self.lexer.next()? {
                match comma_or_parens {
                    Token::Comma => (),
                    Token::RightBracket => break,
                    e => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in list: {:?}",
                            e
                        )))
                    }
                }
            } else {
                return Err(AeonDeserializeError::deserialization(format!(
                    "Unterminated list with values {:?}",
                    values
                )));
            }
        }
        Ok(AeonValue::List(values))
    }

    fn deserialize_map_entry(
        &mut self,
        key: String,
        aeon: &mut AeonDocument,
    ) -> DeserializeResult<(String, AeonValue)> {
        require!(self.lexer.next(), Token::Colon)?;
        if let Some(next_tok) = self.lexer.next()? {
            match self.deserialize_property_value(&next_tok, aeon) {
                Ok(val) => Ok((key, val)),
                Err(e) => Err(AeonDeserializeError::deserialization(format!(
                    "Unexpected token in map: {:?}",
                    e
                ))),
            }
        } else {
            Err(AeonDeserializeError::deserialization(format!(
                "Unterminated map with key {:?}",
                key
            )))
        }
    }

    fn deserialize_map(&mut self, aeon: &mut AeonDocument) -> DeserializeResult<AeonValue> {
        let mut values = HashMap::<String, AeonValue>::new();
        while let Some(tok) = self.lexer.next()? {
            match tok {
                Token::String(key) => {
                    let entry = self.deserialize_map_entry(key, aeon)?;
                    values.insert(entry.0, entry.1);
                }
                Token::Identifier(key) => {
                    let entry = self.deserialize_map_entry(key, aeon)?;
                    values.insert(entry.0, entry.1);
                }
                Token::RightBrace => break,
                e => {
                    return Err(AeonDeserializeError::deserialization(format!(
                        "Unexpected token in map: was {:?}, expected string key",
                        e
                    )))
                }
            }

            if let Some(comma_or_parens) = self.lexer.next()? {
                match comma_or_parens {
                    Token::Comma => (),
                    Token::RightBrace => break,
                    e => {
                        return Err(AeonDeserializeError::deserialization(format!(
                            "Unexpected token in map: {:?}",
                            e
                        )))
                    }
                }
            } else {
                return Err(AeonDeserializeError::deserialization(format!(
                    "Unterminated map with values {:?}",
                    values
                )));
            }
        }
        Ok(AeonValue::Object(values))
    }
}
