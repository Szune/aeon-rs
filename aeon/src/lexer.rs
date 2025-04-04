use crate::error::AeonDeserializeError;
use crate::flags;
use crate::token::Token;
use crate::DeserializeResult;
use std::str::Chars;

pub struct Lexer<'a> {
    code: Chars<'a>,
    prev: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Lexer<'a> {
        Lexer {
            code: code.chars(),
            prev: None,
        }
    }

    pub fn next(&mut self) -> DeserializeResult<Option<Token>> {
        match self.prev {
            Some(p) => {
                self.prev = None;
                if p == '#' {
                    self.skip_comment();
                    self.perform_full_match()
                } else if p.is_whitespace() {
                    self.perform_full_match()
                } else {
                    self.perform_match(p)
                }
            }
            None => self.perform_full_match(),
        }
    }

    fn skip_comment(&mut self) {
        for t in self.code.by_ref() {
            if t == '\n' {
                break;
            }
        }
    }

    fn perform_full_match(&mut self) -> DeserializeResult<Option<Token>> {
        while let Some(now) = self.code.next() {
            if now == '#' {
                self.skip_comment();
                continue;
            }
            if !now.is_whitespace() {
                return self.perform_match(now);
            }
        }

        Ok(None)
    }

    fn perform_match(&mut self, now: char) -> DeserializeResult<Option<Token>> {
        match now {
            '(' => Ok(Some(Token::LeftParenthesis)),
            ')' => Ok(Some(Token::RightParenthesis)),
            '[' => Ok(Some(Token::LeftBracket)),
            ']' => Ok(Some(Token::RightBracket)),
            '{' => Ok(Some(Token::LeftBrace)),
            '}' => Ok(Some(Token::RightBrace)),
            ':' => Ok(Some(Token::Colon)),
            ',' => Ok(Some(Token::Comma)),
            '@' => Ok(Some(Token::At)),
            'a'..='z' | 'A'..='Z' => self.get_identifier(now),
            '"' => self.get_string(),
            '0'..='9' | '-' => self.get_number(now),
            _ => Err(AeonDeserializeError::lexing(format!(
                "Unknown char {}",
                now
            ))),
        }
    }

    fn get_identifier(&mut self, now: char) -> DeserializeResult<Option<Token>> {
        let mut t_str = String::with_capacity(10);
        t_str.push(now);
        for t in self.code.by_ref() {
            match t {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    t_str.push(t);
                }
                p => {
                    self.prev = Some(p);
                    t_str.shrink_to_fit();
                    break;
                }
            }
        }

        match t_str.as_str() {
            "nil" => Ok(Some(Token::Nil)),
            "true" => Ok(Some(Token::True)),
            "false" => Ok(Some(Token::False)),
            _ => Ok(Some(Token::Identifier(t_str))),
        }
    }

    const FLAG_HAS_DECIMAL_POINT: u8 = 1;
    const FLAG_HAS_DECIMALS: u8 = 2;

    fn get_number(&mut self, c: char) -> DeserializeResult<Option<Token>> {
        let mut t_str = String::with_capacity(10);
        let mut num_flags = 0u8;
        t_str.push(c);
        for t in self.code.by_ref() {
            match t {
                '0'..='9' | '_' => {
                    if flags::has(num_flags, Self::FLAG_HAS_DECIMAL_POINT) {
                        flags::add(&mut num_flags, Self::FLAG_HAS_DECIMALS);
                    }
                    t_str.push(t);
                }
                '.' => {
                    if flags::has(num_flags, Self::FLAG_HAS_DECIMAL_POINT) {
                        return Err(AeonDeserializeError::lexing(format!(
                            "Two decimal points in number: {}",
                            t_str
                        )));
                    }
                    flags::add(&mut num_flags, Self::FLAG_HAS_DECIMAL_POINT);
                    t_str.push(t);
                }
                p => {
                    self.prev = Some(p);
                    break;
                }
            }
        }

        t_str.shrink_to_fit();
        if flags::has(num_flags, Self::FLAG_HAS_DECIMAL_POINT) {
            if flags::has(num_flags, Self::FLAG_HAS_DECIMALS) {
                Ok(Some(Token::Double(t_str.parse().unwrap())))
            } else {
                Err(AeonDeserializeError::lexing(format!(
                    "Trailing decimal point in number: {}",
                    t_str
                )))
            }
        } else {
            Ok(Some(Token::Integer(t_str.parse().unwrap())))
        }
    }

    fn get_string(&mut self) -> DeserializeResult<Option<Token>> {
        // TODO: allow 'text' as alternative string syntax as well?
        // TODO: use 0xFFFD for replacement char
        // TODO: write string byte + unicode syntax
        let mut t_str = String::with_capacity(10);
        while let Some(t) = self.code.next() {
            match t {
                '\\' => {
                    if let Some(next) = self.code.next() {
                        match next {
                            't' => t_str.push('\t'),
                            'r' => t_str.push('\r'),
                            'n' => t_str.push('\n'),
                            '\\' => t_str.push('\\'),
                            '"' => t_str.push('"'),
                            'x' => {}
                            'u' => {
                                let next = self.code.next();
                                if !matches!(next, Some('{')) {
                                    return Err(AeonDeserializeError::lexing(format!(
                                        "Expected '{{' after '\\u' in string: {}{:?}",
                                        t_str, next
                                    )));
                                }

                                let mut uc: [char; 4] = ['0'; 4];
                                let mut dig: u32 = 0;

                                for next in self.code.by_ref() {
                                    if next == '}' {
                                        break;
                                    }
                                    if dig == 4 {
                                        return Err(AeonDeserializeError::lexing(format!(
                                            "Expected exactly 4 hex digits in '\\u' in string: {}{:?}",
                                            t_str, next
                                        )));
                                    }
                                    if !next.is_ascii_hexdigit() {
                                        return Err(AeonDeserializeError::lexing(format!(
                                            "Expected '{{' after '\\u' in string: {}{:?}",
                                            t_str, next
                                        )));
                                    }
                                    uc[dig as usize] = next;
                                    dig += 1;
                                }
                                let uc: String = uc.iter().collect();
                                dig = match u32::from_str_radix(&uc, 16) {
                                    Ok(dig) => dig,
                                    Err(e) => {
                                        return Err(AeonDeserializeError::lexing(format!(
                                            "Invalid hex '{:04x}' in '\\u' in string: {}{:?}\n{:?}",
                                            dig, t_str, next, e
                                        )))
                                    }
                                };
                                let ch = match char::from_u32(dig) {
                                    Some(ch) => ch,
                                    None => {
                                        return Err(AeonDeserializeError::lexing(format!(
                                            "Invalid hex '{:04x}' in '\\u' in string: {}{:?}",
                                            dig, t_str, next
                                        )))
                                    }
                                };
                                t_str.push(ch);
                            }
                            _ => {
                                return Err(AeonDeserializeError::lexing(format!(
                                    "Unescaped string: {}{:?}",
                                    t_str, next
                                )))
                            }
                        }
                    } else {
                        return Err(AeonDeserializeError::lexing(format!(
                            "Unescaped string: {}",
                            t_str
                        )));
                    }
                }
                '"' => {
                    t_str.shrink_to_fit();
                    return Ok(Some(Token::String(t_str)));
                }
                _ => {
                    t_str.push(t);
                }
            }
        }

        Err(AeonDeserializeError::lexing(format!(
            "Unescaped string: {}",
            t_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    pub fn multiple_tokens() {
        let s = r#"hello:"world"world:1236"#.to_string();
        let mut lex = Lexer::new(&s);
        macro_rules! get_current( () => { lex.next().unwrap().unwrap() } );
        let mut current = get_current!();

        // assert
        assert!(
            matches!(current, Token::Identifier(_)),
            "expected ident hello, was {:?}",
            current
        );
        current = get_current!();
        assert!(
            matches!(current, Token::Colon),
            "expected colon, was {:?}",
            current
        );
        current = get_current!();
        assert!(
            matches!(current, Token::String(_)),
            "expected string \"world\", was {:?}",
            current
        );
        current = get_current!();
        assert!(
            matches!(current, Token::Identifier(_)),
            "expected ident world, was {:?}",
            current
        );
        current = get_current!();
        assert!(
            matches!(current, Token::Colon),
            "expected colon, was {:?}",
            current
        );
        current = get_current!();
        assert!(
            matches!(current, Token::Integer(_)),
            "expected integer 1236, was {:?}",
            current
        );
    }

    #[test]
    pub fn string_token_with_escaped_unicode_or_hex() {
        let result = Lexer::new(r#""\u{2714}""#).next().unwrap().unwrap();
        //eprintln!("{:#?}", result);
        assert!(
            matches!(
                result,
                Token::String(s) if s.as_str() == "\u{2714}" //"✔"
            ),
            "expected string token with string '\u{2714}'"
        );
    }

    #[test]
    pub fn integer_token() {
        assert!(
            matches!(
                Lexer::new("150").next().unwrap().unwrap(),
                Token::Integer(150)
            ),
            "expected integer token"
        );
    }

    #[test]
    pub fn nil_token() {
        assert!(
            matches!(Lexer::new("nil").next().unwrap().unwrap(), Token::Nil),
            "expected nil token"
        );
    }

    #[test]
    pub fn string_token() {
        let t_str = Lexer::new("\"hello world\"").next().unwrap().unwrap();
        match t_str {
            Token::String(s) => {
                assert_eq!(s, "hello world", "Expected string token");
            }
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    pub fn identifier_token() {
        let t_str = Lexer::new("WORLD01_HELLo").next().unwrap().unwrap();
        match t_str {
            Token::Identifier(s) => {
                assert_eq!(s, "WORLD01_HELLo", "Expected identifier token");
            }
            _ => panic!("Expected identifier token"),
        }
    }

    #[test]
    pub fn double_token() {
        let double = Lexer::new("19.13").next().unwrap().unwrap();
        match double {
            Token::Double(d) => {
                assert_eq!(d, 19.13, "Expected double token");
            }
            _ => panic!("Expected double token"),
        }
    }
}
