use std::str::Chars;
use crate::token::Token;
pub struct Lexer<'a> {
    code: Chars<'a>,
    prev: Option<char>,
}

impl <'a> Lexer<'a> {
    pub fn new(code: &'a String) -> Lexer<'a> {
        Lexer {
            code: code.chars(),
            prev: None,
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>, String> {
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
            None => {
                self.perform_full_match()
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(t) = self.code.next() {
            if t == '\n' { 
                break; 
            }
        }
    }

    fn perform_full_match(&mut self) -> Result<Option<Token>, String> {
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


    fn perform_match(&mut self, now: char) -> Result<Option<Token>, String> {
        return match now {
            '(' => Ok(Some(Token::LeftParenthesis)),
            ')' => Ok(Some(Token::RightParenthesis)),
            '[' => Ok(Some(Token::LeftBracket)),
            ']' => Ok(Some(Token::RightBracket)),
            '{' => Ok(Some(Token::LeftBrace)),
            '}' => Ok(Some(Token::RightBrace)),
            ':' => Ok(Some(Token::Colon)),
            ',' => Ok(Some(Token::Comma)),
            '@' => Ok(Some(Token::At)),
            'a' ..= 'z' | 'A' ..= 'Z' => self.get_identifier(now),
            '"' => self.get_string(),
            '0'..='9' | '-' => self.get_number(now),
            _ => Err(format!("Unknown char {}",now)),
        }
    }

    fn get_identifier(&mut self, now: char) -> Result<Option<Token>, String> {
        let mut t_str = String::with_capacity(10);
        t_str.push(now);
        while let Some(t) = self.code.next() {
            match t {
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => {
                    t_str.push(t);
                },
                p => {
                    self.prev = Some(p);
                    t_str.shrink_to_fit();
                    return match t_str.as_str() {
                        "nil" => Ok(Some(Token::Nil)),
                        "true" => Ok(Some(Token::True)),
                        "false" => Ok(Some(Token::False)),
                        _ => Ok(Some(Token::Identifier(t_str))),
                    };
                },
            }
        }

        match t_str.as_str() {
            "nil" => Ok(Some(Token::Nil)),
            "true" => Ok(Some(Token::True)),
            "false" => Ok(Some(Token::False)),
            _ => Ok(Some(Token::Identifier(t_str))),
        }
    }

    fn get_number(&mut self, c: char) -> Result<Option<Token>, String> {
        let mut t_str = String::with_capacity(10);
        let mut flags = 0;
        t_str.push(c);
        while let Some(t) = self.code.next() {
            match t {
                '0' ..= '9' | '_' => {
                    if flags & 1 == 1 {
                        flags |= 2;
                    }
                    t_str.push(t);
                },
                '.' => {
                    if flags & 1 == 1 {
                        return Err(format!("Two decimal points in number: {}", t_str))
                    }
                    flags |= 1;
                    t_str.push(t);
                },
                p => {
                    self.prev = Some(p);
                    break;
                },
            }
        }

        t_str.shrink_to_fit();
        if flags & 1 == 1 {
            if flags & 2 == 2 {
                Ok(Some(Token::Double(t_str.parse().unwrap())))
            }
            else{
                Err("bad".into())
            }
        } else {
            Ok(Some(Token::Integer(t_str.parse().unwrap())))
        }
    }

    fn get_string(&mut self) -> Result<Option<Token>, String> {
        // TODO: allow 'text' as alternative string syntax as well?
        let mut t_str = String::with_capacity(10);
        while let Some(t) = self.code.next() {
            match t {
                '\\' => {
                    if let Some(next) = self.code.next() {
                        match next {
                            't' => t_str.push('\t'),
                            'n' => t_str.push('\n'),
                            '\\' => t_str.push('\\'),
                            '"' => t_str.push('"'),
                            _ => return Err(format!("Unescaped string: {}", t_str)),
                        }
                    } else {
                        return Err(format!("Unescaped string: {}", t_str));
                    }
                },
                '"' => {
                    t_str.shrink_to_fit();
                    return Ok(Some(Token::String(t_str)));
                },
                _ => {
                    t_str.push(t);
                },
            }
        }

        Err(format!("Unescaped string: {}", t_str))
    }

}


#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::lexer::Lexer;

    #[test]
    pub fn multiple_tokens() {
        let s = r#"hello:"world"world:1236"#.to_string();
        let mut lex = Lexer::new(&s);
        macro_rules! get_current( () => { lex.next().unwrap().unwrap() } );
        let mut current = get_current!();
        macro_rules! msg( ($str:expr) => { format!("expected {}, was {:?}", $str, current) } );

        // assert
        assert!(matches!(current, Token::Identifier(_)), msg!("ident hello"));
        current = get_current!();
        assert!(matches!(current, Token::Colon), msg!("colon"));
        current = get_current!();
        assert!(
            matches!(current, Token::String(_)), msg!("string \"world\""));
        current = get_current!();
        assert!(
            matches!(current, Token::Identifier(_)), msg!("ident world"));
        current = get_current!();
        assert!(
            matches!(current, Token::Colon), msg!("colon"));
        current = get_current!();
        assert!(
            matches!(current, Token::Integer(_)), msg!("integer 1236"));
    }

    #[test]
    pub fn integer_token() {
        assert!(matches!
                (Lexer::new(&"150".to_string())
                 .next()
                 .unwrap()
                 .unwrap(),
                     Token::Integer(150)),
                     "expected integer token");
    }

    #[test]
    pub fn nil_token() {
        assert!(matches!
                (Lexer::new(&"nil".to_string())
                 .next()
                 .unwrap()
                 .unwrap(),
                     Token::Nil),
                     "expected nil token");
    }


    #[test]
    pub fn string_token() {
        let t_str = 
                Lexer::new(&"\"hello world\"".to_string())
                 .next()
                 .unwrap()
                 .unwrap();
        match t_str {
            Token::String(s) => {
                assert_eq!(s, "hello world", "Expected string token");
            }
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    pub fn identifier_token() {
        let t_str = 
                Lexer::new(&"WORLD01_HELLo".to_string())
                 .next()
                 .unwrap()
                 .unwrap();
        match t_str {
            Token::Identifier(s) => {
                assert_eq!(s, "WORLD01_HELLo", "Expected identifier token");
            }
            _ => panic!("Expected identifier token"),
        }
    }

    #[test]
    pub fn double_token() {
        let double = 
                Lexer::new(&"19.13".to_string())
                 .next()
                 .unwrap()
                 .unwrap();
        match double {
            Token::Double(d) => {
                assert_eq!(d, 19.13, "Expected double token");
            }
            _ => panic!("Expected double token"),
        }
    }
}
